use std::time::{Duration, Instant};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum IndicatorState {
    Waiting,
    Connected,
    Switching,
    Error,
}

pub(crate) fn led_on(state: IndicatorState, elapsed: Duration) -> bool {
    match state {
        IndicatorState::Connected => true,
        IndicatorState::Waiting => elapsed.as_millis() % 2000 < 500,
        IndicatorState::Switching => elapsed.as_millis() % 250 < 125,
        IndicatorState::Error => elapsed.as_millis() % 1000 < 100,
    }
}

/// Software debounce, one action on short release, no action for boot-held keys.
pub(crate) struct Button {
    candidate: bool,
    since: Instant,
    stable: Option<bool>,
    pressed_at: Option<Instant>,
    armed: bool,
    last_action: Option<Instant>,
}

impl Button {
    pub(crate) fn new(now: Instant) -> Self {
        Self {
            candidate: false,
            since: now,
            stable: None,
            pressed_at: None,
            armed: false,
            last_action: None,
        }
    }

    pub(crate) fn sample(&mut self, pressed: bool, now: Instant) -> bool {
        if pressed != self.candidate {
            self.candidate = pressed;
            self.since = now;
        }
        if now.duration_since(self.since) < Duration::from_millis(40) || self.stable == Some(pressed) {
            return false;
        }
        self.stable = Some(pressed);
        if pressed {
            if self.armed {
                self.pressed_at = Some(now);
            }
            return false;
        }
        self.armed = true;
        let Some(start) = self.pressed_at.take() else { return false };
        let held = now.duration_since(start);
        if held < Duration::from_millis(40) || held > Duration::from_secs(2) {
            return false;
        }
        if self
            .last_action
            .is_some_and(|t| now.duration_since(t) < Duration::from_secs(1))
        {
            return false;
        }
        self.last_action = Some(now);
        true
    }
}

/// WS2812 GRB: 0 => 100, 1 => 110 at 2.4 MHz. Reset low before/after frame.
pub(crate) fn rgb_spi_frame(on: bool) -> Vec<u8> {
    let mut frame = vec![0; 96];
    let mut bits = Vec::with_capacity(72);
    // Dim green; no unbounded LED current or board-specific colour assumptions.
    for component in [if on { 16u8 } else { 0 }, 0, 0] {
        for bit in (0..8).rev() {
            bits.extend_from_slice(if component & (1 << bit) != 0 {
                &[true, true, false]
            } else {
                &[true, false, false]
            });
        }
    }
    for byte in bits.chunks_exact(8) {
        frame.push(byte.iter().fold(0, |v, bit| (v << 1) | u8::from(*bit)));
    }
    frame.extend_from_slice(&[0; 96]);
    frame
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn boot_hold_long_hold_and_bounce_are_ignored() {
        let start = Instant::now();
        let mut key = Button::new(start);
        let mut sample = |ms, pressed| key.sample(pressed, start + Duration::from_millis(ms));
        for (ms, pressed) in [
            (0, true),
            (50, true),
            (100, false),
            (150, false),
            (200, true),
            (210, false),
            (300, false),
            (400, true),
            (450, true),
            (2500, false),
            (2550, false),
        ] {
            assert!(!sample(ms, pressed));
        }
    }
    #[test]
    fn short_release_fires_once_and_repeat_is_rate_limited() {
        let start = Instant::now();
        let mut key = Button::new(start);
        let events = [
            (50, false, false),
            (100, true, false),
            (150, true, false),
            (300, false, false),
            (350, false, true),
            (400, false, false),
            (450, true, false),
            (500, true, false),
            (600, false, false),
            (650, false, false),
            (1500, true, false),
            (1550, true, false),
            (1700, false, false),
            (1750, false, true),
        ];
        for (ms, pressed, expected) in events {
            assert_eq!(key.sample(pressed, start + Duration::from_millis(ms)), expected, "at {ms}");
        }
    }
    #[test]
    fn rgb_frame_is_grb_with_reset_and_valid_symbols() {
        for on in [false, true] {
            let frame = rgb_spi_frame(on);
            assert_eq!(frame.len(), 201);
            assert!(frame[..96].iter().chain(&frame[105..]).all(|v| *v == 0));
            let bits: Vec<_> = frame[96..105]
                .iter()
                .flat_map(|v| (0..8).rev().map(move |bit| (v >> bit) & 1))
                .collect();
            let mut decoded = Vec::new();
            for symbol in bits.chunks_exact(3) {
                assert!(symbol == [1, 0, 0] || symbol == [1, 1, 0]);
                decoded.push(symbol[1]);
            }
            let channels: Vec<u8> = decoded
                .chunks_exact(8)
                .map(|bits| bits.iter().fold(0, |v, bit| (v << 1) | bit))
                .collect();
            assert_eq!(channels, [if on { 16 } else { 0 }, 0, 0]);
        }
    }
    #[test]
    fn connected_is_solid_and_switching_is_faster_than_waiting() {
        assert!(led_on(IndicatorState::Connected, Duration::from_secs(19)));
        assert!(led_on(IndicatorState::Waiting, Duration::ZERO));
        assert!(!led_on(IndicatorState::Waiting, Duration::from_millis(1000)));
        assert!(!led_on(IndicatorState::Switching, Duration::from_millis(150)));
        assert!(led_on(IndicatorState::Switching, Duration::from_millis(250)));
        assert!(!led_on(IndicatorState::Error, Duration::from_millis(200)));
    }
}
