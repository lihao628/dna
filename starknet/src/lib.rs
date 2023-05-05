pub mod core;
pub mod db;
pub mod healer;
pub mod ingestion;
pub mod node;
pub mod provider;
pub mod server;
pub mod stream;

pub use crate::node::StarkNetNode;
pub use crate::provider::HttpProvider;

pub use apibara_node::db::libmdbx::NoWriteMap;

use std::path::PathBuf;

use crate::server::{MetadataKeyRequestObserver, SimpleRequestObserver};
use anyhow::Result;
use apibara_node::{db::default_data_dir, o11y::init_opentelemetry};
use clap::Args;
use tempdir::TempDir;
use tokio_util::sync::CancellationToken;
use tracing::info;

#[derive(Args)]
pub struct StartArgs {
    /// StarkNet RPC address.
    #[arg(long, env)]
    pub rpc: String,
    /// Data directory. Defaults to `$XDG_DATA_HOME`.
    #[arg(long, env)]
    pub data: Option<PathBuf>,
    /// Indexer name. Defaults to `starknet`.
    #[arg(long, env)]
    pub name: Option<String>,
    /// Wait for RPC to be available before starting.
    #[arg(long, env)]
    pub wait_for_rpc: bool,
    /// Create a temporary directory for data, deleted when devnet is closed.
    #[arg(long, env)]
    pub devnet: bool,
    /// Use the specified metadata key for tracing and metering.
    #[arg(long, env)]
    pub use_metadata: Vec<String>,
}

pub async fn start_node(args: StartArgs, cts: CancellationToken) -> Result<()> {
    init_opentelemetry()?;

    let mut node =
        StarkNetNode::<HttpProvider, SimpleRequestObserver, NoWriteMap>::builder(&args.rpc)?
            .with_request_observer(MetadataKeyRequestObserver::new(args.use_metadata));

    // Setup cancellation for graceful shutdown
    ctrlc::set_handler({
        let cts = cts.clone();
        move || {
            cts.cancel();
        }
    })?;

    if args.devnet {
        let tempdir = TempDir::new("apibara")?;
        info!("starting in devnet mode");
        node.with_datadir(tempdir.path().to_path_buf());
    } else if let Some(datadir) = args.data {
        info!("using user-provided datadir");
        node.with_datadir(datadir);
    } else if let Some(name) = args.name {
        let datadir = default_data_dir()
            .map(|p| p.join(name))
            .expect("no datadir");
        node.with_datadir(datadir);
    }

    node.build()?.start(cts.clone(), args.wait_for_rpc).await?;

    Ok(())
}
