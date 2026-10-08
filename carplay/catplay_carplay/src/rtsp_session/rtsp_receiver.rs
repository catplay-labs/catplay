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

/// The iPhone sends requests from more than one queue over the same connection, so a request can
/// arrive just after one with a higher CSeq (an iAP2 `POST /command` overtaken by `SETUP`). Such
/// small reordering is accepted; a repeated or much older CSeq still fails the session.
const CSEQ_REORDER_WINDOW: u32 = 8;

/// Returns the newest CSeq seen so far.
fn check_cseq(newest: Option<u32>, cseq: u32) -> RtspResult<u32> {
    match newest {
        None => Ok(cseq),
        Some(newest) if cseq > newest => Ok(cseq),
        Some(newest) if cseq < newest && newest - cseq <= CSEQ_REORDER_WINDOW => Ok(newest),
        Some(newest) => Err(RtspError::CSeqSanity(cseq, newest)),
    }
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

            if Self::ENFORCE_CSEQ_SANITY {
                let newest = check_cseq(self.last_cseq, cseq)?;
                if newest != cseq {
                    warn!("Request CSeq {cseq} arrived after CSeq {newest}, accepting the reordering");
                }
                self.last_cseq.replace(newest);
            } else {
                self.last_cseq.replace(cseq);
            }

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_request_sets_the_newest_cseq() {
        assert_eq!(check_cseq(None, 1).unwrap(), 1);
    }

    #[test]
    fn increasing_cseq_is_accepted() {
        assert_eq!(check_cseq(Some(16), 18).unwrap(), 18);
    }

    #[test]
    fn request_overtaken_by_a_newer_one_is_accepted() {
        // iPhone with iAP2 over CarPlay: POST /command CSeq 17 arrives after SETUP CSeq 18
        assert_eq!(check_cseq(Some(18), 17).unwrap(), 18);
    }

    #[test]
    fn repeated_newest_cseq_is_rejected() {
        assert!(matches!(check_cseq(Some(18), 18), Err(RtspError::CSeqSanity(18, 18))));
    }

    #[test]
    fn cseq_far_behind_the_newest_is_rejected() {
        assert!(matches!(check_cseq(Some(100), 5), Err(RtspError::CSeqSanity(5, 100))));
    }
}
