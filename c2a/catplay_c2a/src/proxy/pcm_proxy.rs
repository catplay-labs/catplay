use std::sync::{Arc, Mutex};

use catplay_carplay::{
    audio::{
        AudioPlayer, AudioPlayerEvent, AudioRecorder, AudioRecorderControl, AudioRecorderEvent, AudioStreamBasicDescription, RingConsumer,
        RingProducer,
    },
    carplay_tx::TeardownGuard,
    msg::StreamType,
    rtsp_frame::{RtspError, RtspResult},
};
use tokio::{sync::Notify, task::JoinHandle};

#[derive(Default)]
struct ProxyState {
    source: Option<RingConsumer<i16>>,
    destination: Option<RingProducer<i16>>,
    control: Option<AudioRecorderControl>,
    playing: bool,
    recorder_ready: bool,
    stopped: bool,
}

#[derive(Default)]
struct ProxyShared {
    state: Mutex<ProxyState>,
    wake: Notify,
}

impl ProxyShared {
    fn stop(&self) {
        let mut state = self.state.lock().unwrap();
        state.stopped = true;
        if let Some(control) = state.control.as_ref() {
            control.pause();
        }
        state.source = None;
        state.destination = None;
        drop(state);
        self.wake.notify_one();
    }

    /// No user callbacks run under this lock. The worker exclusively accesses
    /// both ring endpoints; lifecycle callbacks only update state and wake it.
    fn synchronize(&self) -> bool {
        let mut state = self.state.lock().unwrap();
        if state.stopped {
            return false;
        }
        let ProxyState {
            source,
            destination,
            control,
            playing,
            recorder_ready,
            ..
        } = &mut *state;
        if let (Some(source), Some(destination), Some(control)) = (source, destination, control) {
            if *playing && *recorder_ready {
                let transferred = destination.transfer_from(source);
                if transferred > 0 {
                    control.added_data_to_ring();
                }
                if !control.is_running() {
                    control.start();
                }
            }
        }
        true
    }
}

/// Event-driven PCM connection between the player's consumer and the recorder's
/// producer. RTP's recorder owns transmission pacing and backpressure.
pub struct PcmProxyPlayer {
    shared: Arc<ProxyShared>,
    task: JoinHandle<()>,
    pub guard: Arc<Mutex<Option<TeardownGuard<StreamType>>>>,
}

impl PcmProxyPlayer {
    pub fn pair() -> (PcmProxyPlayer, PcmProxyRecorder) {
        let shared = Arc::new(ProxyShared::default());
        let worker = shared.clone();
        let task = tokio::spawn(async move {
            loop {
                worker.wake.notified().await;
                if !worker.synchronize() {
                    break;
                }
            }
        });
        (
            Self {
                shared: shared.clone(),
                task,
                guard: Default::default(),
            },
            PcmProxyRecorder {
                shared,
                guard: Default::default(),
            },
        )
    }
}

impl AudioPlayer for PcmProxyPlayer {
    type Sample = i16;

    fn init(&mut self, ring: RingConsumer<i16>, _format: AudioStreamBasicDescription) -> RtspResult<()> {
        let mut state = self.shared.state.lock().unwrap();
        if state.stopped || state.source.is_some() {
            return Err(RtspError::DisconnectNow);
        }
        state.source = Some(ring);
        drop(state);
        self.shared.wake.notify_one();
        Ok(())
    }

    fn on_event(&mut self, event: AudioPlayerEvent) {
        if event == AudioPlayerEvent::Stop {
            self.shared.stop();
            return;
        }
        let mut state = self.shared.state.lock().unwrap();
        if state.stopped {
            return;
        }
        match event {
            AudioPlayerEvent::Start => state.playing = true,
            AudioPlayerEvent::Pause => {
                state.playing = false;
                if let Some(control) = state.control.as_ref() {
                    control.pause();
                }
            }
            AudioPlayerEvent::Anchor | AudioPlayerEvent::AddedDataToRing => {}
            AudioPlayerEvent::Stop => unreachable!(),
        }
        drop(state);
        self.shared.wake.notify_one();
    }
}

impl Drop for PcmProxyPlayer {
    fn drop(&mut self) {
        self.shared.stop();
        self.task.abort();
    }
}

pub struct PcmProxyRecorder {
    shared: Arc<ProxyShared>,
    pub guard: Arc<Mutex<Option<TeardownGuard<StreamType>>>>,
}

impl AudioRecorder for PcmProxyRecorder {
    type Sample = i16;

    fn init(&mut self, ring: RingProducer<i16>, _format: AudioStreamBasicDescription, control: AudioRecorderControl) -> RtspResult<()> {
        let mut state = self.shared.state.lock().unwrap();
        if state.stopped || state.destination.is_some() {
            return Err(RtspError::DisconnectNow);
        }
        state.destination = Some(ring);
        state.control = Some(control);
        drop(state);
        self.shared.wake.notify_one();
        Ok(())
    }

    fn on_event(&mut self, event: AudioRecorderEvent) {
        if event == AudioRecorderEvent::Stop {
            self.shared.stop();
            return;
        }
        let mut state = self.shared.state.lock().unwrap();
        if state.stopped {
            return;
        }
        match event {
            AudioRecorderEvent::Start => state.recorder_ready = true,
            AudioRecorderEvent::Pause => {
                state.recorder_ready = false;
                if let Some(control) = state.control.as_ref() {
                    control.pause();
                }
            }
            AudioRecorderEvent::DrainedDataFromRing => {}
            AudioRecorderEvent::Stop => unreachable!(),
        }
        drop(state);
        self.shared.wake.notify_one();
    }
}

