use std::{
    collections::{BTreeMap, VecDeque},
    net::TcpStream,
    sync::Arc,
    time::Instant,
};

use async_trait::async_trait;
use catplay_plist::CachingSerializer;
use catplay_tokio::{CItem, TcpSession, TcpSink};
use catplay_util::{AsyncShutdown, EventSleeper, EventToken};
use futures::{StreamExt, future::BoxFuture, stream::FuturesUnordered};
use log::{debug, warn};

use crate::{
    cipher::AirPlayCipherSaltType,
    msg::{Command, CommandError},
    rtsp_frame::{HttpStatus, RtspError, RtspResponse, RtspResult},
    rtsp_transport::{AirPlayTransportCodec, RtspFrame},
};

pub struct EventsServer<T: EventServerCallback> {
    shared_secret: Option<[u8; 32]>,
    handler: Arc<T>,

    pending: VecDeque<(u64, Command)>,
    inflight: FuturesUnordered<BoxFuture<'static, (u64, RtspResponse)>>,
    completed: BTreeMap<u64, RtspResponse>,

    next_response: Option<RtspResponse>,
    next_id: u64,
    next_send: u64,

    max_inflight: usize,
    max_observed_inflight: usize,

    serializer: CachingSerializer,
}

#[async_trait]
pub trait EventServerCallback: Send + Sync + 'static {
    async fn on_command(&self, command: Command) -> RtspResponse;
}

impl<T: EventServerCallback> EventsServer<T> {
    pub fn new(shared_secret: [u8; 32], handler: T, max_inflight: usize) -> Self {
        Self {
            shared_secret: Some(shared_secret),
            ..Self::unencrypted(handler, max_inflight)
        }
    }

    pub fn unencrypted(handler: T, max_inflight: usize) -> Self {
        Self {
            shared_secret: None,
            handler: Arc::new(handler),

            pending: VecDeque::new(),
            inflight: FuturesUnordered::new(),
            completed: BTreeMap::new(),

            next_id: 0,
            next_send: 0,
            next_response: None,

            max_inflight,
            max_observed_inflight: 0,
            serializer: Default::default(),
        }
    }

    fn command_error_status(err: &CommandError) -> HttpStatus {
        match err {
            CommandError::UnknownCommand(_) => HttpStatus::NotImplemented,
            CommandError::InvalidPayload(_) => HttpStatus::BadRequest,
            CommandError::FailedDeserialize(_, _) => HttpStatus::BadRequest,
        }
    }

    async fn handle_request(handler: Arc<T>, cmd: Command) -> RtspResponse {
        debug!("Received command: {cmd:?}");
        handler.on_command(cmd).await
    }

    fn pump_inflight(&mut self) {
        while self.inflight.len() < self.max_inflight {
            let Some((id, cmd)) = self.pending.pop_front() else { break };

            let handler = self.handler.clone();
            self.inflight.push(Box::pin(async move {
                let start = Instant::now();
                let resp = Self::handle_request(handler, cmd).await;
                debug!("Command {id} done in {:?}", Instant::now() - start);
                (id, resp)
            }));

            self.max_observed_inflight = self.max_observed_inflight.max(self.inflight.len());
            debug!(
                "Spawned command {id} (inflight={} vs max={})",
                self.inflight.len(),
                self.max_inflight
            );
        }
    }

    fn try_prepare_next_response(&mut self) -> bool {
        if self.next_response.is_some() {
            return true;
        }

        if let Some(resp) = self.completed.remove(&self.next_send) {
            self.next_send += 1;
            self.next_response.replace(resp);
            return true;
        }

        false
    }

    pub fn drain_next_response(&mut self, sink: &mut dyn TcpSink<Self>) -> RtspResult<()> {
        if let Some(resp) = self.next_response.take() {
            warn!("Writing response -> {resp}");
            sink.write(vec![resp.into()])?;
        }

        Ok(())
    }
}

#[async_trait]
impl<T: EventServerCallback> TcpSession for EventsServer<T> {
    type Codec = AirPlayTransportCodec;
    type Error = RtspError;

    fn init_stream(&mut self, stream: &mut TcpStream) -> Result<(), Self::Error> {
        stream.set_nodelay(true)?;
        std::os::linux::net::TcpStreamExt::set_quickack(stream, true)?;

        Ok(())
    }

    fn init_codec(&mut self) -> RtspResult<AirPlayTransportCodec> {
        let mut codec = AirPlayTransportCodec::new(true);
        if let Some(shared_secret) = self.shared_secret {
            codec.encrypt(AirPlayCipherSaltType::Events, shared_secret);
        }

        Ok(codec)
    }

    async fn on_msg(&mut self, sink: &mut dyn TcpSink<Self>, msgs: CItem<Self::Codec>) -> RtspResult<()> {
        self.drain_next_response(sink)?;

        for msg in msgs {
            let RtspFrame::Request(request) = msg else {
                return Err(RtspError::ProtocolViolationGeneric);
            };

            warn!("Received event frame <- {request}");

            let id = self.next_id;
            self.next_id += 1;

            if request.url != "/command" {
                self.completed
                    .insert(id, RtspResponse::new(None, HttpStatus::NotFound));
                continue;
            }

            match Command::deserialize_caching(&request.payload, &mut self.serializer) {
                Ok(cmd) => {
                    self.pending.push_back((id, cmd));
                    self.pump_inflight();
                }
                Err(err) => {
                    warn!("Failed to deserialize command: {err:?}");
                    let status = Self::command_error_status(&err);
                    self.completed.insert(id, RtspResponse::new(None, status));
                }
            }
        }

        self.try_prepare_next_response();
        self.drain_next_response(sink)?;

        Ok(())
    }

    async fn reconcile(&mut self, sink: &mut dyn TcpSink<Self>) -> Result<(), Self::Error> {
        self.drain_next_response(sink)?;
        Ok(())
    }
}

impl<T: EventServerCallback> EventSleeper for EventsServer<T> {
    async fn sleep(&mut self) -> Option<EventToken> {
        if self.inflight.is_empty() && self.pending.is_empty() {
            return None;
        }

        self.pump_inflight();

        if let Some((id, resp)) = self.inflight.next().await {
            self.completed.insert(id, resp);

            self.pump_inflight();

            if self.try_prepare_next_response() {
                // Wake up, FIFO response available
                debug!(
                    "FIFO completion; queue depth = {}; max observed = {}",
                    self.inflight.len(),
                    self.max_observed_inflight
                );
                Some(EventToken(1))
            } else {
                // Out-of-order completion, don't wakeup
                None
            }
        } else {
            None
        }
    }
}

impl<T: EventServerCallback> AsyncShutdown for EventsServer<T> {}
