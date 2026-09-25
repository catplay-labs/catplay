use std::{
    net::{SocketAddr, TcpStream},
    time::Instant,
};

use async_trait::async_trait;
use bytes::BytesMut;
use catplay_tokio::{CItem, TcpSession, TcpSink};
use catplay_tracing::{strace, tracer::SessionTracer};
use catplay_util::{AsyncShutdown, EventReconciler, EventSink, EventSleeper, EventToken};
use log::{debug, warn};

use crate::{
    cipher::AirPlayCipherSaltType,
    rtsp_frame::{HttpStatus, RtspError, RtspRequest, RtspResponse, RtspResult},
    rtsp_transport::{AirPlayTransportCodec, RtspFrame},
};

pub struct RtspReceiver<T: RtspReceiverCallback> {
    last_cseq: Option<u32>,
    payload_response_cache: BytesMut,

    callback: T,
    tracer: Option<SessionTracer>,
    peer_addr: Option<SocketAddr>,
    local_addr: Option<SocketAddr>,
}

#[derive(Debug)]
pub enum RtspReceiverEvent<'a> {
    SetBindIp(SocketAddr),
    SetPeerIp(SocketAddr),

    Request {
        request: &'a RtspRequest,
        response: &'a mut RtspResponse,
    },
    Eof(RtspError),
}

pub struct RtspReceiverEventResult {
    pub encrypt_after_response: Option<[u8; 32]>,
    pub disconnect_after_response: bool,
    pub response: Option<RtspResult<RtspResponse>>,
}

impl RtspReceiverEventResult {
    pub fn ready() -> Self {
        Self {
            encrypt_after_response: None,
            disconnect_after_response: false,
            response: None,
        }
    }

    pub fn encrypt_ready(shared_secret: [u8; 32]) -> Self {
        Self {
            encrypt_after_response: Some(shared_secret),
            disconnect_after_response: false,
            response: None,
        }
    }

    pub fn new(response: RtspResult<RtspResponse>) -> Self {
        Self {
            encrypt_after_response: None,
            disconnect_after_response: false,
            response: Some(response),
        }
    }

    pub fn encrypt(response: RtspResult<RtspResponse>, shared_secret: [u8; 32]) -> Self {
        Self {
            encrypt_after_response: Some(shared_secret),
            disconnect_after_response: false,
            response: Some(response),
        }
    }

    pub fn disconnect(response: RtspResult<RtspResponse>) -> Self {
        Self {
            encrypt_after_response: None,
            disconnect_after_response: true,
            response: Some(response),
        }
    }

    pub fn noop() -> Self {
        Self {
            encrypt_after_response: None,
            disconnect_after_response: false,
            response: None,
        }
    }
}

pub trait RtspReceiverCallback:
    for<'a> EventSink<RtspReceiverEvent<'a>, RtspReceiverEventResult> + AsyncShutdown + EventSleeper + EventReconciler<Error = RtspError>
{
}

impl<T: RtspReceiverCallback> RtspReceiver<T> {
    const RESPONSE_PAYLOAD_CACHE_MAX: usize = 4096;
    // Some CarPlay dongles violate CSeq sanity so changing this allows communication with them.
    const ENFORCE_CSEQ_SANITY: bool = true;

    pub fn new(callback: T) -> Self {
        Self {
            last_cseq: None,
            payload_response_cache: BytesMut::with_capacity(Self::RESPONSE_PAYLOAD_CACHE_MAX),
            callback,
            tracer: None,
            peer_addr: None,
            local_addr: None,
        }
    }
}

#[async_trait]
impl<T: RtspReceiverCallback> TcpSession for RtspReceiver<T> {
    type Codec = AirPlayTransportCodec;
    type Error = RtspError;

    fn init_stream(&mut self, stream: &mut TcpStream) -> Result<(), Self::Error> {
        stream.set_nodelay(true)?;
        std::os::linux::net::TcpStreamExt::set_quickack(stream, true)?;

        Ok(())
    }

    fn init_codec(&mut self) -> RtspResult<AirPlayTransportCodec> {
        Ok(AirPlayTransportCodec::new(true))
    }

