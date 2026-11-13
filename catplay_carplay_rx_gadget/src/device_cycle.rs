use std::time::{Duration, Instant};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SwitchTimeout {
    Bluetooth,
    Radio,
    CarPlay,
}

/// Bluetooth and radio setup run concurrently. CarPlay gets its own budget
/// only after both prerequisites have completed.
#[derive(Clone, Copy)]
pub(crate) struct SwitchTiming {
    bluetooth_deadline: Instant,
    bluetooth_ready: bool,
    radio_deadline: Option<Instant>,
    carplay_deadline: Option<Instant>,
}

impl SwitchTiming {
    pub fn new(now: Instant) -> Self {
        Self {
            bluetooth_deadline: now + Duration::from_secs(60),
            bluetooth_ready: false,
            radio_deadline: Some(now + Duration::from_secs(30)),
            carplay_deadline: None,
        }
    }

    pub fn bluetooth_deadline(&self) -> Instant {
        self.bluetooth_deadline
    }

    pub fn bluetooth_connected(&mut self) {
        self.bluetooth_ready = true;
    }

    pub fn observe(&mut self, now: Instant, radio_ready: bool) -> Option<SwitchTimeout> {
        if !self.bluetooth_ready && now >= self.bluetooth_deadline {
            return Some(SwitchTimeout::Bluetooth);
        }
        if radio_ready {
            self.radio_deadline = None;
            if self.bluetooth_ready {
                let deadline = self
                    .carplay_deadline
                    .get_or_insert(now + Duration::from_secs(60));
                if now >= *deadline {
                    return Some(SwitchTimeout::CarPlay);
                }
            }
        } else {
            let deadline = self
                .radio_deadline
                .get_or_insert(now + Duration::from_secs(30));
            if now >= *deadline {
                return Some(SwitchTimeout::Radio);
            }
        }
        None
    }
}

/// Stable ordering avoids depending on BlueZ's hash-map iteration order.
pub(crate) fn next_device<T: Copy + Eq>(devices: &[T], current: Option<T>) -> Option<T> {
    if let Some(index) = current.and_then(|current| devices.iter().position(|v| *v == current)) {
        return (devices.len() > 1).then(|| devices[(index + 1) % devices.len()]);
    }
    devices.first().copied()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn cycle_wraps_and_skips_current() {
        assert_eq!(next_device(&[1, 2, 3], Some(2)), Some(3));
        assert_eq!(next_device(&[1, 2, 3], Some(3)), Some(1));
    }
    #[test]
    fn no_alternative_does_not_disconnect() {
        assert_eq!(next_device::<u8>(&[], Some(1)), None);
        assert_eq!(next_device(&[1], Some(1)), None);
        assert_eq!(next_device(&[2], Some(1)), Some(2));
        assert_eq!(next_device(&[1, 2], None), Some(1));
    }

    #[test]
    fn bluetooth_does_not_wait_for_radio_and_has_its_own_budget() {
        let now = Instant::now();
        let mut timing = SwitchTiming::new(now);
        assert_eq!(timing.bluetooth_deadline(), now + Duration::from_secs(60));
        assert_eq!(timing.observe(now + Duration::from_secs(29), false), None);
        assert_eq!(timing.observe(now + Duration::from_secs(30), false), Some(SwitchTimeout::Radio));
    }

    #[test]
    fn carplay_budget_starts_when_both_prerequisites_are_ready() {
        let now = Instant::now();
        let mut timing = SwitchTiming::new(now);
        assert_eq!(timing.observe(now + Duration::from_secs(20), true), None);
        timing.bluetooth_connected();
        assert_eq!(timing.observe(now + Duration::from_secs(55), true), None);
        assert_eq!(timing.observe(now + Duration::from_secs(114), true), None);
        assert_eq!(timing.observe(now + Duration::from_secs(115), true), Some(SwitchTimeout::CarPlay));
    }

    #[test]
    fn early_bluetooth_does_not_start_carplay_timeout_while_radio_is_missing() {
        let now = Instant::now();
        let mut timing = SwitchTiming::new(now);
        timing.bluetooth_connected();
        assert_eq!(timing.observe(now + Duration::from_secs(29), true), None);
        assert_eq!(timing.observe(now + Duration::from_secs(88), true), None);
        assert_eq!(timing.observe(now + Duration::from_secs(89), true), Some(SwitchTimeout::CarPlay));
    }

    #[test]
    fn a_ready_radio_cannot_hide_bluetooth_failure() {
        let now = Instant::now();
        let mut timing = SwitchTiming::new(now);
        assert_eq!(timing.observe(now + Duration::from_secs(59), true), None);
        assert_eq!(timing.observe(now + Duration::from_secs(60), true), Some(SwitchTimeout::Bluetooth));
    }

    #[test]
    fn radio_loss_has_a_bounded_recovery_without_extending_carplay_budget() {
        let now = Instant::now();
        let mut timing = SwitchTiming::new(now);
        timing.bluetooth_connected();
        assert_eq!(timing.observe(now, true), None);
        assert_eq!(timing.observe(now + Duration::from_secs(5), false), None);
        assert_eq!(timing.observe(now + Duration::from_secs(34), false), None);
        assert_eq!(timing.observe(now + Duration::from_secs(35), false), Some(SwitchTimeout::Radio));
        assert_eq!(timing.observe(now + Duration::from_secs(59), true), None);
        assert_eq!(timing.observe(now + Duration::from_secs(60), true), Some(SwitchTimeout::CarPlay));
    }
}
