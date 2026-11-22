use std::time::Duration;

use log::debug;

/// Manages PLL clock offset synchronization between iPhone (remote) and CarPlay headunit (local).
#[derive(Debug, Clone)]
pub struct MediaPll {
    /// Constant time base difference (CarPlay vs iPhone)
    pub offset_ns: i128,

    /// Short-term phase correction
    pub phase_adjust_ns: i128,

    /// Long-term phase correction (ns/s)
    pub freq_offset_ns_per_s: i128,

    /// Last udpate (monotonic ns)
    pub last_adjust_ns: i128,

    pub drift_base_ns: i128,
}

impl MediaPll {
    const DISABLE_PLL: bool = true;

    pub fn new(now_ns: i128, offset_ns: i128) -> Self {
        Self {
            offset_ns,
            phase_adjust_ns: 0,
            freq_offset_ns_per_s: 0,
            last_adjust_ns: now_ns,
            drift_base_ns: now_ns,
        }
    }

    /// Converts a timestamp from the **local** clock domain (e.g. HU monotonic clock)
    /// into the **remote** clock domain (e.g. iPhone time base).
    ///
    /// This function applies the current PLL state (offset, phase and drift correction)
    /// to translate a locally measured timestamp into its corresponding value
    /// in the remote clock space.
    ///
    /// Formula:
    /// `t_remote = t_local + offset + phase + drift`
    pub fn local_to_remote(&self, local_ns: i128) -> i128 {
        if Self::DISABLE_PLL {
            return local_ns + self.offset_ns;
        }

        let offset = self.offset_ns;
        let phase = self.phase_adjust_ns;

        let dt_ns = local_ns - self.drift_base_ns;
        let drift = (self.freq_offset_ns_per_s * dt_ns) / 1_000_000_000;

        let pll_total = phase + drift;

        debug!(
            "PLL local->remote: base={} offset={} phase={} dt={} drift={} total={}{:?}",
            self.drift_base_ns,
            offset,
            phase,
            dt_ns,
            drift,
            if pll_total.is_negative() { "-" } else { "" },
            Duration::from_nanos(pll_total.unsigned_abs() as _)
        );

        local_ns + offset + pll_total
    }

    /// Converts a timestamp from the **remote** clock domain (e.g. iPhone)
    /// back into the **local** clock domain (e.g. HU monotonic clock).
    ///
    /// This is the inverse of `local_to_remote()`. It removes the PLL corrections
    /// to recover the corresponding local time when a given remote timestamp
    /// would occur on the host device.
    ///
    /// Formula:
    /// `t_local = t_remote - (offset + phase + drift)`
    pub fn remote_to_local(&self, remote_ns: i128) -> i128 {
        if Self::DISABLE_PLL {
            return remote_ns - self.offset_ns;
        }

        let offset = self.offset_ns;
        let phase = self.phase_adjust_ns;

        let base = self.drift_base_ns;
        let f = self.freq_offset_ns_per_s;

        // remote - offset - phase = local + f*(local - base)/1e9
        // Let denom = 1e9 + f, then:
        // local = base + (remote - offset - phase - base) * 1e9 / denom
        let denom = 1_000_000_000i128 + f;

        // denom can be zero only if f == -1e9 ns/s (impossible with your clamps)
        debug_assert!(denom != 0);

        let num = remote_ns - offset - phase - base;
        let local_ns = base + (num * 1_000_000_000i128) / denom;

        // for debug symmetry, compute drift at resulting local
        let dt_ns = local_ns - base;
        let drift = (f * dt_ns) / 1_000_000_000;
        let pll_total = phase + drift;

        debug!(
            "PLL remote->local: base={} offset={} phase={} denom={} num={} dt={} drift={} total={}{:?}",
            base,
            offset,
            phase,
            denom,
            num,
            dt_ns,
            drift,
            if pll_total.is_negative() { "-" } else { "" },
            Duration::from_nanos(pll_total.unsigned_abs() as _)
        );

        local_ns
    }

