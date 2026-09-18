use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use async_trait::async_trait;
use catplay_tokio::{CItem, TcpSession, TcpSink};
use catplay_util::{AsyncShutdown, EventSleeper, EventToken, deadline_after, event_select, notify::Notify};
use log::{debug, info, warn};

use crate::{
    cipher::AirPlayStreamEncryption,
    clock::MediaClockBox,
    rtsp_frame::{RtspError, RtspResult},
    screen::{ScreenFrame, ScreenFrameCodec, tx::screen_tx_proxy::ScreenTransmitProxy},
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
    keep_alive_send_stats_as_body: bool,
    last_keepalive: Instant,
    waiting_for_writable: bool,

    pending_ops: Arc<Mutex<VecDeque<ScreenTransmitOp>>>,
    nal_size_len: usize,

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
            keep_alive_send_stats_as_body: false,
            last_keepalive: Instant::now(),
            waiting_for_writable: false,
            nal_size_len: 4,

            pending_ops,
            notify_frame_added,
            notify_frame_consumed,
        };
        (me, proxy)
    }

    fn close(&mut self) {
        *self.closed.lock().unwrap() = true;
        self.notify_frame_consumed.notify();
    }

    /// Select the screen heartbeat format from the receiving peer's `/info` response.
    /// Existing constructor callers retain the legacy empty opcode-2 format.
    pub fn with_keep_alive_send_stats_as_body(mut self, enabled: bool) -> Self {
        self.keep_alive_send_stats_as_body = enabled;
        self
    }
}

#[async_trait]
impl TcpSession for ScreenTransmitSession {
    type Codec = ScreenFrameCodec;
    type Error = RtspError;

    fn init_codec(&mut self) -> RtspResult<ScreenFrameCodec> {
        info!(
            "Screen keepalive mode: {}",
            if self.keep_alive_send_stats_as_body {
                "opcode 5, empty stats dictionary"
            } else {
                "legacy opcode 2"
            }
        );
        let codec = ScreenFrameCodec::new(self.cipher, self.stream_connection_id, false);
        Ok(codec)
    }

    async fn reconcile(&mut self, sink: &mut dyn TcpSink<Self>) -> RtspResult<()> {
        self.reconcile_at(sink, Instant::now())
    }

    async fn on_eof(&mut self, status: Option<RtspError>) {
        warn!("Observed EOF on screen socket! {status:?}");
        self.close();
    }

    async fn on_msg(&mut self, _sink: &mut dyn TcpSink<Self>, _msg: CItem<Self::Codec>) -> RtspResult<()> {
        Err(RtspError::ProtocolViolationGeneric)
    }
}

impl ScreenTransmitSession {
    fn reconcile_at(&mut self, sink: &mut dyn TcpSink<Self>, now: Instant) -> RtspResult<()> {
        let mut ops = self.pending_ops.lock().unwrap();
        let shutting_down = ops.iter().any(|op| op.is_shutdown());
        self.waiting_for_writable = !sink.is_writable();

        if self.waiting_for_writable && !shutting_down {
            debug!("Skip reconcile, socket unwritable");
            return Ok(());
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
                    sink.write_composite(screen_frame)?;
                    self.notify_frame_consumed.notify();
                }
                _ => {}
            }
        }

        if shutting_down {
            return Err(RtspError::DisconnectNow);
        }

        // Independent of video traffic; recheck backpressure after draining video ops.
        self.waiting_for_writable = !sink.is_writable();
        if !self.keep_alive_interval.is_zero() && now >= self.last_keepalive + self.keep_alive_interval && !self.waiting_for_writable {
            let frame = if self.keep_alive_send_stats_as_body {
                ScreenFrame::keep_alive_with_empty_stats()
            } else {
                ScreenFrame::keep_alive()
            };
            sink.write(frame)?;
            self.last_keepalive = now;
            debug!("Queued screen keepalive (stats body={})", self.keep_alive_send_stats_as_body);
        }

        Ok(())
    }

    fn keep_alive_sleep_delay(&self, now: Instant) -> Duration {
        if self.keep_alive_interval.is_zero() || self.waiting_for_writable {
            // TcpHelperTask waits for write.writable() while composites are queued,
            // then reconciles after flushing. Do not spin on an overdue timer here.
            Duration::MAX
        } else {
            (self.last_keepalive + self.keep_alive_interval).saturating_duration_since(now)
        }
    }
}

