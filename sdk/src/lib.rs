pub mod configuration;

use core::fmt;
use std::{
    marker::PhantomData,
    pin::Pin,
    task::{Context, Poll},
    time::Duration,
};

use apibara_core::node::v1alpha2::{
    stream_client::StreamClient as ProtoStreamClient, stream_data_response, Cursor, DataFinality,
    StatusRequest, StatusResponse, StreamDataRequest, StreamDataResponse,
};
use error_stack::{Result, ResultExt};
use futures::Stream;
use pin_project::pin_project;
use prost::Message;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::{self, Sender};
use tokio_stream::{wrappers::ReceiverStream, StreamExt, Timeout};
use tonic::{
    codegen::InterceptedService,
    metadata::{AsciiMetadataValue, KeyAndValueRef},
    service::Interceptor,
    transport::Channel,
    Streaming,
};
use tracing::debug;

// Re-export tonic Uri
pub use http::uri::InvalidUri;
pub use tonic::{
    metadata::{
        errors::{InvalidMetadataKey, InvalidMetadataValue},
        MetadataMap,
    },
    transport::Uri,
};

pub type MetadataKey = tonic::metadata::MetadataKey<tonic::metadata::Ascii>;
pub type MetadataValue = tonic::metadata::MetadataValue<tonic::metadata::Ascii>;

pub use crate::configuration::Configuration;

#[derive(Debug)]
pub struct ClientError;

impl error_stack::Context for ClientError {}

impl fmt::Display for ClientError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("the DNA client encountered an error")
    }
}

/// A message generated by [DataStream].
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum DataMessage<D: Message + Default> {
    /// A new batch of data.
    Data {
        /// The batch starting cursor.
        cursor: Option<Cursor>,
        /// The batch end cursor.
        ///
        /// Use this value as the start cursor to receive data for the next batch.
        end_cursor: Cursor,
        /// The data finality.
        finality: DataFinality,
        /// The batch of data.
        batch: Vec<D>,
    },
    /// Invalidate all data received after the given cursor.
    Invalidate {
        /// The cursor.
        cursor: Option<Cursor>,
    },
    Heartbeat,
}

/// Data stream client.
#[derive(Clone)]
pub struct StreamClient {
    inner: ProtoStreamClient<InterceptedService<Channel, MetadataInterceptor>>,
    timeout: Duration,
}

/// Data stream builder.
///
/// This struct is used to configure and connect to an Apibara data stream.
pub struct ClientBuilder {
    token: Option<String>,
    max_message_size: Option<usize>,
    metadata: MetadataMap,
    timeout: Duration,
}

/// A stream of on-chain data.
#[derive(Debug)]
#[pin_project]
pub struct DataStream<F, D, C>
where
    F: Message + Default,
    D: Message + Default,
    C: Stream<Item = Configuration<F>> + Send + Sync + 'static,
{
    stream_id: u64,
    #[pin]
    configuration_stream: C,
    #[pin]
    inner: Pin<Box<Timeout<Streaming<StreamDataResponse>>>>,
    inner_tx: Sender<StreamDataRequest>,
    _data: PhantomData<D>,
}

impl ClientBuilder {
    /// Use the given `token` to authenticate with the server.
    pub fn with_bearer_token(mut self, token: Option<String>) -> Self {
        self.token = token;
        self
    }

    /// Use the given `metadata` when connecting to the server.
    ///
    /// Notice: metadata will be merged with the authentication header if any.
    pub fn with_metadata(mut self, metadata: MetadataMap) -> Self {
        self.metadata = metadata;
        self
    }

    /// Set the maximum time to wait for a message from the server.
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    pub fn with_max_message_size(mut self, message_size: usize) -> Self {
        self.max_message_size = Some(message_size);
        self
    }

    /// Create and connect to the stream at the given url.
    ///
    /// If a configuration was provided, the client will immediately send it to the server upon
    /// connecting.
    pub async fn connect(self, url: Uri) -> Result<StreamClient, ClientError> {
        let channel = Channel::builder(url)
            .connect()
            .await
            .change_context(ClientError)?;
        let interceptor = MetadataInterceptor::new(self.metadata, self.token)?;

        let mut default_client = ProtoStreamClient::with_interceptor(channel, interceptor);
        default_client = if let Some(max_message_size) = self.max_message_size {
            default_client.max_decoding_message_size(max_message_size)
        } else {
            default_client
        };

        Ok(StreamClient {
            inner: default_client,
            timeout: self.timeout,
        })
    }
}

impl Default for ClientBuilder {
    fn default() -> Self {
        ClientBuilder {
            token: None,
            max_message_size: None,
            metadata: MetadataMap::default(),
            timeout: Duration::from_secs(45),
        }
    }
}

impl StreamClient {
    /// Start streaming data.
    ///
    /// If a configuration was provided, the client will immediately send it to the server upon
    /// connecting.
    pub async fn start_stream<F, D, C>(
        mut self,
        configuration: C,
    ) -> Result<DataStream<F, D, C>, ClientError>
    where
        F: Message + Default,
        D: Message + Default,
        C: Stream<Item = Configuration<F>> + Send + Sync + 'static,
    {
        let (inner_tx, inner_rx) = mpsc::channel(128);

        let inner_stream = self
            .inner
            .stream_data(ReceiverStream::new(inner_rx))
            .await
            .change_context(ClientError)?
            .into_inner()
            .timeout(self.timeout);

        let inner_stream = Box::pin(inner_stream);

        let stream = DataStream {
            stream_id: 0,
            configuration_stream: configuration,
            inner: inner_stream,
            inner_tx,
            _data: PhantomData::default(),
        };

        Ok(stream)
    }

