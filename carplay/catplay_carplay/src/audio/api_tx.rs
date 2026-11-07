use std::{
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use catplay_util::notify::Notify;

use crate::{
    audio::{AudioStreamBasicDescription, RingProducer, Zeroable},
    rtsp_frame::RtspResult,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioRecorderEvent {
    /// The transport is ready. Transmission still requires the user's start().
    Start,
    Pause,
    Stop,
    /// The encoder advanced its read head, freeing ring capacity. This includes
    /// skipped gaps and failed encodes, and does not acknowledge UDP delivery.
    DrainedDataFromRing,
}

#[derive(Default)]
struct RecorderState {
    running: bool,
    paused_at: Option<Instant>,
    paused_total: Duration,
}

/// Controls encoding/transmission independently of writing to the PCM ring.
/// Initially paused, so the ring can be filled before start().
#[derive(Clone, Default)]
pub struct AudioRecorderControl {
    state: Arc<Mutex<RecorderState>>,
    pub(crate) notify: Notify,
}

impl AudioRecorderControl {
    pub fn start(&self) {
        let mut state = self.state.lock().unwrap();
        if let Some(paused_at) = state.paused_at.take() {
            state.paused_total += paused_at.elapsed();
        }
        state.running = true;
        drop(state);
        self.notify.notify();
    }

    pub fn pause(&self) {
        let mut state = self.state.lock().unwrap();
        if state.running {
            state.paused_at = Some(Instant::now());
            state.running = false;
        }
        drop(state);
        self.notify.notify();
    }

    pub fn is_running(&self) -> bool {
        self.state.lock().unwrap().running
    }

    /// Call after committing data or gaps directly to the producer ring.
    pub fn added_data_to_ring(&self) {
        self.notify.notify();
    }

    pub(crate) fn playback_state(&self) -> (bool, Duration) {
        let state = self.state.lock().unwrap();
        (state.running, state.paused_total)
    }
}

/// Receives the PCM producer directly; the user chooses when RTP starts.
pub trait AudioRecorder: Send + 'static {
    type Sample: Zeroable;

    fn init(
        &mut self,
        ring: RingProducer<Self::Sample>,
        format: AudioStreamBasicDescription,
        control: AudioRecorderControl,
    ) -> RtspResult<()>;

    fn on_event(&mut self, event: AudioRecorderEvent);
}

pub type AudioRecorderBox<S> = Box<dyn AudioRecorder<Sample = S>>;
