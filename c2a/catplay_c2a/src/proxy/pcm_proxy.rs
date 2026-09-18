use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

use catplay_carplay::{
    audio::{AudioPlayer, AudioRecorder, AudioSinkBox, AudioSourceBox, RingBuffer, RingConsumer, RingProducer},
    carplay_tx::TeardownGuard,
    msg::StreamType,
    rtsp_frame::RtspResult,
};
use log::{debug, info};
use tokio::{
    sync::mpsc,
    task::JoinHandle,
    time::{Instant, MissedTickBehavior},
};

// Keep the existing lead limit, but never drain that entire lead in one turn.
const READ_AHEAD: Duration = Duration::from_millis(150);
const CHUNKS_PER_TICK: usize = 2;

fn eligible_samples(elapsed: Duration, rate: u32, channels: usize, written: usize, consumed: usize) -> usize {
    let frames = (elapsed + READ_AHEAD).as_nanos() * rate as u128 / 1_000_000_000;
    let allowed = (frames * channels as u128).min(usize::MAX as u128) as usize;
    allowed.min(written).saturating_sub(consumed)
}

fn chunk_samples(eligible: usize, writable: usize, capacity: usize, channels: usize) -> usize {
    let samples = eligible.min(writable).min(capacity);
    samples / channels * channels
}

#[derive(Default)]
struct PacingStats {
    ticks: usize,
    samples: usize,
    chunks: usize,
    xruns: usize,
    blocked: usize,
    max_gap: Duration,
    ring_high_water: usize,
    eligible_high_water: usize,
    max_tick_samples: usize,
}

#[inline]
pub fn duration_to_frames_u32_saturating(d: Duration, freq: usize) -> u32 {
    let ns = d.as_nanos(); // u128
    let freq = freq as u128;

    let samples = (ns * freq) / 1_000_000_000u128;

    samples.min(u32::MAX as u128) as u32
}

/// Represents along with [PcmProxyRecorder] a loopback pair of AudioPlayer and AudioRecorder that exchange PCM samples using
/// ALSA-like high accuracy timing thread and [RingBuffer].
pub struct PcmProxyPlayer {
    ring: Option<(RingProducer<i16>, RingConsumer<i16>)>,
    source: Option<AudioSourceBox<i16>>,
    task: Option<JoinHandle<()>>,
    pub guard: Arc<Mutex<Option<TeardownGuard<StreamType>>>>,

    sample_rate: u32,
    channels_per_frame: usize,
    interval: Duration,
    receiver: Option<mpsc::Receiver<AudioSinkBox<i16>>>,
    limit: usize,
}

impl PcmProxyPlayer {
    pub fn new(
        ring: (RingProducer<i16>, RingConsumer<i16>),
        sample_rate: u32,
        interval: Duration,
        channels_per_frame: usize,
        receiver: mpsc::Receiver<AudioSinkBox<i16>>,
        limit: usize,
    ) -> Self {
        Self {
            ring: Some(ring),
            source: None,
            task: None,
            guard: Default::default(),

            sample_rate,
            channels_per_frame,
            interval,
            receiver: Some(receiver),
            limit,
        }
    }

    pub fn pair(sample_rate: u32, channels_per_frame: usize) -> (PcmProxyPlayer, PcmProxyRecorder) {
        let _latency = Duration::from_millis(32);

        const INTERVAL: Duration = Duration::from_millis(10);
        const RING_INTERVAL_MULTIPLIER: usize = 50; // 10ms interval, 500ms ring

        let channel = mpsc::channel(1);

        let ring_size_samples =
            duration_to_frames_u32_saturating(INTERVAL, sample_rate as _) as usize * channels_per_frame * RING_INTERVAL_MULTIPLIER;

        let ring = RingBuffer::spsc(ring_size_samples);
        let player = PcmProxyPlayer::new(ring, sample_rate, INTERVAL, channels_per_frame, channel.1, ring_size_samples);
        let recorder = PcmProxyRecorder::new(channel.0);
        (player, recorder)
    }
}

impl AudioPlayer for PcmProxyPlayer {
    type Sample = i16;

    fn init(&mut self, source: AudioSourceBox<Self::Sample>) -> RtspResult<()> {
        let _ring = self.ring.take().unwrap();
        self.source.replace(source);

        Ok(())
    }

