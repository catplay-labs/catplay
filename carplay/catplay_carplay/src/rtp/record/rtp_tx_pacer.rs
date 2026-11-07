use std::{
    num::NonZeroU32,
    time::{Duration, Instant},
};

const NS_PER_SEC: u128 = 1_000_000_000;

#[derive(Debug, Clone, Copy)]
pub struct RtpPacketMeta {
    pub first_ts: u64,
    pub end_ts: u64, // exclusive
}

#[derive(Debug, Clone, Copy)]
pub struct PaceDecision {
    pub permits: usize,
    pub wake_at: Option<Instant>,
}

#[derive(Debug, Clone, Copy)]
struct MediaAnchor {
    instant: Instant,
    first_ts: u64,
    initial_credit: u64,
}

pub struct RtpTxPacer {
    sample_rate: NonZeroU32,
    /// How many seconds of audio can we send ahead relative to the media anchor.
    ///
    /// Big values can lead to remote dropping data.
    ///
    /// A value of 0 will aim to create real-time pacing, similar to streams like the microphone stream
    /// where this effect would occur naturally.
    write_ahead: Duration,

    /// We anchor our media timeline at the moment of sending first UDP packet.
    ///
    /// Remote side will usually anchor at the moment of receiving this packet.
    ///
    /// This gives us a good-enough approximation of which timestamp the remote is currently playing.
    anchor: Option<MediaAnchor>,
    last_end_ts: Option<u64>,

    // Manages secondary pacing mechanism.
    //
    // Even if we have a non-zero `write_ahead`, for example `150ms`, we should not aim to deliver all of it at once
    // as some remotes have limited UDP receive queues and it may result in packet loss.

    // GCRA: theoretical arrival time for the next packet.
    udp_tat: Option<Instant>,
    udp_interval: Duration,
    udp_burst: NonZeroU32,
}

impl RtpTxPacer {
    pub fn new(sample_rate: NonZeroU32, write_ahead: Duration, udp_interval: Duration, udp_burst: NonZeroU32) -> Self {
        assert!(!udp_interval.is_zero());

        Self {
            sample_rate,
            write_ahead,
            anchor: None,
            last_end_ts: None,
            udp_tat: None,
            udp_interval,
            udp_burst,
        }
    }

    /// Freeze the media and UDP schedules for time spent paused.
    pub(super) fn delay(&mut self, duration: Duration) {
        if let Some(anchor) = self.anchor.as_mut() {
            anchor.instant += duration;
        }
        if let Some(tat) = self.udp_tat.as_mut() {
            *tat += duration;
        }
    }

    pub(super) fn udp_interval(&self) -> Duration {
        self.udp_interval
    }

    #[cfg(test)]
    pub(super) fn last_end_ts(&self) -> Option<u64> {
        self.last_end_ts
    }

    #[cfg(test)]
    pub(super) fn has_anchor(&self) -> bool {
        self.anchor.is_some()
    }

    fn duration_samples(&self, duration: Duration) -> u64 {
        let samples = duration
            .as_nanos()
            .saturating_mul(self.sample_rate.get() as u128)
            / NS_PER_SEC;

        samples.min(u64::MAX as u128) as u64
    }

    fn initial_credit(&self, first: RtpPacketMeta) -> u64 {
        // At least one complete packet can be sent immediately.
        self.duration_samples(self.write_ahead)
            .max(first.end_ts - first.first_ts)
    }

    fn media_deadline(&self, now: Instant, packet: RtpPacketMeta) -> Instant {
        let Some(anchor) = self.anchor else {
            // Before the first commit, only the initial
            // credit is available.
            return now;
        };

        let offset = packet.end_ts.saturating_sub(anchor.first_ts);
        let credit = anchor.initial_credit;

        if offset <= credit {
            return now;
        }

        let needed_samples = offset - credit;

        // ceil(samples * 1e9 / sample_rate)
        let rate = self.sample_rate.get() as u128;
        let ns = (needed_samples as u128 * NS_PER_SEC)
            .div_ceil(rate)
            .min(u64::MAX as u128) as u64;

        anchor.instant + Duration::from_nanos(ns)
    }

    fn udp_tolerance(&self) -> Duration {
        self.udp_interval
            .checked_mul(self.udp_burst.get() - 1)
            .expect("UDP burst tolerance overflow")
    }

    fn udp_deadline(&self, now: Instant, tat: Option<Instant>) -> Instant {
        tat.and_then(|t| t.checked_sub(self.udp_tolerance()))
            .unwrap_or(now)
    }

