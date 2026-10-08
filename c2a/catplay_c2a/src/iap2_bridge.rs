//! Bridge between the dongle's two iAP2 sessions: the iPhone (over wireless CarPlay) and the head
//! unit (over USB). The relays here are I/O-free; the task in this module feeds them events and
//! sends what they return.

use catplay_carplay_rx_gadget::CarPlayServerSessionEventTx;
use catplay_carplay_tx_gadget::carplay_client_session::{CarPlayClientSessionEventTx, now_playing_cleared};
use catplay_csm::msg::NowPlayingUpdate;
use catplay_iap2_client::CsmClientHandleRef;
use catplay_util::{AbortOnDropHandle, mpsc, spawn};
use futures::{StreamExt, select};
use log::{debug, info, warn};

/// Owns the bridge task; sessions get senders to publish into it.
pub struct Iap2Bridge {
    phone_tx: mpsc::UnboundedSender<CarPlayServerSessionEventTx>,
    head_unit_tx: mpsc::UnboundedSender<CarPlayClientSessionEventTx>,
    _task: AbortOnDropHandle<()>,
}

impl Iap2Bridge {
    /// Must be called inside the tokio runtime.
    pub fn start() -> Self {
        let (phone_tx, phone_rx) = mpsc::unbounded();
        let (head_unit_tx, head_unit_rx) = mpsc::unbounded();
        let task = spawn(run(phone_rx, head_unit_rx));
        Self {
            phone_tx,
            head_unit_tx,
            _task: task,
        }
    }

    pub fn phone_events(&self) -> mpsc::UnboundedSender<CarPlayServerSessionEventTx> {
        self.phone_tx.clone()
    }

    pub fn head_unit_events(&self) -> mpsc::UnboundedSender<CarPlayClientSessionEventTx> {
        self.head_unit_tx.clone()
    }
}

async fn run(
    mut phone: mpsc::UnboundedReceiver<CarPlayServerSessionEventTx>,
    mut head_unit: mpsc::UnboundedReceiver<CarPlayClientSessionEventTx>,
) {
    let mut now_playing = NowPlayingRelay::default();
    let mut head_unit_handle: Option<CsmClientHandleRef> = None;
    // The iPhone repeats NowPlaying several times a second; only title changes are logged at info
    let mut logged_title: Option<String> = None;

    loop {
        select! {
            ev = phone.next() => match ev {
                Some(CarPlayServerSessionEventTx::NowPlayingMerged(np)) => {
                    let out = now_playing.phone_now_playing(np);
                    forward(&mut head_unit_handle, out, &mut logged_title);
                }
                Some(CarPlayServerSessionEventTx::SessionStarted(_)) => info!("iAP2 bridge: iPhone session started"),
                Some(CarPlayServerSessionEventTx::SessionEnded) => {
                    info!("iAP2 bridge: iPhone session ended");
                    let out = now_playing.phone_ended();
                    forward(&mut head_unit_handle, out, &mut logged_title);
                }
                // Cover art and GPS are not bridged
                Some(_) => {}
                None => break,
            },
            ev = head_unit.next() => match ev {
                Some(CarPlayClientSessionEventTx::SessionStarted(handle)) => {
                    info!("iAP2 bridge: head unit session started");
                    head_unit_handle = Some(handle);
                    now_playing.head_unit_started();
                }
                Some(CarPlayClientSessionEventTx::NowPlayingRequested) => {
                    info!("iAP2 bridge: head unit subscribed to NowPlaying");
                    logged_title = None;
                    let out = now_playing.head_unit_requested();
                    forward(&mut head_unit_handle, out, &mut logged_title);
                }
                Some(CarPlayClientSessionEventTx::NowPlayingStopped) => {
                    info!("iAP2 bridge: head unit unsubscribed from NowPlaying");
                    now_playing.head_unit_stopped();
                }
                Some(CarPlayClientSessionEventTx::SessionEnded) => {
                    info!("iAP2 bridge: head unit session ended");
                    head_unit_handle = None;
                    now_playing.head_unit_ended();
                }
                None => break,
            },
        }
    }
}

/// Sends to the head unit. A closed handle is dropped; this never fails either session.
fn forward(handle: &mut Option<CsmClientHandleRef>, np: Option<NowPlayingUpdate>, logged_title: &mut Option<String>) {
    let (Some(h), Some(np)) = (handle.as_ref(), np) else {
        return;
    };
    let title = np
        .media_item
        .as_ref()
        .and_then(|m| m.title.clone())
        .unwrap_or_default();
    match h.send(&np) {
        Ok(()) if logged_title.as_ref() == Some(&title) => debug!("iAP2 bridge: NowPlaying to head unit: {title:?}"),
        Ok(()) => {
            info!("iAP2 bridge: NowPlaying to head unit: {title:?}");
            *logged_title = Some(title);
        }
        Err(err) => {
            warn!("iAP2 bridge: head unit unreachable, dropping its handle: {err}");
            handle.take();
        }
    }
}

/// NowPlaying from the iPhone to the head unit.
#[derive(Default)]
pub struct NowPlayingRelay {
    subscribed: bool,
    latest: Option<NowPlayingUpdate>,
}

impl NowPlayingRelay {
    pub fn head_unit_started(&mut self) {
        self.subscribed = false;
    }

    /// The head unit asked for NowPlaying: it gets the latest known state, if any.
    pub fn head_unit_requested(&mut self) -> Option<NowPlayingUpdate> {
        self.subscribed = true;
        self.latest.clone()
    }

