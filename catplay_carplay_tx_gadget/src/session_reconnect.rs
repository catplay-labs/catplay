use std::time::{Duration, Instant};

/// A remembered radio endpoint is valid only for the USB session that advertised it.
pub(super) struct SessionReconnect<T> {
    session: Option<String>,
    target: Option<T>,
    deadline: Option<Instant>,
    next_attempt: Option<Instant>,
    attempts: u8,
}

impl<T> Default for SessionReconnect<T> {
    fn default() -> Self {
        Self {
            session: None,
            target: None,
            deadline: None,
            next_attempt: None,
            attempts: 0,
        }
    }
}

impl<T: Clone> SessionReconnect<T> {
    pub fn set_session(&mut self, session: Option<&str>) {
        if self.session.as_deref() != session {
            self.session = session.map(str::to_owned);
            self.target = None;
            self.connected();
        }
    }

    pub fn remember(&mut self, target: T) {
        if self.session.is_some() {
            self.target = Some(target);
        }
    }

    pub fn request(&mut self, now: Instant) {
        if self.session.is_some() {
            self.deadline = Some(now + Duration::from_secs(30));
            self.next_attempt = Some(now + Duration::from_millis(250));
            self.attempts = 0;
        }
    }

    pub fn connected(&mut self) {
        self.deadline = None;
        self.next_attempt = None;
        self.attempts = 0;
    }

    pub fn requested(&self) -> bool {
        self.deadline.is_some()
    }

    pub fn take_due(&mut self, now: Instant) -> Option<T> {
        let deadline = self.deadline?;
        if now >= deadline || self.attempts >= 3 {
            self.connected();
            return None;
        }
        if now < self.next_attempt? {
            return None;
        }
        let target = self.target.clone()?;
        self.attempts += 1;
        self.next_attempt = Some(now + Duration::from_secs(2));
        Some(target)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn remembered_radio_is_retried_after_release_and_retries_are_bounded() {
        let now = Instant::now();
        let mut r = SessionReconnect::default();
        r.set_session(Some("usb-a"));
        r.remember(42);
        r.request(now);
        assert_eq!(r.take_due(now), None);
        assert_eq!(r.take_due(now + Duration::from_millis(250)), Some(42));
        assert_eq!(r.take_due(now + Duration::from_secs(1)), None);
        assert_eq!(r.take_due(now + Duration::from_secs(3)), Some(42));
        assert_eq!(r.take_due(now + Duration::from_secs(5)), Some(42));
        assert_eq!(r.take_due(now + Duration::from_secs(7)), None);
        assert!(!r.requested());
    }

    #[test]
    fn old_radio_is_never_reused_after_usb_session_change_or_disconnect() {
        for next in [Some("usb-b"), None] {
            let now = Instant::now();
            let mut r = SessionReconnect::default();
            r.set_session(Some("usb-a"));
            r.remember(42);
            r.request(now);
            r.set_session(next);
            r.request(now);
            assert_eq!(r.take_due(now + Duration::from_secs(1)), None);
        }
    }

    #[test]
    fn successful_reconnect_cancels_retries_but_keeps_current_radio() {
        let now = Instant::now();
        let mut r = SessionReconnect::default();
        r.set_session(Some("usb-a"));
        r.remember(42);
        r.request(now);
        r.connected();
        assert_eq!(r.take_due(now + Duration::from_secs(3)), None);
        r.request(now + Duration::from_secs(4));
        assert_eq!(r.take_due(now + Duration::from_secs(5)), Some(42));
    }

    #[test]
    fn no_endpoint_waits_for_fresh_invite_and_deadline_expires() {
        let now = Instant::now();
        let mut r = SessionReconnect::default();
        r.set_session(Some("usb-a"));
        r.request(now);
        assert_eq!(r.take_due(now + Duration::from_secs(1)), None);
        r.remember(42);
        assert_eq!(r.take_due(now + Duration::from_secs(2)), Some(42));
        assert_eq!(r.take_due(now + Duration::from_secs(31)), None);
        assert!(!r.requested());
    }
}