    fn start(&mut self) {
        debug!("PcmProxyPlayer: start");
        if self.task.is_some() {
            return;
        }

        let Some(source) = self.source.take() else {
            debug!("PcmProxyPlayer: start without source");
            return;
        };
        let mut receiver = self.receiver.take().unwrap();
        let _limit = self.limit;
        let sample_rate = self.sample_rate;
        let channels_per_frame = self.channels_per_frame;

        let interval = self.interval;
        let task = tokio::spawn(async move {
            let mut player = source;
            // No polling timer while waiting for recorder SETUP. Dropping the player
            // aborts this wait just like the active-stream loop.
            let Some(mut recorder) = receiver.recv().await else { return };
            let start = Instant::now();
            let mut ticks = tokio::time::interval_at(start, interval);
            ticks.set_missed_tick_behavior(MissedTickBehavior::Skip);
            let capacity = duration_to_frames_u32_saturating(interval, sample_rate as _) as usize * channels_per_frame;
            let mut buf = vec![0i16; capacity];
            let mut stats = PacingStats::default();
            let mut previous_tick = start;
            let mut report_at = start;
            loop {
                ticks.tick().await;
                let now = Instant::now();
                stats.ticks += 1;
                stats.max_gap = stats.max_gap.max(now - previous_tick);
                previous_tick = now;
                let mut tick_samples = 0;
                for chunk in 0..CHUNKS_PER_TICK {
                    let stat = player.stat_raw();
                    let eligible = eligible_samples(now - start, sample_rate, channels_per_frame, stat.written, stat.consumed);
                    stats.ring_high_water = stats.ring_high_water.max(stat.readable);
                    stats.eligible_high_water = stats.eligible_high_water.max(eligible);
                    let count = chunk_samples(eligible, recorder.writable(), buf.len(), channels_per_frame);
                    if count == 0 {
                        stats.blocked += usize::from(eligible >= channels_per_frame);
                        break;
                    }
                    stats.xruns += usize::from(!player.read(&mut buf[..count]));
                    recorder.write(&buf[..count]);
                    stats.chunks += 1;
                    tick_samples += count;
                    if chunk + 1 < CHUNKS_PER_TICK {
                        tokio::task::yield_now().await;
                    }
                }
                stats.samples += tick_samples;
                stats.max_tick_samples = stats.max_tick_samples.max(tick_samples);
                if now - report_at >= Duration::from_secs(5) {
                    info!(
                        "PCM pacing rate={sample_rate} channels={channels_per_frame} window_ms={} ticks={} chunks={} samples={} max_gap_us={} ring_hwm={} eligible_hwm={} max_tick_samples={} blocked={} xruns={}",
                        (now - report_at).as_millis(),
                        stats.ticks,
                        stats.chunks,
                        stats.samples,
                        stats.max_gap.as_micros(),
                        stats.ring_high_water,
                        stats.eligible_high_water,
                        stats.max_tick_samples,
                        stats.blocked,
                        stats.xruns
                    );
                    stats = PacingStats::default();
                    report_at = now;
                }
            }
        });
        self.task.replace(task);
    }

    fn stop(&mut self, _drain: bool) {
        debug!("PcmProxyPlayer: stop");
        if let Some(task) = self.task.take() {
            task.abort();
        }
    }
}

impl Drop for PcmProxyPlayer {
    fn drop(&mut self) {
        debug!("PcmProxyPlayer: drop");
        if let Some(task) = self.task.take() {
            task.abort();
        }
    }
}

pub struct PcmProxyRecorder {
    sender: mpsc::Sender<AudioSinkBox<i16>>,
    pub guard: Arc<Mutex<Option<TeardownGuard<StreamType>>>>,
}

impl PcmProxyRecorder {
    pub fn new(sender: mpsc::Sender<AudioSinkBox<i16>>) -> Self {
        Self {
            guard: Default::default(),
            sender,
        }
    }
}

impl AudioRecorder for PcmProxyRecorder {
    type Sample = i16;

    fn init(&mut self, source: AudioSinkBox<Self::Sample>) -> RtspResult<()> {
        if let Err(err) = self.sender.try_send(source) {
            debug!("PcmProxyRecorder failed to send source: {err:?}");
        }

        Ok(())
    }