impl Drop for PcmProxyRecorder {
    fn drop(&mut self) {
        self.shared.stop();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use catplay_carplay::audio::{RingBuffer, RingSegment};
    use std::time::Duration;

    fn format() -> AudioStreamBasicDescription {
        AudioStreamBasicDescription::fill_pcm(48_000, 16, 16, 2, false)
    }

    async fn wait_for(mut condition: impl FnMut() -> bool) {
        tokio::time::timeout(Duration::from_secs(1), async {
            while !condition() {
                tokio::task::yield_now().await;
            }
        })
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn unequal_rings_refill_on_drain_without_new_player_data() {
        for recorder_first in [false, true] {
            let (mut player, mut recorder) = PcmProxyPlayer::pair();
            let (mut input, source) = RingBuffer::spsc(8192);
            let (mut destination, mut output) = RingBuffer::spsc(8);
            let capacity = destination.capacity();
            let samples: Vec<i16> = (0..capacity * 3 + 112).map(|i| i as i16).collect();
            input.write(&samples);
            let control = AudioRecorderControl::default();
            if recorder_first {
                recorder
                    .init(destination, format(), control.clone())
                    .unwrap();
                recorder.on_event(AudioRecorderEvent::Start);
                player.on_event(AudioPlayerEvent::AddedDataToRing);
                player.on_event(AudioPlayerEvent::Start);
                // Start arriving before the consumer is attached must be retained.
                player.init(source, format()).unwrap();
            } else {
                player.init(source, format()).unwrap();
                recorder
                    .init(destination, format(), control.clone())
                    .unwrap();
                recorder.on_event(AudioRecorderEvent::Start);
                player.on_event(AudioPlayerEvent::Anchor);
                player.on_event(AudioPlayerEvent::AddedDataToRing);
                tokio::task::yield_now().await;
                assert_eq!(input.stat().consumed, 0);
                assert_eq!(output.stat().written, 0);
                assert!(!control.is_running());
                player.on_event(AudioPlayerEvent::Start);
            }
            wait_for(|| input.stat().consumed == capacity && control.is_running()).await;
            let mut received = Vec::new();
            loop {
                let readable = output.stat().readable;
                let mut chunk = vec![0; readable];
                output.read(&mut chunk).unwrap();
                received.extend_from_slice(&chunk);
                if received.len() == samples.len() {
                    break;
                }
                let expected_consumed = (received.len() + capacity).min(samples.len());
                recorder.on_event(AudioRecorderEvent::DrainedDataFromRing);
                wait_for(|| input.stat().consumed == expected_consumed).await;
            }
            assert_eq!(received, samples);
            assert_eq!(input.stat().consumed, samples.len());
            recorder.on_event(AudioRecorderEvent::Stop);
            assert!(!control.is_running());
            wait_for(|| player.task.is_finished()).await;
        }
    }

    #[tokio::test]
    async fn missing_timeline_survives_many_partial_transfers() {
        let (mut player, mut recorder) = PcmProxyPlayer::pair();
        let (mut input, source) = RingBuffer::<i16>::spsc(8192);
        let capacity = input.capacity();
        input.advance_timeline(capacity + 32);
        input.write(&[7; 128]);
        let total = capacity + 160;
        let (destination, output) = RingBuffer::spsc(8);
        let control = AudioRecorderControl::default();
        player.init(source, format()).unwrap();
        recorder.init(destination, format(), control).unwrap();
        player.on_event(AudioPlayerEvent::Start);
        recorder.on_event(AudioRecorderEvent::Start);
        let mut missing = 0;
        let mut data = Vec::new();
        while missing + data.len() < total {
            let before = input.stat().consumed;
            wait_for(|| input.stat().consumed > before || input.stat().consumed == total).await;
            for segment in output.iter_segments(usize::MAX, false) {
                match segment {
                    RingSegment::Gap(n) => {
                        assert!(data.is_empty());
                        missing += n;
                    }
                    RingSegment::Data(samples) => data.extend_from_slice(samples),
                    other => panic!("unexpected destination segment: {other:?}"),
                }
            }
            recorder.on_event(AudioRecorderEvent::DrainedDataFromRing);
        }
        assert_eq!(missing, capacity + 32);
        assert_eq!(data, [7; 128]);
        assert_eq!(input.stat().consumed, total);
    }

    #[tokio::test]
    async fn pause_retains_rings_resume_refills_and_stop_is_terminal() {
        let (mut player, mut recorder) = PcmProxyPlayer::pair();
        let (mut input, source) = RingBuffer::spsc(8192);
        let (mut destination, mut output) = RingBuffer::spsc(8);
        let capacity = destination.capacity();
        input.write(&vec![1; capacity * 2]);
        let control = AudioRecorderControl::default();
        player.init(source, format()).unwrap();
        recorder
            .init(destination, format(), control.clone())
            .unwrap();
        player.on_event(AudioPlayerEvent::Start);
        recorder.on_event(AudioRecorderEvent::Start);
        wait_for(|| input.stat().consumed == capacity).await;

        player.on_event(AudioPlayerEvent::Pause);
        assert!(!control.is_running());
        output.read_commit(capacity);
        recorder.on_event(AudioRecorderEvent::DrainedDataFromRing);
        // Explicitly process the same state to verify pause independently of scheduling.
        assert!(player.shared.synchronize());
        assert_eq!(input.stat().consumed, capacity);
        player.on_event(AudioPlayerEvent::Start);
        wait_for(|| input.stat().consumed == capacity * 2 && control.is_running()).await;
        player.on_event(AudioPlayerEvent::Stop);
        assert!(!control.is_running());
        player.on_event(AudioPlayerEvent::Start);
        recorder.on_event(AudioRecorderEvent::Start);
        assert!(!player.shared.synchronize());
        assert!(!control.is_running());
        assert_eq!(output.stat().readable, capacity);
    }
}
