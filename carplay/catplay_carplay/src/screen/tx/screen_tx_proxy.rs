use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
    time::Duration,
};

use catplay_util::{event_select, notify::Notify};
use log::debug;

use crate::{
    screen::tx::{
        screen_tx_session::ScreenTransmitOp,
        screen_tx_sink::{ScreenTransmitError, ScreenTransmitSink},
    },
    video::{AvccConfigExtended, EncodedVideoFrame, Pts},
};
pub struct ScreenTransmitProxy {
    stream_latency: Duration,

    queue: Arc<Mutex<VecDeque<ScreenTransmitOp>>>,
    closed: Arc<Mutex<bool>>,
    notify_frame_added: Notify,
    notify_frame_consumed: Notify,
    max_queued_frames: usize,
    last_config: Option<AvccConfigExtended>,

    needs_keyframe: bool,
}

impl ScreenTransmitProxy {
    pub fn new(
        stream_latency: Duration,
        queue: Arc<Mutex<VecDeque<ScreenTransmitOp>>>,
        closed: Arc<Mutex<bool>>,
        notify_frame_added: Notify,
        notify_frame_consumed: Notify,
        max_queue: usize,
    ) -> Self {
        Self {
            stream_latency,
            queue,
            closed,
            notify_frame_added,
            notify_frame_consumed,
            max_queued_frames: max_queue,
            last_config: None,
            needs_keyframe: true,
        }
    }
}

impl ScreenTransmitSink for ScreenTransmitProxy {
    fn stream_latency(&self) -> Duration {
        self.stream_latency
    }

    fn queue_len(&self) -> usize {
        let ops = self.queue.lock().unwrap();
        ops.iter().filter(|op| op.is_frame()).count()
    }

    fn queue_max(&self) -> usize {
        self.max_queued_frames
    }

    fn is_full(&self) -> bool {
        self.queue_len() > self.queue_max()
    }

    fn is_closed(&self) -> bool {
        *self.closed.lock().unwrap()
    }

    fn push_avcc_config(&mut self, config: AvccConfigExtended, pts: Pts) -> Result<(), ScreenTransmitError> {
        let closed = self.closed.lock().unwrap();
        if *closed {
            return Err(ScreenTransmitError::Closed);
        }

        // One config frame per stream, as an iPhone sends: it repeats neither the config nor the
        // opening marker for later keyframes.
        if let Some(last_config) = self.last_config.as_ref()
            && last_config == &config
        {
            return Ok(());
        }

        let mut ops = self.queue.lock().unwrap();
        ops.push_back(ScreenTransmitOp::Configure(config.clone(), pts));
        self.last_config.replace(config);
        self.notify_frame_added.notify();

        Ok(())
    }

    fn needs_keyframe(&self) -> bool {
        self.needs_keyframe
    }

    async fn push_frame(&mut self, frame: EncodedVideoFrame) -> Result<(), (ScreenTransmitError, EncodedVideoFrame)> {
        if *self.closed.lock().unwrap() {
            return Err((ScreenTransmitError::Closed, frame));
        }

        if let Some(config) = frame.config.clone()
            && let Err(err) = self.push_avcc_config(config, frame.pts)
        {
            return Err((err, frame));
        }

        if self.last_config.is_none() {
            return Err((ScreenTransmitError::NotConfigured, frame));
        }

        if self.needs_keyframe && !frame.is_known_keyframe() {
            return Err((ScreenTransmitError::NeedsKeyframe, frame));
        }

        loop {
            let overflow = {
                let ops = self.queue.lock().unwrap();
                let queued = ops.iter().filter(|op| op.is_frame()).count();
                queued > self.max_queued_frames
            };

            if *self.closed.lock().unwrap() {
                return Err((ScreenTransmitError::Closed, frame));
            }

            if overflow {
                // return Err(ScreenTransmitError::QueueOverflow);
                event_select!(self.notify_frame_consumed);
            } else {
                let mut ops = self.queue.lock().unwrap();
                ops.push_back(ScreenTransmitOp::Frame(frame));
                self.notify_frame_added.notify();
                self.needs_keyframe = false;
                break;
            }
        }

        Ok(())
    }
}

impl Drop for ScreenTransmitProxy {
    fn drop(&mut self) {
        debug!("Screen transmitter was dropped!");
        // self.queue.lock().unwrap().push_back(ScreenTransmitOp::TransmitterDropped);
        self.notify_frame_added.notify();
    }
}

#[cfg(test)]
mod tests {
    use std::time::Instant;

    use super::*;
    use crate::video::AvccConfig;

    fn config() -> AvccConfigExtended {
        AvccConfigExtended {
            hevc: true,
            avcc: AvccConfig {
                nal_size_len: 4,
                sps_pps: vec![0, 0, 0, 1, 0x40, 0x01],
            },
            video_latency: Duration::ZERO,
            width: 800,
            height: 1280,
            respect_timestamps: true,
            view: None,
        }
    }

    /// A phone sends one config frame per stream; ours must not repeat it for every keyframe.
    #[test]
    fn an_unchanged_config_is_sent_once() {
        let queue = Arc::new(Mutex::new(VecDeque::new()));
        let mut proxy = ScreenTransmitProxy::new(
            Duration::ZERO,
            queue.clone(),
            Arc::new(Mutex::new(false)),
            Notify::new(),
            Notify::new(),
            8,
        );

        let pts = Pts(Instant::now());
        proxy.push_avcc_config(config(), pts).unwrap();
        proxy.push_avcc_config(config(), pts).unwrap();

        let queued = queue.lock().unwrap().iter().filter(|op| op.is_configure()).count();
        assert_eq!(queued, 1);
    }
}
