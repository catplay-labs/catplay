use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use async_trait::async_trait;
use catplay_tokio::{CItem, TcpSession, TcpSink};
use catplay_util::{AsyncShutdown, EventSleeper, EventToken, deadline_after, event_select, notify::Notify};
use log::{debug, warn};

use crate::{
    cipher::AirPlayStreamEncryption,
    clock::MediaClockBox,
    rtsp_frame::{RtspError, RtspResult},
    screen::{ScreenFrame, ScreenFrameCodec, ScreenOpCode, ScreenSenderStats, tx::screen_tx_proxy::ScreenTransmitProxy},
    video::{AvccConfigExtended, EncodedVideoFrame},
};

pub enum ScreenTransmitOp {
    Configure(AvccConfigExtended),
    Frame(EncodedVideoFrame),
    TransmitterDropped,
}

impl ScreenTransmitOp {
    pub fn is_frame(&self) -> bool {
        matches!(self, ScreenTransmitOp::Frame(_))
    }

    pub fn is_shutdown(&self) -> bool {
        matches!(self, ScreenTransmitOp::TransmitterDropped)
    }

    pub fn is_configure(&self) -> bool {
        matches!(self, ScreenTransmitOp::Configure(_))
    }
}

pub struct ScreenTransmitSession {
    closed: Arc<Mutex<bool>>,

    clock: MediaClockBox,
    cipher: AirPlayStreamEncryption,
    stream_connection_id: u64,

    keep_alive_interval: Duration,
    last_keepalive: Instant,
    keep_alive_send_stats_as_body: bool,

    pending_ops: Arc<Mutex<VecDeque<ScreenTransmitOp>>>,
    nal_size_len: usize,
    frames_since_keepalive: u32,
    bytes_since_keepalive: u64,
    queued_frames_sum: u64,
    queued_frames_samples: u64,

    notify_frame_added: Notify,
    notify_frame_consumed: Notify,
}

impl ScreenTransmitSession {
    pub const DEFAULT_MAX_PENDING_FRAMES: usize = 8;
    pub const DEFAULT_KEEP_ALIVE: Duration = Duration::from_millis(1000);

    pub fn new(
        cipher: AirPlayStreamEncryption,
        stream_connection_id: u64,
        clock: MediaClockBox,
        max_pending_frames: usize,
        keep_alive_interval: Duration,
        keep_alive_send_stats_as_body: bool,
        stream_latency: Duration,
    ) -> (Self, ScreenTransmitProxy) {
        let pending_ops = Arc::new(Mutex::new(VecDeque::with_capacity(max_pending_frames * 4)));
        let closed = Arc::new(Mutex::new(false));
        let notify_frame_added = Notify::new();
        let notify_frame_consumed = Notify::new();

        let proxy = ScreenTransmitProxy::new(
            stream_latency,
            pending_ops.clone(),
            closed.clone(),
            notify_frame_added.clone(),
            notify_frame_consumed.clone(),
            max_pending_frames,
        );

        let me = Self {
            closed,

            clock,
            cipher,
            stream_connection_id,

            keep_alive_interval,
            last_keepalive: Instant::now(),
            keep_alive_send_stats_as_body,
            nal_size_len: 4,
            frames_since_keepalive: 0,
            bytes_since_keepalive: 0,
            queued_frames_sum: 0,
            queued_frames_samples: 0,

            pending_ops,
            notify_frame_added,
            notify_frame_consumed,
        };
        (me, proxy)
    }

    fn create_keep_alive(&mut self, now: Instant) -> ScreenFrame {
        let elapsed = now
            .duration_since(self.last_keepalive)
            .as_secs_f64()
            .max(f64::MIN_POSITIVE);
        self.last_keepalive = now;

        if !self.keep_alive_send_stats_as_body {
            return ScreenFrame::keep_alive();
        }

        let frames = std::mem::take(&mut self.frames_since_keepalive);
        let stats = ScreenSenderStats {
            tx_usage_avg: std::mem::take(&mut self.bytes_since_keepalive) as f64 / elapsed,
            encoder_current_fps: (frames as f64 / elapsed).round() as u32,
            sent_frames_avg: frames,
            queued_frames_avg: (std::mem::take(&mut self.queued_frames_sum) / std::mem::take(&mut self.queued_frames_samples).max(1))
                as u32,
            loss_avg: 0.0,
        };
        let frame = ScreenFrame::keep_alive_with_stats(&stats);
        if frame.header.opcode == ScreenOpCode::KeepAliveWithBody {
            debug!("Sending screen keep alive with stats: {frame:?}");
        }
        frame
    }

