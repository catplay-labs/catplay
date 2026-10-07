use std::pin::Pin;
use std::sync::Arc;

use async_trait::async_trait;
use log::debug;

use crate::{CsmClientHandleRef, CsmFileTransferEvent, CsmSessionResult};
use catplay_csm::decoder::CsmPacketBox;
use catplay_lingo::{CommandKey, lingos::LingoMessage};

/// Represents an iAP2 CSM session with user logic, which can:
/// - read CSM messages and return messages to be written as a response
/// - check session direction(server/client) and writability status using the client handle
/// - write CSM messages in background, outside of `start` and `respond` callbacks
/// - use a custom `Drop` handler to finalize itself (in some cases, the client along with session object is forcefully terminated - especially with USB transport)
#[async_trait]
#[cfg_attr(test, mockall::automock)]
pub trait CsmSession: Send + Sync + 'static {
    /// Called **only** after handshake and link negotiation, once.
    ///
    /// An error from this method will be treated as a signal to terminate the session.
    async fn start(&mut self, handle: CsmClientHandleRef) -> CsmSessionResult<()>;

    /// Called on new CSM message to prepare a response.
    ///
    /// For convenience, the messages can be added to `output` instead of using the client handle to send them.
    ///
    /// An error from this method will be treated as a signal to terminate the session.
    async fn respond(&mut self, packet: CsmPacketBox, handle: CsmClientHandleRef) -> CsmSessionResult<()>;

    /// Called with a decoded iAP1 message in downgrade mode.
    async fn on_legacy_message(
        &mut self,
        _message: LingoMessage,
        _transaction_id: Option<u16>,
        _pending_command: Option<CommandKey>,
        _handle: CsmClientHandleRef,
    ) -> CsmSessionResult<()> {
        Ok(())
    }

    async fn on_file_event(&mut self, id: u8, ev: CsmFileTransferEvent, _handle: CsmClientHandleRef) -> CsmSessionResult<()> {
        debug!("on_file_event STUB {id:?} {ev:?}");
        Ok(())
    }
}

#[async_trait]
impl CsmSession for Box<dyn CsmSession> {
    async fn start(&mut self, handle: CsmClientHandleRef) -> CsmSessionResult<()> {
        self.as_mut().start(handle).await
    }

    async fn respond(&mut self, packet: CsmPacketBox, handle: CsmClientHandleRef) -> CsmSessionResult<()> {
        self.as_mut().respond(packet, handle).await
    }

    async fn on_legacy_message(
        &mut self,
        message: LingoMessage,
        transaction_id: Option<u16>,
        pending_command: Option<CommandKey>,
        handle: CsmClientHandleRef,
    ) -> CsmSessionResult<()> {
        self.as_mut()
            .on_legacy_message(message, transaction_id, pending_command, handle)
            .await
    }

    async fn on_file_event(&mut self, id: u8, ev: CsmFileTransferEvent, handle: CsmClientHandleRef) -> CsmSessionResult<()> {
        self.as_mut().on_file_event(id, ev, handle).await
    }
}

/// Initializer for new CSM sessions.
pub type CsmSessionCallback = Arc<dyn Fn() -> CsmSessionBox + Send + Sync>;
pub type CsmSessionBox = Box<dyn CsmSession>;

type StartCallback = Arc<dyn Fn(CsmClientHandleRef) -> Pin<Box<dyn Future<Output = CsmSessionResult<()>> + Send>> + Send + Sync>;
type RespondCallback =
    Arc<dyn Fn(CsmPacketBox, CsmClientHandleRef) -> Pin<Box<dyn Future<Output = CsmSessionResult<()>> + Send>> + Send + Sync>;

pub struct CsmSessionCallbacks {
    start_cb: StartCallback,
    respond_cb: RespondCallback,
}

impl CsmSessionCallbacks {
    pub fn new<F1, Fut1, F2, Fut2>(start_cb: F1, respond_cb: F2) -> Self
    where
        F1: Fn(CsmClientHandleRef) -> Fut1 + Send + Sync + 'static,
        F2: Fn(CsmPacketBox, CsmClientHandleRef) -> Fut2 + Send + Sync + 'static,
        Fut1: Future<Output = CsmSessionResult<()>> + Send + 'static,
        Fut2: Future<Output = CsmSessionResult<()>> + Send + 'static,
    {
        Self {
            start_cb: Arc::new(move |h| Box::pin(start_cb(h))),
            respond_cb: Arc::new(move |p, h| Box::pin(respond_cb(p, h))),
        }
    }

    pub fn noop() -> Self {
        Self::new(async |_a| Ok(()), async |_a, _b| Ok(()))
    }
}

#[async_trait]
impl CsmSession for CsmSessionCallbacks {
    async fn start(&mut self, handle: CsmClientHandleRef) -> CsmSessionResult<()> {
        (self.start_cb)(handle).await
    }

    async fn respond(&mut self, packet: CsmPacketBox, handle: CsmClientHandleRef) -> CsmSessionResult<()> {
        (self.respond_cb)(packet, handle).await
    }

    async fn on_file_event(&mut self, _id: u8, _ev: CsmFileTransferEvent, _handle: CsmClientHandleRef) -> CsmSessionResult<()> {
        // TODO: ignored
        Ok(())
    }
}