    /// Updates the PLL based on the measured phase error.
    pub fn update(&mut self, now_ns: i128, phase_error_ns: i128) {
        const MAX_PHASE_NS: i128 = 500_000_000; // ±0.5s
        const MAX_FREQ_NS_PER_S: i128 = 500_000; // ±500 ppm
        const PLL_SHIFT: u32 = 4; // gain = 1/16

        let dt_ns = now_ns - self.last_adjust_ns;
        if dt_ns <= 0 {
            return;
        }

        // --- REBASE drift up to now into phase_adjust, then move base to now ---
        // This keeps the mapping continuous while preventing "drift grows forever".
        {
            let base_dt_ns = now_ns - self.drift_base_ns;
            if base_dt_ns != 0 && self.freq_offset_ns_per_s != 0 {
                let carried = (self.freq_offset_ns_per_s * base_dt_ns) / 1_000_000_000;
                self.phase_adjust_ns += carried;
            }
            self.drift_base_ns = now_ns;
        }

        let phase = phase_error_ns.clamp(-MAX_PHASE_NS, MAX_PHASE_NS);

        // Frequency correction (very slow)
        let corr_freq = (phase >> (PLL_SHIFT + 4)) * dt_ns / 1_000_000_000;
        self.freq_offset_ns_per_s = (self.freq_offset_ns_per_s + corr_freq).clamp(-MAX_FREQ_NS_PER_S, MAX_FREQ_NS_PER_S);

        // Phase correction (faster)
        let corr_phase = phase >> PLL_SHIFT;
        self.phase_adjust_ns -= corr_phase;

        self.last_adjust_ns = now_ns;

        debug!(
            "PLL: raw_offset={} ns, phase_error={}{:?}, corr_phase={}{:?}, freq_off={}{:?}/s, prev_adjust={:?} ago, base={}",
            self.offset_ns,
            if phase_error_ns.is_negative() { "-" } else { "" },
            Duration::from_nanos(phase_error_ns.unsigned_abs() as _),
            if corr_phase.is_negative() { "-" } else { "" },
            Duration::from_nanos(corr_phase.unsigned_abs() as _),
            if self.freq_offset_ns_per_s.is_negative() { "-" } else { "" },
            Duration::from_nanos(self.freq_offset_ns_per_s.unsigned_abs() as _),
            Duration::from_nanos(dt_ns as _),
            self.drift_base_ns
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_round_trip_basic() {
        let mut pll = MediaPll::new(0, 123_000); // offset 123 µs
        pll.phase_adjust_ns = 500; // +0.5 µs phase
        pll.freq_offset_ns_per_s = 100; // 100 ns/s drift

        let local_ts = 2_000_000_000i128; // 2 s

        // Convert to remote and back
        let remote_ts = pll.local_to_remote(local_ts);
        let roundtrip = pll.remote_to_local(remote_ts);

        let diff = (roundtrip - local_ts).abs();
        assert!(diff <= 1, "round-trip error too high: {} ns (expected ≤1 ns)", diff);
    }

    #[test]
    fn test_drift_effect() {
        let mut pll = MediaPll::new(0, 0);
        pll.freq_offset_ns_per_s = 1_000; // 1 µs per second drift

        let t1 = 0i128;
        let t2 = 1_000_000_000i128; // 1 s later

        let remote1 = pll.local_to_remote(t1);
        let remote2 = pll.local_to_remote(t2);

        let drift = remote2 - remote1 - (t2 - t1);
        // Drift correction is currently switched off; the mapping is then a plain offset.
        let expected = if MediaPll::DISABLE_PLL { 0 } else { 1_000 };
        assert_eq!(drift, expected, "drift mismatch: got {} ns", drift);
    }
    #[test]
    fn test_round_trip_after_time_passes() {
        let mut pll = MediaPll::new(0, 0);
        pll.freq_offset_ns_per_s = 500_000; // 500 µs/s

        let t0 = 0i128;
        let t1 = 40_000_000_000i128; // 40 s later

        let r0 = pll.local_to_remote(t0);
        let l0 = pll.remote_to_local(r0);
        assert_eq!(l0, t0);

        let r1 = pll.local_to_remote(t1);
        let l1 = pll.remote_to_local(r1);

        let diff = l1 - t1;
        assert!(diff.abs() < 1, "round-trip drifted after time: {} ns", diff);
    }
}