    pub fn head_unit_ended(&mut self) {
        self.subscribed = false;
    }

    /// The head unit sent StopNowPlayingUpdates: nothing more may be sent until it asks again.
    pub fn head_unit_stopped(&mut self) {
        self.subscribed = false;
    }

    /// A merged NowPlaying from the iPhone; forwarded when the head unit is subscribed.
    pub fn phone_now_playing(&mut self, mut np: NowPlayingUpdate) -> Option<NowPlayingUpdate> {
        // File transfers (artwork, queue list) exist on the phone link only; never announce them.
        if let Some(item) = np.media_item.as_mut() {
            item.artwork_file_transfer_id = None;
        }
        if let Some(playback) = np.playback_attributes.as_mut() {
            playback.playback_queue_list_transfer_id = None;
            if playback.playback_queue_list_available.is_some() {
                playback.playback_queue_list_available = Some(false);
            }
        }
        self.latest = Some(np.clone());
        self.subscribed.then_some(np)
    }

    /// The iPhone is gone: clear the head unit once, so it does not keep a stale title.
    pub fn phone_ended(&mut self) -> Option<NowPlayingUpdate> {
        let had_state = self.latest.take().is_some();
        (self.subscribed && had_state).then(now_playing_cleared)
    }
}

#[cfg(test)]
mod tests {
    use catplay_csm::msg::{MediaItem, NowPlayingUpdate, PlaybackAttributes};

    use super::*;

    fn song(title: &str) -> NowPlayingUpdate {
        NowPlayingUpdate {
            media_item: Some(MediaItem {
                title: Some(title.into()),
                artwork_file_transfer_id: Some(7),
                ..MediaItem::default()
            }),
            playback_attributes: None,
        }
    }

    fn title(np: &NowPlayingUpdate) -> String {
        np.media_item
            .as_ref()
            .and_then(|m| m.title.clone())
            .unwrap_or_default()
    }

    #[test]
    fn queue_list_transfer_is_stripped() {
        let mut r = NowPlayingRelay::default();
        r.head_unit_started();
        r.head_unit_requested();
        let mut np = song("A");
        np.playback_attributes = Some(PlaybackAttributes {
            playback_queue_list_available: Some(true),
            playback_queue_list_transfer_id: Some(5),
            ..PlaybackAttributes::default()
        });
        let sent = r
            .phone_now_playing(np)
            .unwrap()
            .playback_attributes
            .unwrap();
        assert_eq!(sent.playback_queue_list_transfer_id, None);
        assert_eq!(sent.playback_queue_list_available, Some(false));
    }

    #[test]
    fn head_unit_stop_stops_forwarding() {
        let mut r = NowPlayingRelay::default();
        r.head_unit_started();
        r.head_unit_requested();
        r.head_unit_stopped();
        assert!(r.phone_now_playing(song("A")).is_none());
        assert!(r.phone_ended().is_none(), "no reset after the head unit unsubscribed");
    }

    #[test]
    fn late_subscriber_gets_latest_state() {
        let mut r = NowPlayingRelay::default();
        r.head_unit_started();
        assert!(r.phone_now_playing(song("A")).is_none(), "not subscribed yet");
        let sent = r.head_unit_requested().expect("latest state on subscribe");
        assert_eq!(title(&sent), "A");
    }

    #[test]
    fn subscribed_head_unit_gets_every_update() {
        let mut r = NowPlayingRelay::default();
        r.head_unit_started();
        assert!(r.head_unit_requested().is_none(), "nothing known yet");
        assert_eq!(title(&r.phone_now_playing(song("A")).unwrap()), "A");
        assert_eq!(title(&r.phone_now_playing(song("B")).unwrap()), "B");
    }

    #[test]
    fn artwork_id_is_stripped() {
        let mut r = NowPlayingRelay::default();
        r.head_unit_started();
        r.head_unit_requested();
        let sent = r.phone_now_playing(song("A")).unwrap();
        assert_eq!(sent.media_item.unwrap().artwork_file_transfer_id, None);
        let again = r.head_unit_requested().unwrap();
        assert_eq!(again.media_item.unwrap().artwork_file_transfer_id, None);
    }

    #[test]
    fn phone_loss_clears_head_unit_once() {
        let mut r = NowPlayingRelay::default();
        r.head_unit_started();
        r.head_unit_requested();
        r.phone_now_playing(song("A"));
        let cleared = r.phone_ended().expect("cleared state sent");
        assert_eq!(title(&cleared), "");
        assert!(r.phone_ended().is_none(), "nothing to clear twice");
        assert!(r.head_unit_requested().is_none(), "state forgotten");
    }

    #[test]
    fn phone_loss_without_data_sends_nothing() {
        let mut r = NowPlayingRelay::default();
        r.head_unit_started();
        r.head_unit_requested();
        assert!(r.phone_ended().is_none());
    }

    #[test]
    fn phone_reconnect_resumes_forwarding() {
        let mut r = NowPlayingRelay::default();
        r.head_unit_started();
        r.head_unit_requested();
        r.phone_now_playing(song("A"));
        r.phone_ended();
        assert_eq!(title(&r.phone_now_playing(song("B")).unwrap()), "B");
    }

    #[test]
    fn head_unit_loss_stops_forwarding_until_it_subscribes_again() {
        let mut r = NowPlayingRelay::default();
        r.head_unit_started();
        r.head_unit_requested();
        r.head_unit_ended();
        assert!(r.phone_now_playing(song("A")).is_none());
        r.head_unit_started();
        assert_eq!(title(&r.head_unit_requested().unwrap()), "A");
    }
}
