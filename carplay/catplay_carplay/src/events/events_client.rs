use std::{net::TcpStream, time::Duration};

use async_trait::async_trait;
use catplay_plist::CachingSerializer;
use catplay_tokio::{CItem, TcpSession, TcpSink};
use catplay_util::{AsyncShutdown, EventSleeper};
use log::warn;

use crate::{
    cipher::AirPlayCipherSaltType,
    msg::Command,
    rtsp_frame::{RtspDrain, RtspError, RtspFuture, RtspMethod, RtspQueue, RtspRequest, RtspResult},
    rtsp_transport::{AirPlayTransportCodec, RtspFrame},
};

#[derive(EventSleeper)]
pub struct EventsClient {
    shared_secret: Option<[u8; 32]>,
    rtsp: RtspQueue,
    #[sleep]
    rtsp_drain: RtspDrain,
}

impl EventsClient {
    pub fn new(shared_secret: [u8; 32]) -> Self {
        let (rtsp, rtsp_drain) = RtspQueue::fifo();
        Self {
            rtsp,
            rtsp_drain,
            shared_secret: Some(shared_secret),
        }
    }

    pub fn unencrypted() -> Self {
        let (rtsp, rtsp_drain) = RtspQueue::fifo();
        Self {
            rtsp,
            rtsp_drain,
            shared_secret: None,
        }
    }

    pub fn rtsp_client(&self) -> RtspQueue {
        self.rtsp.clone()
    }
}

pub trait CommandsClient {
    fn send_command(&self, cmd: &Command, timeout: Duration) -> RtspResult<RtspFuture>;

    fn send_command_caching(&self, cmd: &Command, timeout: Duration, serializer: &mut CachingSerializer) -> RtspResult<RtspFuture>;
}

impl CommandsClient for RtspQueue {
    fn send_command(&self, cmd: &Command, timeout: Duration) -> RtspResult<RtspFuture> {
        let ev = cmd.serialize().map_err(RtspError::SerializationFailed)?;
        let fut = self.request_timeout(RtspRequest::with_payload(RtspMethod::Post, "/command", &ev[..]), timeout);
        Ok(fut)
    }

    fn send_command_caching(&self, cmd: &Command, timeout: Duration, serializer: &mut CachingSerializer) -> RtspResult<RtspFuture> {
        let ev = cmd
            .serialize_caching(serializer)
            .map_err(RtspError::SerializationFailed)?;
        let fut = self.request_timeout(RtspRequest::with_payload(RtspMethod::Post, "/command", ev), timeout);
        Ok(fut)
    }
}

#[async_trait]
impl TcpSession for EventsClient {
    type Codec = AirPlayTransportCodec;
    type Error = RtspError;

    fn init_stream(&mut self, stream: &mut TcpStream) -> Result<(), Self::Error> {
        stream.set_nodelay(true)?;
        std::os::linux::net::TcpStreamExt::set_quickack(stream, true)?;

        Ok(())
    }

    fn init_codec(&mut self) -> RtspResult<AirPlayTransportCodec> {
        let mut codec = AirPlayTransportCodec::new(false);
        if let Some(shared_secret) = self.shared_secret {
            codec.encrypt(AirPlayCipherSaltType::Events, shared_secret);
        }
        Ok(codec)
    }

    async fn on_msg(&mut self, _sink: &mut dyn TcpSink<Self>, msgs: CItem<Self::Codec>) -> RtspResult<()> {
        for msg in msgs {
            if let RtspFrame::Response(resp) = msg {
                warn!("Event socket: <- {resp}");
                self.rtsp_drain.feed(resp);
            }
        }
        Ok(())
    }

    async fn reconcile(&mut self, _sink: &mut dyn TcpSink<Self>) -> RtspResult<()> {
        for req in self.rtsp_drain.drain() {
            warn!("Event socket: -> {}", &req);
            _sink.write(vec![req.into()])?;
        }

        Ok(())
    }

    async fn on_eof(&mut self, _status: Option<RtspError>) {
        self.rtsp_drain.close();
    }
}

impl AsyncShutdown for EventsClient {}