impl EventSleeper for ScreenTransmitSession {
    async fn sleep(&mut self) -> Option<EventToken> {
        let keep_alive_deadline = self.keep_alive_sleep_delay(Instant::now());

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
    use crate::{clock::MediaClockSession, screen::ScreenOpCode};

    struct Sink {
        frames: Vec<ScreenFrame>,
        writable: bool,
        fail: bool,
        block_after_write: bool,
    }

    impl Sink {
        fn new() -> Self {
            Self {
                frames: Vec::new(),
                writable: true,
                fail: false,
                block_after_write: false,
            }
        }
    }

    impl TcpSink<ScreenTransmitSession> for Sink {
        fn codec_mut(&mut self) -> &mut ScreenFrameCodec {
            unreachable!("mock queues frames without a codec")
        }

        fn is_writable(&self) -> bool {
            self.writable
        }

        fn write(&mut self, frame: ScreenFrame) -> RtspResult<()> {
            if self.fail {
                return Err(RtspError::ProtocolViolationGeneric);
            }
            self.frames.push(frame);
            if self.block_after_write {
                self.writable = false;
            }
            Ok(())
        }

        fn write_composite(&mut self, frame: ScreenFrame) -> RtspResult<()> {
            self.write(frame)
        }
    }

    fn session() -> (ScreenTransmitSession, ScreenTransmitProxy) {
        ScreenTransmitSession::new(
            AirPlayStreamEncryption::None,
            1,
            Box::new(MediaClockSession::new()),
            ScreenTransmitSession::DEFAULT_MAX_PENDING_FRAMES,
            ScreenTransmitSession::DEFAULT_KEEP_ALIVE,
            Duration::ZERO,
        )
    }

    #[test]
    fn idle_heartbeat_due_and_modes() {
        for stats in [false, true] {
            let (session, _proxy) = session();
            assert!(!session.keep_alive_send_stats_as_body);
            let mut session = session.with_keep_alive_send_stats_as_body(stats);
            let start = session.last_keepalive;
            let due = start + session.keep_alive_interval;
            let mut sink = Sink::new();
            session.reconcile_at(&mut sink, due - Duration::from_nanos(1)).unwrap();
            assert!(sink.frames.is_empty());
            session.reconcile_at(&mut sink, due).unwrap();
            assert_eq!(sink.frames.len(), 1);
            assert_eq!(
                sink.frames[0].header.opcode,
                if stats {
                    ScreenOpCode::KeepAliveWithBody
                } else {
                    ScreenOpCode::KeepAlive
                }
            );
            assert_eq!(session.last_keepalive, due);
            session.reconcile_at(&mut sink, due).unwrap();
            assert_eq!(sink.frames.len(), 1);
            // A late wake queues one heartbeat, not a catch-up burst.
            session.reconcile_at(&mut sink, due + Duration::from_secs(10)).unwrap();
            assert_eq!(sink.frames.len(), 2);
        }
    }

    #[test]
    fn backpressure_and_failed_write_do_not_advance_deadline() {
        let (mut session, _proxy) = session();
        let start = session.last_keepalive;
        let due = start + session.keep_alive_interval;
        let mut sink = Sink::new();
        sink.writable = false;
        session.reconcile_at(&mut sink, due).unwrap();
        assert!(sink.frames.is_empty());
        assert_eq!(session.last_keepalive, start);
        sink.writable = true;
        sink.fail = true;
        assert!(session.reconcile_at(&mut sink, due).is_err());
        assert_eq!(session.last_keepalive, start);
        sink.fail = false;
        session.reconcile_at(&mut sink, due).unwrap();
        assert_eq!(sink.frames.len(), 1);
        assert_eq!(session.last_keepalive, due);
    }

    #[test]
    fn blocked_timer_stays_suppressed_until_writable_reconcile() {
        let (mut session, _proxy) = session();
        let start = session.last_keepalive;
        let interval = session.keep_alive_interval;
        let due = start + interval;
        let later = due + Duration::from_secs(60);
        let mut sink = Sink::new();
        assert_eq!(session.keep_alive_sleep_delay(start), interval);
        assert_eq!(session.keep_alive_sleep_delay(due), Duration::ZERO);
        sink.writable = false;
        session.reconcile_at(&mut sink, due).unwrap();
        assert_eq!(session.keep_alive_sleep_delay(due), Duration::MAX);
        assert_eq!(session.keep_alive_sleep_delay(later), Duration::MAX);
        // A notification while still blocked must not re-arm the overdue timer.
        session.reconcile_at(&mut sink, later).unwrap();
        assert_eq!(session.keep_alive_sleep_delay(later), Duration::MAX);
        assert_eq!(session.last_keepalive, start);
        assert!(sink.frames.is_empty());
        sink.writable = true;
        session.reconcile_at(&mut sink, later).unwrap();
        assert!(!session.waiting_for_writable);
        assert_eq!(session.last_keepalive, later);
        assert_eq!(session.keep_alive_sleep_delay(later), interval);
        session.reconcile_at(&mut sink, later).unwrap();
        assert_eq!(sink.frames.len(), 1);
    }

    #[test]
    fn disabled_timer_stays_disabled_when_writability_returns() {
        let (mut session, _proxy) = session();
        let start = session.last_keepalive;
        let interval = session.keep_alive_interval;
        let due = start + interval;
        let mut sink = Sink::new();
        session.keep_alive_interval = Duration::ZERO;
        assert_eq!(session.keep_alive_sleep_delay(start), Duration::MAX);
        sink.writable = false;
        session.reconcile_at(&mut sink, due).unwrap();
        assert_eq!(session.keep_alive_sleep_delay(due), Duration::MAX);
        sink.writable = true;
        session.reconcile_at(&mut sink, due).unwrap();
        assert!(!session.waiting_for_writable);
        assert_eq!(session.keep_alive_sleep_delay(due), Duration::MAX);
        assert_eq!(session.last_keepalive, start);
        assert!(sink.frames.is_empty());
        session.keep_alive_interval = interval;
        assert_eq!(session.keep_alive_sleep_delay(due), Duration::ZERO);
        session.reconcile_at(&mut sink, due).unwrap();
        assert_eq!(sink.frames.len(), 1);
        assert_eq!(session.keep_alive_sleep_delay(due), interval);
    }

    #[test]
    fn shutdown_pending_suppresses_due_heartbeat_even_when_blocked() {
        for writable in [false, true] {
            let (mut session, _proxy) = session();
            session.pending_ops.lock().unwrap().push_back(ScreenTransmitOp::TransmitterDropped);
            let start = session.last_keepalive;
            let mut sink = Sink::new();
            sink.writable = writable;
            assert!(matches!(
                session.reconcile_at(&mut sink, start + session.keep_alive_interval),
                Err(RtspError::DisconnectNow)
            ));
            assert!(sink.frames.is_empty());
            assert_eq!(session.last_keepalive, start);
        }
    }

    #[test]
    fn disabled_interval_and_drop_semantics() {
        let (mut session, proxy) = session();
        session.keep_alive_interval = Duration::ZERO;
        let mut sink = Sink::new();
        session.reconcile_at(&mut sink, session.last_keepalive + Duration::from_secs(60)).unwrap();
        assert!(sink.frames.is_empty());
        // Proxy drop currently only notifies; preserve that existing behavior.
        drop(proxy);
        assert!(session.pending_ops.lock().unwrap().is_empty());
        let closed = session.closed.clone();
        drop(session);
        assert!(*closed.lock().unwrap());
    }

    #[test]
    fn recheck_backpressure_after_media_queue_write() {
        let (mut session, _proxy) = session();
        session.pending_ops.lock().unwrap().push_back(ScreenTransmitOp::Configure(AvccConfigExtended {
            hevc: false,
            avcc: crate::video::AvccConfig {
                nal_size_len: 4,
                sps_pps: Vec::new(),
            },
            video_latency: Duration::ZERO,
            width: 800,
            height: 480,
            respect_timestamps: false,
        }));
        let start = session.last_keepalive;
        let mut sink = Sink::new();
        sink.block_after_write = true;
        session.reconcile_at(&mut sink, start + session.keep_alive_interval).unwrap();
        assert_eq!(sink.frames.len(), 1);
        assert_eq!(sink.frames[0].header.opcode, ScreenOpCode::VideoConfig);
        assert_eq!(session.last_keepalive, start);
        assert_eq!(session.keep_alive_sleep_delay(start + session.keep_alive_interval), Duration::MAX);
        sink.writable = true;
        sink.block_after_write = false;
        let resumed = start + Duration::from_secs(10);
        session.reconcile_at(&mut sink, resumed).unwrap();
        assert_eq!(sink.frames.len(), 2);
        assert_eq!(sink.frames[1].header.opcode, ScreenOpCode::KeepAlive);
        assert_eq!(session.last_keepalive, resumed);
        assert_eq!(session.keep_alive_sleep_delay(resumed), session.keep_alive_interval);
    }
}