    /// Request the stream status.
    pub async fn status(mut self) -> Result<StatusResponse, ClientError> {
        let request = StatusRequest {};
        let response = self
            .inner
            .status(request)
            .await
            .change_context(ClientError)?;
        Ok(response.into_inner())
    }
}

impl<F, D, C> Stream for DataStream<F, D, C>
where
    F: Message + Default,
    D: Message + Default,
    C: Stream<Item = Configuration<F>> + Send + Sync + 'static,
{
    type Item = Result<DataMessage<D>, ClientError>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.project();
        match this.configuration_stream.poll_next(cx) {
            Poll::Ready(None) => return Poll::Ready(None),
            Poll::Ready(Some(configuration)) => {
                (*this.stream_id) += 1;
                let request = StreamDataRequest {
                    stream_id: Some(*this.stream_id),
                    batch_size: Some(configuration.batch_size),
                    starting_cursor: configuration.starting_cursor,
                    finality: configuration.finality.map(|f| f as i32),
                    filter: configuration.filter.encode_to_vec(),
                };

                this.inner_tx
                    .try_send(request)
                    .change_context(ClientError)?;
                cx.waker().wake_by_ref();
                return Poll::Pending;
            }
            Poll::Pending => {}
        }

        match this.inner.poll_next(cx) {
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
            Poll::Ready(Some(Err(e))) => Poll::Ready(Some(Err(e).change_context(ClientError))),
            Poll::Ready(Some(Ok(inner_message))) => match inner_message {
                Err(err) => Poll::Ready(Some(Err(err).change_context(ClientError))),
                Ok(response) => {
                    if response.stream_id != *this.stream_id {
                        cx.waker().wake_by_ref();
                        return Poll::Pending;
                    }

                    match response.message {
                        None => {
                            cx.waker().wake_by_ref();
                            Poll::Pending
                        }
                        Some(stream_data_response::Message::Data(data)) => {
                            let batch = data
                                .data
                                .into_iter()
                                .map(|b| D::decode(b.as_slice()))
                                .filter_map(|b| b.ok())
                                .collect::<Vec<D>>();
                            let message = DataMessage::Data {
                                cursor: data.cursor,
                                end_cursor: data.end_cursor.unwrap_or_default(),
                                finality: DataFinality::from_i32(data.finality).unwrap_or_default(),
                                batch,
                            };
                            Poll::Ready(Some(Ok(message)))
                        }
                        Some(stream_data_response::Message::Invalidate(invalidate)) => {
                            let message = DataMessage::Invalidate {
                                cursor: invalidate.cursor,
                            };
                            Poll::Ready(Some(Ok(message)))
                        }
                        Some(stream_data_response::Message::Heartbeat(_)) => {
                            debug!("received heartbeat");
                            cx.waker().wake_by_ref();
                            Poll::Pending
                        }
                    }
                }
            },
        }
    }
}

impl<D: Message + Default> DataMessage<D> {
    pub fn from_stream_data_response(response: StreamDataResponse) -> Option<Self> {
        match response.message {
            None => None,
            Some(stream_data_response::Message::Heartbeat(_)) => Some(DataMessage::Heartbeat),
            Some(stream_data_response::Message::Data(data)) => {
                let batch = data
                    .data
                    .into_iter()
                    .map(|b| D::decode(b.as_slice()))
                    .filter_map(|b| b.ok())
                    .collect::<Vec<D>>();
                let message = DataMessage::Data {
                    cursor: data.cursor,
                    end_cursor: data.end_cursor.unwrap_or_default(),
                    finality: DataFinality::from_i32(data.finality).unwrap_or_default(),
                    batch,
                };
                Some(message)
            }
            Some(stream_data_response::Message::Invalidate(invalidate)) => {
                let message = DataMessage::Invalidate {
                    cursor: invalidate.cursor,
                };
                Some(message)
            }
        }
    }
}

#[derive(Clone)]
pub struct MetadataInterceptor {
    metadata: MetadataMap,
    token_meta: Option<(&'static str, MetadataValue)>,
}

impl MetadataInterceptor {
    fn new(metadata: MetadataMap, token: Option<String>) -> Result<Self, ClientError> {
        // parse authorization token outside of the interceptor
        let token_meta = if let Some(token) = token {
            let token = AsciiMetadataValue::try_from(format!("Bearer {token}"))
                .change_context(ClientError)?;
            Some(("authorization", token))
        } else {
            None
        };

        Ok(Self {
            metadata,
            token_meta,
        })
    }
}

impl Interceptor for MetadataInterceptor {
    fn call(
        &mut self,
        mut request: tonic::Request<()>,
    ) -> std::result::Result<tonic::Request<()>, tonic::Status> {
        let req_meta = request.metadata_mut();

        for kv in self.metadata.iter() {
            match kv {
                KeyAndValueRef::Ascii(key, value) => {
                    req_meta.insert(key, value.clone());
                }
                KeyAndValueRef::Binary(key, value) => {
                    req_meta.insert_bin(key, value.clone());
                }
            }
        }

        if let Some((key, value)) = self.token_meta.clone() {
            req_meta.insert(key, value);
        }

        Ok(request)
    }
}
