use std::{
    net::{SocketAddr, TcpStream},
    sync::{Arc, Mutex},
};

use async_trait::async_trait;
use catplay_tokio::{CItem, TcpSession, TcpSink};
use catplay_tracing::{strace, tracer::SessionTracer};
use catplay_util::{AsyncShutdown, EventReconciler, EventSink, EventSleeper, EventToken, event_select, notify::Notify};
use log::{debug, warn};

use crate::{
    cipher::AirPlayCipherSaltType,
    rtsp_frame::{RtspDrain, RtspError, RtspQueue, RtspResult},
    rtsp_transport::{AirPlayTransportCodec, RtspFrame},
};

pub struct RtspTransmitter<T: RtspTransmitterCallback> {
    rtsp_drain: RtspDrain,
    inner: Arc<Inner>,

    pending_init: Option<RtspTransmitterEvent>,

    callback: T,

    tracer: Option<SessionTracer>,
    peer_addr: Option<SocketAddr>,
    local_addr: Option<SocketAddr>,
}

struct Inner {
    pending_encrypt: Mutex<Option<[u8; 32]>>,
    pending_close: Mutex<bool>,
    pending_notify: Notify,
}

#[derive(Clone)]
pub struct RtspTransmitterHandle {
    inner: Arc<Inner>,
}

pub enum RtspTransmitterEvent {
    SetBindIp(SocketAddr),
    SetPeerIp(SocketAddr),

    Init { queue: RtspQueue, handle: RtspTransmitterHandle },

    Connected,
    ConnectionFailed,

    Encrypted { shared_secret: [u8; 32] },

    Eof(RtspError),
}

pub trait RtspTransmitterCallback:
    EventSink<RtspTransmitterEvent, ()> + EventSleeper + EventReconciler<Error = RtspError> + AsyncShutdown
{
}

impl RtspTransmitterHandle {
    fn new(inner: Arc<Inner>) -> Self {
        Self { inner }
    }

    pub fn encrypt(&self, key: [u8; 32]) {
        self.inner.pending_encrypt.lock().unwrap().replace(key);
        self.inner.pending_notify.notify();
    }

    pub fn close(&self) {
        *self.inner.pending_close.lock().unwrap() = true;
        self.inner.pending_notify.notify();
    }
}

impl<T: RtspTransmitterCallback> RtspTransmitter<T> {
    pub fn new(callback: T) -> Self {
        let (queue, drain) = RtspQueue::new();

        let inner = Arc::new(Inner {
            pending_encrypt: Mutex::new(None),
            pending_close: Mutex::new(false),
            pending_notify: Notify::new(),
        });
        let handle = RtspTransmitterHandle::new(inner.clone());

        RtspTransmitter {
            rtsp_drain: drain,
            inner,

            callback,
            pending_init: Some(RtspTransmitterEvent::Init { queue, handle }),
            tracer: None,
            local_addr: None,
            peer_addr: None,
        }
    }
}

#[async_trait]
impl<T: RtspTransmitterCallback> TcpSession for RtspTransmitter<T> {
    type Codec = AirPlayTransportCodec;
    type Error = RtspError;

    fn init_stream(&mut self, stream: &mut TcpStream) -> Result<(), Self::Error> {
        stream.set_nodelay(true)?;
        std::os::linux::net::TcpStreamExt::set_quickack(stream, true)?;

        Ok(())
    }

    fn init_codec(&mut self) -> RtspResult<AirPlayTransportCodec> {
        Ok(AirPlayTransportCodec::new(false))
    }

    async fn reconcile(&mut self, sink: &mut dyn TcpSink<Self>) -> RtspResult<()> {
        // Encryption flow is a bit more complex than RtspReceiver side
        // However, this code is enough to establish a happens-before and ensure all pending requests go properly encrypted, since the callback

        let mut call_encrypted_event = None;

        if let Some(shared_secret) = self.inner.pending_encrypt.lock().unwrap().take() {
            debug!("Encrypting connection now");

            sink.codec_mut()
                .encrypt(AirPlayCipherSaltType::Control, shared_secret);
            call_encrypted_event.replace(shared_secret);
        }

        if let Some(shared_secret) = call_encrypted_event {
            self.callback
                .on_event(RtspTransmitterEvent::Encrypted { shared_secret })
                .await;
        }

        let out: Vec<_> = self
            .rtsp_drain
            .drain()
            .into_iter()
            .map(|req| req.into())
            .collect();
        for req in &out {
            if let Some(tracer) = self.tracer.as_ref()
                && let RtspFrame::Request(req) = req
            {
                let req_for_trace = req.clone();
                strace!(tracer, "Sending request: \n{}", req_for_trace);
            }

            warn!("Sending request: \n{req}");
        }

        sink.write(out)?;

        if *self.inner.pending_close.lock().unwrap() {
            return Err(RtspError::DisconnectNow);
        }

        self.callback.reconcile().await?;
        Ok(())
    }

    async fn on_msg(&mut self, _sink: &mut dyn TcpSink<Self>, msgs: CItem<Self::Codec>) -> RtspResult<()> {
        if self.tracer.is_none() && self.local_addr.is_some() && self.peer_addr.is_some() {
            self.tracer.replace(SessionTracer::new(format!(
                "rtsp_tx-{:?}-{:?}",
                self.peer_addr.unwrap(),
                self.local_addr.unwrap()
            )));
        }

        for msg in msgs {
            let RtspFrame::Response(resp) = msg else {
                return Err(RtspError::ProtocolViolationGeneric);
            };

            #[cfg(feature = "tracing")]
            if let Some(tracer) = self.tracer.as_ref() {
                let resp_for_trace = resp.clone();
                strace!(tracer, "Received response: \n{}", resp_for_trace);
            }

            warn!("Received response: \n{resp}");

            self.rtsp_drain.feed(resp);
        }

        Ok(())
    }

    async fn on_eof(&mut self, status: Option<RtspError>) {
        warn!("Observed EOF on transmitter: {status:?}");
        self.rtsp_drain.close();
        let _ = self
            .callback
            .on_event(RtspTransmitterEvent::Eof(status.unwrap_or(RtspError::Closed)))
            .await;
    }

    async fn on_connected(&mut self) -> RtspResult<()> {
        self.callback
            .on_event(self.pending_init.take().unwrap())
            .await;
        let _ = self
            .callback
            .on_event(RtspTransmitterEvent::Connected)
            .await;
        Ok(())
    }

    async fn on_peer_addr(&mut self, peer_addr: SocketAddr) -> Result<(), Self::Error> {
        debug!("peer_addr = {peer_addr}");
        self.peer_addr.replace(peer_addr);
        self.callback
            .on_event(RtspTransmitterEvent::SetPeerIp(peer_addr))
            .await;
        Ok(())
    }

    async fn on_local_addr(&mut self, local_addr: SocketAddr) -> Result<(), Self::Error> {
        debug!("local_addr = {local_addr}");
        self.local_addr.replace(local_addr);
        self.callback
            .on_event(RtspTransmitterEvent::SetBindIp(local_addr))
            .await;
        Ok(())
    }
}

impl<T: RtspTransmitterCallback> EventSleeper for RtspTransmitter<T> {
    async fn sleep(&mut self) -> Option<EventToken> {
        event_select!(self.callback, self.inner.pending_notify.clone(), self.rtsp_drain)
    }
}

impl<T: RtspTransmitterCallback> AsyncShutdown for RtspTransmitter<T> {
    async fn shutdown(&mut self) {}
}