    fn close(&mut self) {
        *self.closed.lock().unwrap() = true;
        self.notify_frame_consumed.notify();
    }
}

#[async_trait]
impl TcpSession for ScreenTransmitSession {
    type Codec = ScreenFrameCodec;
    type Error = RtspError;

    fn init_codec(&mut self) -> RtspResult<ScreenFrameCodec> {
        let codec = ScreenFrameCodec::new(self.cipher, self.stream_connection_id, false);
        Ok(codec)
    }

    async fn reconcile(&mut self, sink: &mut dyn TcpSink<Self>) -> RtspResult<()> {
        let mut ops = self.pending_ops.lock().unwrap();
        let shutting_down = ops.iter().any(|op| op.is_shutdown());
        let writable = sink.is_writable() || shutting_down;

        if !writable {
            debug!("Skip reconcile, socket unwritable");
            return Ok(());
        }

        if self.keep_alive_send_stats_as_body {
            self.queued_frames_sum += ops.iter().filter(|op| op.is_frame()).count() as u64;
            self.queued_frames_samples += 1;
        }

        while let Some(elem) = ops.pop_front() {
            match elem {
                ScreenTransmitOp::Configure(config) => {
                    debug!("Writing AVCC config now");
                    self.nal_size_len = config.avcc.nal_size_len;
                    sink.write(ScreenFrame::config(&config))?;
                }
                ScreenTransmitOp::Frame(frame) => {
                    warn!("Flushing video frame pts={:?} keyframe={}", frame.pts, frame.is_known_keyframe());
                    let screen_frame = ScreenFrame::video_proxied(frame, self.nal_size_len, self.clock.as_ref())
                        .map_err(|e| RtspError::UnexpectedState(format!("unexpected failure during NAL serialization: {e:?}")))?;
                    if self.keep_alive_send_stats_as_body {
                        self.frames_since_keepalive += 1;
                        self.bytes_since_keepalive += screen_frame.data.len() as u64;
                    }
                    sink.write_composite(screen_frame)?;
                    self.notify_frame_consumed.notify();
                }
                _ => {}
            }
        }
        drop(ops);

        // Send periodic keep-alives (if outside low-power mode)
        let now = Instant::now();
        if !self.keep_alive_interval.is_zero() && now > self.last_keepalive + self.keep_alive_interval {
            sink.write(self.create_keep_alive(now))?;
        }

        if shutting_down {
            return Err(RtspError::DisconnectNow);
        }

        Ok(())
    }

    async fn on_eof(&mut self, status: Option<RtspError>) {
        warn!("Observed EOF on screen socket! {status:?}");
        self.close();
    }

    async fn on_msg(&mut self, _sink: &mut dyn TcpSink<Self>, _msg: CItem<Self::Codec>) -> RtspResult<()> {
        Err(RtspError::ProtocolViolationGeneric)
    }
}

impl EventSleeper for ScreenTransmitSession {
    async fn sleep(&mut self) -> Option<EventToken> {
        let keep_alive_deadline = if self.keep_alive_interval.is_zero() {
            Duration::MAX
        } else {
            (self.last_keepalive + self.keep_alive_interval).saturating_duration_since(Instant::now())
        };

        event_select!(self.notify_frame_added, deadline_after(keep_alive_deadline))
    }
}

impl AsyncShutdown for ScreenTransmitSession {
    async fn shutdown(&mut self) {}
}

impl Drop for ScreenTransmitSession {
    fn drop(&mut self) {
        self.close();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::clock::MediaClockSession;

    #[test]
    fn keep_alive_body_requires_receiver_support() {
        for supported in [false, true] {
            let (mut session, _proxy) = ScreenTransmitSession::new(
                AirPlayStreamEncryption::None,
                1,
                Box::new(MediaClockSession::new()),
                8,
                Duration::from_secs(1),
                supported,
                Duration::ZERO,
            );
            session.frames_since_keepalive = 3;
            session.bytes_since_keepalive = 300;
            session.queued_frames_sum = 6;
            session.queued_frames_samples = 2;

            let frame = session.create_keep_alive(session.last_keepalive + Duration::from_secs(1));
            if supported {
                assert_eq!(frame.header.opcode, ScreenOpCode::KeepAliveWithBody);
                let stats = frame.sender_stats_decode().unwrap();
                assert_eq!(stats.sent_frames_avg, 3);
                assert_eq!(stats.queued_frames_avg, 3);
                assert_eq!(stats.tx_usage_avg, 300.0);
            } else {
                assert_eq!(frame.header.opcode, ScreenOpCode::KeepAlive);
                assert!(frame.data.is_empty());
            }
        }
    }
}