    async fn on_msg(&mut self, sink: &mut dyn TcpSink<Self>, msgs: CItem<Self::Codec>) -> RtspResult<()> {
        if self.tracer.is_none() && self.local_addr.is_some() && self.peer_addr.is_some() {
            self.tracer.replace(SessionTracer::new(format!(
                "rtsp_rx-{:?}-{:?}",
                self.peer_addr.unwrap(),
                self.local_addr.unwrap()
            )));
        }

        for msg in msgs {
            let RtspFrame::Request(req) = msg else {
                return Err(RtspError::ProtocolViolationGeneric);
            };

            let start = Instant::now();

            warn!("Received request: \n{req}");

            #[cfg(feature = "tracing")]
            if let Some(tracer) = self.tracer.as_ref() {
                let req_for_trace = req.clone();
                strace!(tracer, "Received request: \n{}", req_for_trace);
            }

            if req.cseq.is_none() {
                return Err(RtspError::ProtocolViolation("missing valid CSeq header"));
            }

            let cseq = req.cseq.unwrap();

            if Self::ENFORCE_CSEQ_SANITY && self.last_cseq.is_some() && cseq <= self.last_cseq.unwrap() {
                return Err(RtspError::CSeqSanity(cseq, self.last_cseq.unwrap()));
            }

            self.last_cseq.replace(cseq);

            self.payload_response_cache.clear();
            self.payload_response_cache
                .reserve(Self::RESPONSE_PAYLOAD_CACHE_MAX);
            let payload_cache = self.payload_response_cache.split_off(0);

            let mut response = RtspResponse::new(req.cseq, HttpStatus::Ok);
            response.proto = req.proto.clone();
            response.payload = payload_cache;

            let ev = RtspReceiverEvent::Request {
                request: &req,
                response: &mut response,
            };
            let resp = self.callback.on_event(ev).await;

            let took = Instant::now() - start;

            let resp_to_send = match resp.response {
                None => response,
                Some(Ok(resp)) => {
                    warn!("Returning generated response in {took:?} to {req}\n\n{resp}");
                    resp
                }
                Some(Err(err)) => {
                    warn!("Returning {} in {took:?} because of {err} in response to {req}", err.to_code());
                    RtspResponse::new(Some(cseq), err.to_code())
                }
            };

            #[cfg(feature = "tracing")]
            if let Some(tracer) = self.tracer.as_ref() {
                let resp_for_trace = resp_to_send.clone();
                strace!(tracer, "Sending response: \n{}", resp_for_trace);
            }

            warn!("Sending response: \n{resp_to_send}");

            sink.write(vec![resp_to_send.into()])?;

            if let Some(key) = resp.encrypt_after_response {
                debug!("Encrypting connection now");
                sink.codec_mut()
                    .encrypt(AirPlayCipherSaltType::Control, key);
            }

            if resp.disconnect_after_response {
                return Err(RtspError::DisconnectNow)?;
            }
        }

        Ok(())
    }

    async fn on_eof(&mut self, status: Option<RtspError>) {
        warn!("Observed EOF on receiver: {status:?}");
        self.callback
            .on_event(RtspReceiverEvent::Eof(status.unwrap_or(RtspError::Closed)))
            .await;
    }

    async fn on_peer_addr(&mut self, peer_addr: SocketAddr) -> Result<(), Self::Error> {
        debug!("peer_addr = {peer_addr}");
        self.peer_addr.replace(peer_addr);
        self.callback
            .on_event(RtspReceiverEvent::SetPeerIp(peer_addr))
            .await;
        Ok(())
    }

    async fn on_local_addr(&mut self, local_addr: SocketAddr) -> Result<(), Self::Error> {
        debug!("local_addr = {local_addr}");
        self.local_addr.replace(local_addr);
        self.callback
            .on_event(RtspReceiverEvent::SetBindIp(local_addr))
            .await;
        Ok(())
    }

    async fn reconcile(&mut self, _sink: &mut dyn TcpSink<Self>) -> RtspResult<()> {
        self.callback.reconcile().await
    }
}

impl<T: RtspReceiverCallback> EventSleeper for RtspReceiver<T> {
    async fn sleep(&mut self) -> Option<EventToken> {
        self.callback.sleep().await
    }
}

impl<T: RtspReceiverCallback> AsyncShutdown for RtspReceiver<T> {
    async fn shutdown(&mut self) {
        self.callback.shutdown().await;
    }
}
