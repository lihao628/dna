use std::mem;

use alloy_primitives::B256;
use apibara_dna_common::{
    core::Cursor,
    error::{DnaError, Result},
    storage::{LocalStorageBackend, StorageBackend},
};
use error_stack::ResultExt;
use futures_util::{future, FutureExt, Stream, TryFutureExt};
use tokio::{io::AsyncWriteExt, sync::mpsc};
use tokio_stream::{wrappers::ReceiverStream, StreamExt};
use tokio_util::sync::CancellationToken;
use tracing::{debug, error, info};

use crate::{
    ingestion::models,
    segment::{conversion::model::GetCursor, SingleBlockBuilder},
};

use super::{ChainChange, RpcIngestionOptions, RpcProvider};

#[derive(Debug, Clone)]
pub enum BlockNumberOrHash {
    Number(u64),
    Hash(B256),
}

#[derive(Debug, Clone)]
pub enum BlockEvent {
    Started { finalized: Cursor },
    Finalized(Cursor),
    Ingested(Cursor),
    Invalidate,
}

pub struct BlockDownloaderService<C>
where
    C: Stream<Item = ChainChange> + Unpin + Send + Sync + 'static,
{
    chain_changes: C,
    downloader: InnerDownloader,
}

struct InnerDownloader {
    provider: RpcProvider,
    storage: LocalStorageBackend,
    options: RpcIngestionOptions,
}

impl<C> BlockDownloaderService<C>
where
    C: Stream<Item = ChainChange> + Unpin + Send + Sync + 'static,
{
    pub fn new(
        provider: RpcProvider,
        storage: LocalStorageBackend,
        chain_changes: C,
        options: RpcIngestionOptions,
    ) -> Self {
        let downloader = InnerDownloader {
            provider,
            storage,
            options,
        };

        Self {
            chain_changes,
            downloader,
        }
    }

    pub fn start(
        self,
        first_block_number: u64,
        ct: CancellationToken,
    ) -> impl Stream<Item = BlockEvent> {
        let (tx, rx) = mpsc::channel(128);

        tokio::spawn(
            self.download_loop(first_block_number, tx, ct)
                .inspect_err(|err| {
                    error!(err = ?err, "download loop returned with error");
                }),
        );

        ReceiverStream::new(rx)
    }

    async fn download_loop(
        self,
        first_block_number: u64,
        tx: mpsc::Sender<BlockEvent>,
        ct: CancellationToken,
    ) -> Result<()> {
        let mut chain_changes = self.chain_changes;
        let downloader = self.downloader;

        let (mut head, finalized) = {
            let Some(change) = chain_changes.next().await else {
                return Err(DnaError::Fatal)
                    .attach_printable("chain changes stream ended unexpectedly");
            };

            match change {
                ChainChange::Initialize { head, finalized } => (head, finalized),
                _ => {
                    return Err(DnaError::Fatal).attach_printable("expected chain initialization");
                }
            }
        };

        if finalized.number < first_block_number {
            return Err(DnaError::Fatal)
                .attach_printable("finalized block is behind the starting block");
        }

        info!(first_block_number, head = %head, finalized = %finalized, "starting block downloader");
        let mut current_block_number = first_block_number;
        let (mut download_fut, mut is_downloading) = if current_block_number < head.number {
            let f = downloader
                .download_block(current_block_number.into())
                .fuse()
                .boxed();
            (f, true)
        } else {
            (future::pending().boxed(), false)
        };

        let Ok(_) = tx
            .send(BlockEvent::Started {
                finalized: finalized.clone(),
            })
            .await
        else {
            return Ok(());
        };

        loop {
            tokio::select! {
                _ = ct.cancelled() => break,
                result = &mut download_fut => {
                    let ingested = result.attach_printable("block download failed")?;

                    let Ok(_) = tx.send(BlockEvent::Ingested(ingested)).await else {
                        break;
                    };

                    if current_block_number < head.number {
                        current_block_number += 1;
                        download_fut = downloader.download_block(current_block_number.into()).fuse().boxed();
                    } else {
                        is_downloading = false;
                    }
                }
                maybe_change = chain_changes.next() => {
                    let Some(change) = maybe_change else {
                        return Err(DnaError::Fatal)
                            .attach_printable("chain changes stream ended unexpectedly");
                    };

                    match change {
                        ChainChange::NewHead(new_head) => {
                            head = new_head;
                            if !is_downloading && current_block_number < head.number {
                                current_block_number += 1;
                                download_fut = downloader.download_block(current_block_number.into()).fuse().boxed();
                                is_downloading = true;
                            }
                        }
                        ChainChange::NewFinalized(new_finalized) => {
                            let Ok(_) = tx.send(BlockEvent::Finalized(new_finalized)).await else {
                                break;
                            };
                        }
                        ChainChange::Invalidate => {
                            let Ok(_) = tx.send(BlockEvent::Invalidate).await else {
                                break;
                            };
                        }
                        _ => {}
                    }
                }
            }
        }

        Ok(())
    }
}

impl InnerDownloader {
    async fn download_block(&self, block: BlockNumberOrHash) -> Result<Cursor> {
        use models::BlockTransactions;

        debug!(block = ?block, "ingesting block by number");

        let block_number = match block {
            BlockNumberOrHash::Number(number) => number,
            BlockNumberOrHash::Hash(_) => {
                todo!();
            }
        };

        let mut block = if self.options.get_block_by_number_with_transactions {
            self.provider
                .get_block_by_number_with_transactions(block_number)
                .await?
        } else {
            self.provider.get_block_by_number(block_number).await?
        };

        let cursor = block
            .header
            .cursor()
            .ok_or(DnaError::Fatal)
            .attach_printable("block header is missing hash")?;

        let empty_transactions = BlockTransactions::Hashes(Vec::new());
        let transactions = match mem::replace(&mut block.transactions, empty_transactions) {
            BlockTransactions::Full(transactions) => transactions,
            BlockTransactions::Hashes(hashes) => {
                self.provider
                    .get_transactions_by_hash(hashes.iter())
                    .await?
            }
            _ => panic!("ingesting uncle block"),
        };

        let receipts = if self.options.get_block_receipts_by_number {
            self.provider
                .get_block_receipts_by_number(block_number)
                .await?
        } else {
            self.provider
                .get_receipts_by_hash(block.transactions.hashes())
                .await?
        };

        // TODO: ideally we should reuse the same builder for all blocks to avoid allocating memory
        // every time.
        let mut builder = SingleBlockBuilder::new();
        builder.add_block_header(&block);
        builder.add_transactions(&transactions);
        builder.add_receipts(&receipts);
        builder.add_logs(&receipts);

        // Same for writer. We should not clone it every time.
        let prefix = format!("blocks/{}-{}", cursor.number, cursor.hash_as_hex());
        let mut writer = self.storage.clone().put(&prefix, "block").await?;
        let data_size = builder.write_block(&mut writer).await?;
        writer.shutdown().await.change_context(DnaError::Io)?;
        debug!(prefix, data_size, "wrote single block");
        builder.reset();

        Ok(cursor)
    }
}

impl Into<BlockNumberOrHash> for u64 {
    fn into(self) -> BlockNumberOrHash {
        BlockNumberOrHash::Number(self)
    }
}
