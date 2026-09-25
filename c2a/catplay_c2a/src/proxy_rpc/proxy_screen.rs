use async_trait::async_trait;
use catplay_carplay::{
    carplay_rx::AirPlayReceiverHandleRef,
    rtsp_frame::{RtspError, RtspResult},
    screen::rx::ScreenReceiverSink,
    video::{AvccConfigExtended, EncodedVideoFrame},
};
use catplay_util::AsyncShutdown;
use catplay_util::{mpsc, oneshot};
use log::warn;
use std::time::Duration;
use tokio::time::timeout;

pub enum ScreenProxyOp {
    IphoneConfig {
        config: AvccConfigExtended,
    },
    IphoneFrame {
        frame: EncodedVideoFrame,
        handle: AirPlayReceiverHandleRef,
        consumed: oneshot::Sender<()>,
    },
    IphoneStreamStart {
        handle: AirPlayReceiverHandleRef,
    },
    IphoneStreamFinish,
}

pub struct ScreenProxy {
    channel: mpsc::UnboundedSender<ScreenProxyOp>,
    handle: AirPlayReceiverHandleRef,
}

impl ScreenProxy {
    const FRAME_CONSUMED_TIMEOUT: Duration = Duration::from_millis(500);

    pub fn new(handle: AirPlayReceiverHandleRef) -> (Self, mpsc::UnboundedReceiver<ScreenProxyOp>) {
        let channel = mpsc::unbounded();
        (Self::with_channel(handle, channel.0), channel.1)
    }

    pub fn with_channel(handle: AirPlayReceiverHandleRef, tx: mpsc::UnboundedSender<ScreenProxyOp>) -> Self {
        let _ = tx.unbounded_send(ScreenProxyOp::IphoneStreamStart { handle: handle.clone() });
        Self { channel: tx, handle }
    }
}

impl Drop for ScreenProxy {
    fn drop(&mut self) {
        let _ = self
            .channel
            .unbounded_send(ScreenProxyOp::IphoneStreamFinish);
    }
}

#[async_trait]
impl ScreenReceiverSink for ScreenProxy {
    async fn init(&mut self) -> RtspResult<()> {
        Ok(())
    }

    async fn set_avcc_config(&mut self, config: AvccConfigExtended) -> RtspResult<()> {
        let _ = self
            .channel
            .unbounded_send(ScreenProxyOp::IphoneConfig { config });
        Ok(())
    }

    async fn process_frame(&mut self, frame: EncodedVideoFrame) -> RtspResult<()> {
        let os = oneshot::channel();
        if self
            .channel
            .unbounded_send(ScreenProxyOp::IphoneFrame {
                frame,
                handle: self.handle.clone(),
                consumed: os.0,
            })
            .is_ok()
        {
            match timeout(Self::FRAME_CONSUMED_TIMEOUT, os.1).await {
                Ok(Ok(())) => {}
                Ok(Err(_)) => {
                    warn!("ScreenProxy frame consumer dropped without ack");
                    return Err(RtspError::Closed);
                }
                Err(_) => {
                    // Prevents worst-case deadlocking if we start shutting down things in unlucky order
                    warn!("ScreenProxy frame consumer timeout after {:?}", Self::FRAME_CONSUMED_TIMEOUT);
                    return Err(RtspError::Timeout);
                }
            }
        }

        Ok(())
    }
}

impl AsyncShutdown for ScreenProxy {}