    fn udp_advance(&self, now: Instant, tat: Option<Instant>) -> Option<Instant> {
        Some(tat.unwrap_or(now).max(now) + self.udp_interval)
    }

    /// Evaluate a FIFO prefix without changing pacer state.
    ///
    /// `wake_at` is meaningful when permits == 0.
    pub fn inspect(&self, now: Instant, packets: impl IntoIterator<Item = RtpPacketMeta>, max_batch: usize) -> PaceDecision {
        let mut permits = 0;
        let mut tat = self.udp_tat;
        let mut first = None;

        for packet in packets.into_iter().take(max_batch) {
            assert!(packet.end_ts > packet.first_ts);

            let initial = *first.get_or_insert(packet);

            let media_at = if self.anchor.is_some() {
                self.media_deadline(now, packet)
            } else {
                let credit = self.initial_credit(initial);
                let offset = packet.end_ts - initial.first_ts;

                // Cannot establish a timed deadline until
                // the first packet has actually been sent.
                if offset > credit {
                    break;
                }

                now
            };

            let udp_at = self.udp_deadline(now, tat);
            let ready_at = media_at.max(udp_at);

            if ready_at > now {
                return PaceDecision {
                    permits,
                    wake_at: (permits == 0).then_some(ready_at),
                };
            }

            permits += 1;
            tat = self.udp_advance(now, tat);
        }

        PaceDecision { permits, wake_at: None }
    }

    /// Commit the successfully transmitted FIFO prefix.
    ///
    /// All packets in a sendmmsg batch share the same
    /// approximate transmission time.
    pub fn commit(&mut self, now: Instant, sent: impl IntoIterator<Item = RtpPacketMeta>) {
        for packet in sent {
            assert!(packet.end_ts > packet.first_ts);

            if let Some(previous) = self.last_end_ts {
                assert!(packet.first_ts >= previous);
            }

            if self.anchor.is_none() {
                self.anchor = Some(MediaAnchor {
                    instant: now,
                    first_ts: packet.first_ts,
                    initial_credit: self.initial_credit(packet),
                });
            }

            self.udp_tat = self.udp_advance(now, self.udp_tat);
            self.last_end_ts = Some(packet.end_ts);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn media_and_udp_limits_return_exact_wakeup() {
        let now = Instant::now();
        let mut pacer = RtpTxPacer::new(
            NonZeroU32::new(48_000).unwrap(),
            Duration::ZERO,
            Duration::from_millis(1),
            NonZeroU32::new(2).unwrap(),
        );
        let first = RtpPacketMeta { first_ts: 0, end_ts: 480 };
        let next = RtpPacketMeta {
            first_ts: 480,
            end_ts: 960,
        };
        assert_eq!(pacer.inspect(now, [first, next], 16).permits, 1);
        pacer.commit(now, [first]);
        let decision = pacer.inspect(now, [next], 16);
        assert_eq!(decision.permits, 0);
        assert_eq!(decision.wake_at, Some(now + Duration::from_millis(10)));
        assert_eq!(pacer.inspect(decision.wake_at.unwrap(), [next], 16).permits, 1);

        let mut pacer = RtpTxPacer::new(
            NonZeroU32::new(48_000).unwrap(),
            Duration::from_millis(150),
            Duration::from_millis(1),
            NonZeroU32::new(2).unwrap(),
        );
        pacer.commit(now, [first, next]);
        let third = RtpPacketMeta {
            first_ts: 960,
            end_ts: 1440,
        };
        let decision = pacer.inspect(now, [third], 16);
        assert_eq!(decision.permits, 0);
        assert_eq!(decision.wake_at, Some(now + Duration::from_millis(1)));
    }
}

#[cfg(test)]
mod pause_tests {
    use super::*;

    #[test]
    fn resume_preserves_remaining_media_delay() {
        let now = Instant::now();
        let mut pacer = RtpTxPacer::new(
            NonZeroU32::new(48_000).unwrap(),
            Duration::ZERO,
            Duration::from_millis(1),
            NonZeroU32::new(4).unwrap(),
        );
        pacer.commit(now, [RtpPacketMeta { first_ts: 0, end_ts: 480 }]);
        let next = RtpPacketMeta {
            first_ts: 480,
            end_ts: 960,
        };
        let before = pacer.inspect(now, [next], 1).wake_at.unwrap();
        let paused = Duration::from_secs(10);
        pacer.delay(paused);
        assert_eq!(pacer.inspect(now + paused, [next], 1).permits, 0);
        assert_eq!(pacer.inspect(now + paused, [next], 1).wake_at, Some(before + paused));
        assert_eq!(pacer.inspect(before + paused, [next], 1).permits, 1);
    }
}
