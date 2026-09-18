use std::time::{Duration, Instant};

use async_trait::async_trait;
use catplay_tokio::{CItem, TcpSession, TcpSink};
use catplay_util::{AsyncShutdown, EventSleeper};
use log::{debug, trace, warn};

use crate::{
    cipher::AirPlayStreamEncryption,
    clock::MediaClockBox,
    rtsp_frame::{RtspError, RtspResult},
    screen::{ScreenFlag, ScreenFrameCodec, ScreenOpCode, rx::screen_rx_sink::ScreenReceiverSinkBox},
    video::AvccConfigExtended,
};

pub struct ScreenReceiverSession {
    cipher: AirPlayStreamEncryption,
    stream_connection_id: u64,

    clock: MediaClockBox,
    sink: ScreenReceiverSinkBox,
    video_latency: Duration,
    config: Option<AvccConfigExtended>,
    prev_frame: Option<Instant>,
    start: Instant,
    negative_ahead_frames: u32,
    late_frames: u32,
    display_delta_ms: i64,
}

impl ScreenReceiverSession {
    pub fn new(
        cipher: AirPlayStreamEncryption,
        stream_connection_id: u64,
        video_latency: Duration,
        sink: ScreenReceiverSinkBox,
        clock: MediaClockBox,
    ) -> Self {
        Self {
            cipher,
            stream_connection_id,
            config: None,
            video_latency,
            sink,
            clock,
            prev_frame: None,
            start: Instant::now(),
            negative_ahead_frames: 0,
            late_frames: 0,
            display_delta_ms: 0,
        }
    }
}

#[async_trait]
impl TcpSession for ScreenReceiverSession {
    type Codec = ScreenFrameCodec;
    type Error = RtspError;

    fn init_codec(&mut self) -> RtspResult<ScreenFrameCodec> {
        let codec = ScreenFrameCodec::new(self.cipher, self.stream_connection_id, true);
        Ok(codec)
    }

    async fn on_msg(&mut self, _sink: &mut dyn TcpSink<Self>, msg: CItem<Self::Codec>) -> RtspResult<()> {
        debug!("Received screen frame: {:?} {:?}", msg.header, msg.flags());

        match msg.header.opcode {
            ScreenOpCode::VideoFrame => {
                let Some(config) = self.config.as_ref() else {
                    return Err(RtspError::ProtocolViolation("received VideoFrame before VideoConfig"));
                };

                let _respect_timestamps = config.respect_timestamps;

                if !self.clock.is_synchronized() {
                    warn!("Received video frame before media clock was synchronized! Using pts = Instant::now()");
                }

                let frame = msg
                    .video_decode(config, self.clock.as_ref())
                    .map_err(|e| RtspError::ProtocolViolationString(format!("failed to scan NAL offsets in VideoFrame: {e:?}")))?
                    .ok_or(RtspError::Empty)?;

                let pts_decoded = frame.pts.decode();
                let display_ticks = frame.pts.0;
                let now = Instant::now();
                let video_latency_ms = self.video_latency.as_millis() as i64;

                let display_vs_now_ms = if display_ticks >= now {
                    display_ticks.saturating_duration_since(now).as_millis() as i64
                } else {
                    self.negative_ahead_frames = self.negative_ahead_frames.saturating_add(1);
                    -(now.saturating_duration_since(display_ticks).as_millis() as i64)
                };
                self.display_delta_ms = video_latency_ms - display_vs_now_ms;

                if self.display_delta_ms >= (2 * video_latency_ms) {
                    self.late_frames = self.late_frames.saturating_add(1);

                    warn!(
                        "Late frame ({} ms, {} total late frames, negativeAheadFrames={})",
                        self.display_delta_ms, self.late_frames, self.negative_ahead_frames
                    );
                }

                let recv_delta = match self.prev_frame {
                    None => {
                        let first_frame_took = now.saturating_duration_since(self.start);
                        warn!("Received first video frame after {first_frame_took:?}");
                        None
                    }
                    Some(v) => Some(now.saturating_duration_since(v)),
                };

                debug!("Decoded video frame pts={pts_decoded} recv_delta={recv_delta:?}");

                if frame.is_known_keyframe() {
                    warn!("Received keyframe");
                }

                self.prev_frame.replace(now);
                self.sink.process_frame(frame).await?;
            }

            ScreenOpCode::VideoConfig => {
                if msg.flags().contains(ScreenFlag::ClearScreen) {
                    warn!("Ignoring ScreenFlag::ClearScreen");
                    return Ok(());
                }
                let config = msg.config_decode(self.video_latency);
                let Some(config) = config else {
                    return Err(RtspError::ProtocolViolation("received unparsable VideoConfig frame"));
                };

                debug!("Received VideoConfig: {config:?}");

                self.sink.set_avcc_config(config.clone()).await?;
                self.config.replace(config);
            }

            ScreenOpCode::KeepAlive | ScreenOpCode::KeepAliveWithBody => {
                trace!("Received screen keep alive: opcode {:?}", msg.header.opcode);

                if let Some(stats) = msg.sender_stats_decode() {
                    debug!("Received screen sender stats: {stats:?}");
                }
            }
            _ => {
                debug!("Received unknown video frame: {:?}", msg.header);
            }
        }
        Ok(())
    }

    async fn on_eof(&mut self, status: Option<RtspError>) {
        warn!("Observed EOF on screen socket! {status:?}");
    }
}

impl EventSleeper for ScreenReceiverSession {}

impl AsyncShutdown for ScreenReceiverSession {
    async fn shutdown(&mut self) {
        self.sink.shutdown_pinned().await;
    }
}