    fn start(&mut self) {
        debug!("PcmProxyRecorder: start (ignored)");
    }

    fn stop(&mut self, drain: bool) {
        debug!("PcmProxyRecorder: stop {drain} (ignored)");
    }
}

impl Drop for PcmProxyRecorder {
    fn drop(&mut self) {
        debug!("PcmProxyRecorder: drop");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use catplay_carplay::audio::{AudioSink, AudioSource, RingStats, RingStatsAudio};

    #[derive(Default)]
    struct Progress {
        consumed: usize,
        written: usize,
        writable: usize,
        reads: Vec<usize>,
        writes: Vec<usize>,
        buffer_addresses: Vec<usize>,
    }

    struct Source(Arc<Mutex<Progress>>);
    impl AudioSource for Source {
        type Sample = i16;
        fn stat(&mut self) -> RingStatsAudio {
            unreachable!("pacer only needs raw counters")
        }
        fn stat_raw(&mut self) -> RingStats {
            let p = self.0.lock().unwrap();
            RingStats {
                capacity: 48_000,
                written: p.written,
                consumed: p.consumed,
                readable: p.written.saturating_sub(p.consumed),
                overflow: 0,
                underflow: 0,
            }
        }
        fn read(&mut self, buf: &mut [i16]) -> bool {
            let mut p = self.0.lock().unwrap();
            assert!(p.consumed + buf.len() <= p.written);
            buf.fill(123);
            p.consumed += buf.len();
            p.reads.push(buf.len());
            p.buffer_addresses.push(buf.as_ptr() as usize);
            true
        }
    }

    struct Sink(Arc<Mutex<Progress>>);
    impl AudioSink for Sink {
        type Sample = i16;
        fn writable(&mut self) -> usize {
            self.0.lock().unwrap().writable
        }
        fn write(&mut self, buf: &[i16]) {
            let mut p = self.0.lock().unwrap();
            assert!(buf.iter().all(|sample| *sample == 123));
            assert!(buf.len() <= p.writable);
            p.writable -= buf.len();
            p.writes.push(buf.len());
        }
    }

    fn runtime() -> tokio::runtime::Runtime {
        tokio::runtime::Builder::new_current_thread().enable_time().start_paused(true).build().unwrap()
    }

    async fn settle() {
        // Let the proxy and its explicit between-chunk yield finish a turn.
        for _ in 0..8 {
            tokio::task::yield_now().await;
        }
    }

    #[test]
    fn delayed_ticks_have_bounded_catchup_reuse_storage_and_skip_missed_ticks() {
        runtime().block_on(async {
            let p = Arc::new(Mutex::new(Progress {
                written: 48_000,
                writable: 48_000,
                ..Default::default()
            }));
            let (mut player, mut recorder) = PcmProxyPlayer::pair(48_000, 2);
            player.init(Box::new(Source(p.clone()))).unwrap();
            recorder.init(Box::new(Sink(p.clone()))).unwrap();
            player.start();
            let observer = {
                let p = p.clone();
                tokio::spawn(async move { p.lock().unwrap().writes.len() })
            };
            settle().await;
            assert_eq!(observer.await.unwrap(), 1, "another runnable task progresses between chunks");
            assert_eq!(p.lock().unwrap().writes, [960, 960]);

            tokio::time::advance(Duration::from_millis(174)).await;
            settle().await;
            assert_eq!(p.lock().unwrap().writes.len(), 4, "only one bounded turn after a long stall");
            tokio::time::advance(Duration::from_millis(5)).await;
            settle().await;
            assert_eq!(p.lock().unwrap().writes.len(), 4);
            tokio::time::advance(Duration::from_millis(1)).await;
            settle().await;
            assert_eq!(p.lock().unwrap().writes.len(), 6, "next tick stays anchored at 180ms, not 184ms");
            {
                let p = p.lock().unwrap();
                assert_eq!(p.reads, p.writes);
                assert!(p.buffer_addresses.iter().all(|a| *a == p.buffer_addresses[0]));
            }
            player.stop(false);
            settle().await;
            tokio::time::advance(Duration::from_secs(1)).await;
            settle().await;
            assert_eq!(p.lock().unwrap().writes.len(), 6);
        });
    }

    #[test]
    fn waits_for_recorder_then_respects_empty_source_and_blocked_sink() {
        runtime().block_on(async {
            let p = Arc::new(Mutex::new(Progress::default()));
            let (mut player, mut recorder) = PcmProxyPlayer::pair(48_000, 2);
            player.init(Box::new(Source(p.clone()))).unwrap();
            player.start();
            settle().await;
            tokio::time::advance(Duration::from_secs(30)).await;
            settle().await;
            assert!(p.lock().unwrap().reads.is_empty());
            recorder.init(Box::new(Sink(p.clone()))).unwrap();
            settle().await;
            assert!(p.lock().unwrap().reads.is_empty());
            p.lock().unwrap().written = 48_000;
            tokio::time::advance(Duration::from_millis(10)).await;
            settle().await;
            assert!(p.lock().unwrap().reads.is_empty(), "full sink must not consume source");
            p.lock().unwrap().writable = 101;
            tokio::time::advance(Duration::from_millis(10)).await;
            settle().await;
            assert_eq!(
                p.lock().unwrap().writes,
                [100],
                "preserve sample frames and recheck sink capacity between chunks"
            );
            drop(player);
            settle().await;
            tokio::time::advance(Duration::from_secs(1)).await;
            settle().await;
            assert_eq!(p.lock().unwrap().writes.len(), 1);
        });
    }

    #[test]
    fn real_rtp_sink_stops_source_at_capacity_and_resumes_after_drain() {
        runtime().block_on(async {
            use catplay_carplay::rtp::record::RtpSink;
            use std::num::NonZeroUsize;

            let (ring, mut consumer) = RingBuffer::<i16>::spsc(1024);
            let capacity = consumer.capacity();
            let mut sink = RtpSink::new(ring, NonZeroUsize::new(704).unwrap(), || {});
            sink.write(&vec![9; capacity - 100]);
            let p = Arc::new(Mutex::new(Progress {
                written: 48_000,
                ..Default::default()
            }));
            let (mut player, mut recorder) = PcmProxyPlayer::pair(48_000, 2);
            player.init(Box::new(Source(p.clone()))).unwrap();
            recorder.init(Box::new(sink)).unwrap();
            player.start();
            settle().await;
            assert_eq!(p.lock().unwrap().consumed, 100);
            assert_eq!(consumer.stat().readable, capacity);

            for _ in 0..10 {
                tokio::time::advance(Duration::from_millis(10)).await;
                settle().await;
            }
            assert_eq!(p.lock().unwrap().consumed, 100, "undrained real sink must stop source consumption");
            assert_eq!(consumer.stat().overflow, 0);
            {
                let (overflow, samples) = consumer.readable_slice();
                assert_eq!(overflow, 0);
                assert!(samples[..capacity - 100].iter().all(|sample| *sample == 9));
                assert_eq!(&samples[capacity - 100..], &[123; 100]);
            }

            let mut drained = [0; 704];
            consumer.read(&mut drained).unwrap();
            tokio::time::advance(Duration::from_millis(10)).await;
            settle().await;
            assert_eq!(p.lock().unwrap().consumed, 804, "resume only into newly freed space");
            assert_eq!(consumer.stat().readable, capacity);
            assert_eq!(consumer.stat().overflow, 0);
            tokio::time::advance(Duration::from_millis(10)).await;
            settle().await;
            assert_eq!(p.lock().unwrap().consumed, 804);
            player.stop(false);
            settle().await;
        });
    }

    #[test]
    fn lead_gate_handles_starvation_and_long_running_sample_counts() {
        assert_eq!(eligible_samples(Duration::ZERO, 48_000, 2, 48_000, 0), 14_400);
        assert_eq!(eligible_samples(Duration::ZERO, 48_000, 2, 100, 100), 0);
        assert_eq!(eligible_samples(Duration::ZERO, 48_000, 2, 100, 102), 0);
        assert_eq!(chunk_samples(99, 101, 960, 2), 98);
        // No intermediate u32 narrowing of the per-channel sample cursor.
        if let Some(consumed) = (u32::MAX as usize).checked_add(1) {
            assert_eq!(
                eligible_samples(Duration::from_secs(50_000), 48_000, 2, consumed + 960, consumed),
                960
            );
        }
    }
}
