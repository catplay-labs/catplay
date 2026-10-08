use std::time::Instant;

use async_trait::async_trait;
use catplay_csm::{
    decoder::{AsCsmPacket, CsmFlag, CsmPacketBox, CsmPacketId, CsmPacketUtil},
    files::FileTransferSetupMetaType,
    msg::*,
};
use catplay_iap2_client::{CsmClientHandleRef, CsmSession, CsmSessionResult};
use catplay_util::mpsc;
use log::debug;

/// What the head-unit side tells the rest of the dongle.
pub enum CarPlayClientSessionEventTx {
    /// The head unit's iAP2 session is up; the handle can be used to send to the head unit.
    SessionStarted(CsmClientHandleRef),
    /// The head unit asked for NowPlaying updates.
    NowPlayingRequested,
    /// The head unit no longer wants NowPlaying updates.
    NowPlayingStopped,
    /// The head unit's iAP2 session is gone.
    SessionEnded,
}

/// NowPlaying that tells the head unit nothing is playing (no artwork).
pub fn now_playing_cleared() -> NowPlayingUpdate {
    NowPlayingUpdate {
        media_item: Some(MediaItem {
            persistent_id: Some(0),
            title: Some("".into()),
            playback_duration_in_ms: Some(0),
            album_title: Some("".into()),
            album_track_number: Some(0),
            album_track_count: Some(0),
            album_disc_number: Some(0),
            album_disc_count: Some(0),
            artist: Some("".into()),
            genre: Some("".into()),
            composer: Some("".into()),
            is_like_supported: Some(false),
            is_ban_supported: Some(false),
            is_liked: Some(false),
            is_banned: Some(false),
            is_resident_on_device: None,
            artwork_file_transfer_id: None,
            chapter_count: Some(0),
            ..MediaItem::default()
        }),
        playback_attributes: Some(PlaybackAttributes {
            playback_status: Some(PlaybackStatus::Stopped),
            playback_elapsed_time_ms: Some(0),
            playback_queue_index: Some(0),
            playback_queue_count: Some(0),
            playback_queue_chapter_index: Some(0),
            playback_shuffle_mode: Some(PlaybackShuffle::Off),
            playback_repeat_mode: Some(PlaybackRepeat::Off),
            playback_app_name: Some("".into()),
            pb_media_library_unique_identifier: Some("".into()),
            pb_apple_music_radio_ad: Some(false),
            pb_apple_music_radio_station_name: Some("".into()),
            pb_apple_music_radio_station_media_playlist_id: Some(0),
            playback_speed: Some(0),
            set_elapsed_time_available: Some(false),
            playback_queue_list_available: Some(false),
            playback_queue_list_transfer_id: None,
            playback_app_bundle_id: Some("".into()),
            ..PlaybackAttributes::default()
        }),
    }
}

pub struct CarPlayClientSession {
    power_sub: bool,
    power_draw: Option<u16>,
    flushed_power: bool,
    start: Option<Instant>,
    artwork_tx: Option<u8>,
    events: Option<mpsc::UnboundedSender<CarPlayClientSessionEventTx>>,
}

impl Default for CarPlayClientSession {
    fn default() -> Self {
        Self::new(None)
    }
}

impl CarPlayClientSession {
    pub fn new(events: Option<mpsc::UnboundedSender<CarPlayClientSessionEventTx>>) -> Self {
        Self {
            power_sub: false,
            power_draw: None,
            flushed_power: false,
            start: None,
            artwork_tx: None,
            events,
        }
    }

    fn publish(&self, event: CarPlayClientSessionEventTx) {
        if let Some(events) = &self.events {
            let _ = events.unbounded_send(event);
        }
    }

    pub fn send(&mut self, packet: &dyn AsCsmPacket, handle: &CsmClientHandleRef) -> CsmSessionResult<()> {
        debug!(
            "<- Outgoing packet @ {:?}: {:?}",
            Instant::now() - self.start.unwrap(),
            packet.as_csm()
        );

        handle.send(packet)
    }

    pub fn send_now_playing(&mut self, handle: CsmClientHandleRef) -> CsmSessionResult<()> {
        let first_tx = handle.send_file_reserve();
        let next_tx = handle.send_file_reserve();

        let reset = [
            {
                let mut np = now_playing_cleared();
                if let Some(item) = np.media_item.as_mut() {
                    item.artwork_file_transfer_id = first_tx;
                }
                np
            },
            NowPlayingUpdate {
                media_item: None,
                playback_attributes: Some(PlaybackAttributes {
                    playback_status: Some(PlaybackStatus::Stopped),
                    playback_elapsed_time_ms: Some(0),
                    playback_queue_index: Some(0),
                    playback_queue_count: Some(0),
                    playback_queue_chapter_index: Some(0),
                    playback_shuffle_mode: Some(PlaybackShuffle::Off),
                    playback_repeat_mode: Some(PlaybackRepeat::Off),
                    playback_app_name: Some("".into()),
                    pb_media_library_unique_identifier: Some("".into()),
                    pb_apple_music_radio_ad: Some(false),
                    pb_apple_music_radio_station_name: Some("".into()),
                    pb_apple_music_radio_station_media_playlist_id: Some(0),
                    playback_speed: Some(100),
                    set_elapsed_time_available: Some(false),
                    playback_app_bundle_id: Some("".into()),
                    ..PlaybackAttributes::default()
                }),
            },
            NowPlayingUpdate {
                media_item: Some(MediaItem {
                    persistent_id: Some(0),
                    title: Some("".into()),
                    playback_duration_in_ms: Some(0),
                    album_title: Some("".into()),
                    album_track_number: Some(0),
                    album_track_count: Some(0),
                    album_disc_number: Some(0),
                    album_disc_count: Some(0),
                    artist: Some("".into()),
                    genre: Some("".into()),
                    composer: Some("".into()),
                    is_like_supported: Some(false),
                    is_ban_supported: Some(false),
                    is_liked: Some(false),
                    is_banned: Some(false),
                    chapter_count: Some(0),
                    ..MediaItem::default()
                }),
                playback_attributes: None,
            },
            NowPlayingUpdate {
                media_item: None,
                playback_attributes: Some(PlaybackAttributes {
                    playback_queue_list_available: Some(false),
                    ..PlaybackAttributes::default()
                }),
            },
            NowPlayingUpdate {
                media_item: Some(MediaItem {
                    artwork_file_transfer_id: next_tx,
                    ..MediaItem::default()
                }),
                playback_attributes: None,
            },
        ];
        for i in reset {
            self.send(&i, &handle)?;
        }

        if let Some(first_tx) = first_tx {
            handle.send_file(first_tx, FileTransferSetupMetaType::NowPlayingArtworkData.into(), &[], Vec::new());
        }
        if let Some(next_tx) = next_tx {
            handle.send_file(next_tx, FileTransferSetupMetaType::NowPlayingArtworkData.into(), &[], Vec::new());
        }

        self.artwork_tx = first_tx;

        Ok(())
    }
}
#[async_trait]
impl CsmSession for CarPlayClientSession {
    async fn start(&mut self, _handle: CsmClientHandleRef) -> CsmSessionResult<()> {
        if self.start.is_none() {
            self.start.replace(Instant::now());
        }

        self.publish(CarPlayClientSessionEventTx::SessionStarted(_handle.clone()));
        self.send(&StartIdentification::default(), &_handle)?;
        Ok(())
    }

    async fn respond(&mut self, packet: CsmPacketBox, _handle: CsmClientHandleRef) -> CsmSessionResult<()> {
        if self.start.is_none() {
            self.start.replace(Instant::now());
        }
        debug!("-> Incoming packet @ {:?}: {:?}", Instant::now() - self.start.unwrap(), packet);

        if let Some(_id) = IdentificationInformation::cast(&packet) {
            let supports_modern_carplay =
                IdentificationInformation::unpack_ids(&_id.messages_sent_by_accessory).contains(&CarPlayStartSession::PACKET_ID);
            debug!("Identification accepted! Modern CarPlay: {supports_modern_carplay}");

            if supports_modern_carplay {
                // TODO
                // _handle.send(&CarPlayAvailability::default())?;
            }

            self.send(&IdentificationAccepted::default(), &_handle)?;
            self.send(&RequestAuthenticationCertificate::default(), &_handle)?;

            // _handle.send(&RequestAccessoryWiFiConfigurationInformation::default())?;
        }

        if let Some(_response) = packet.cast::<AuthenticationCertificate>() {
            self.send(
                &RequestAuthenticationChallengeResponse {
                    authentication_challenge: vec![0x42u8; 20].into(),
                },
                &_handle,
            )?;

            debug!("Received certificate!");
        }

        if let Some(_response) = packet.cast::<AuthenticationResponse>() {
            debug!("Authentication accepted!");
            self.send(&AuthenticationSucceeded::default(), &_handle)?;
            self.send(
                &DeviceUUIDUpdate {
                    uuid: "869556BE-4CE7-4602-9EB9-023B2A1E86FE".into(),
                },
                &_handle,
            )?;
            self.send(
                &DeviceInformationUpdate {
                    device_name: Some("iPhone SIM".into()),
                },
                &_handle,
            )?;
            // _handle.send(&DeviceTransportIdentifierNotification {
            //     usb_transport_identifier: Some("00008130000E044E384B1D3ADDDDDDDDDDDDDDDF".into()),
            //     bluetooth_transport_identifier: Some("aa:bb:cc:dd:ee:ff".into()),
            // })?;

            // _handle.send(&DeviceTimeUpdate {
            //     seconds_since_epoch: 1768172567,
            //     timezone_offset_minutes: 0,
            //     daylight_savings_offset_minutes: 0,
            // })?;
            self.send(
                &StartLocationInformation {
                    global_positioning_system_fix_data: CsmFlag::Yes,
                    recommended_minimum_specific_gps_transit_data: CsmFlag::Yes,
                    gps_satellites_in_view: CsmFlag::No,
                    vehicle_speed_data: CsmFlag::Yes,
                    vehicle_gyro_data: CsmFlag::No,
                    vehicle_accelerometer_data: CsmFlag::No,
                    vehicle_heading_data: CsmFlag::No,
                },
                &_handle,
            )?;
            self.send(&StopLocationInformation {}, &_handle)?;
            /*
                       NowPlayingUpdate { media_item: Some(MediaItem { persistent_id: None, title: None, media_type: [], rating: None, playback_duration_in_ms: None, album_persistent_id: None, album_title: None, album_track_number: None, album_track_count: None, album_disc_number: None, album_disc_count: None, artist_persistent_id: None, artist: None, album_artist_persistent_id: None, album_artist: None, genre_persistent_id: None, genre: None, composer_persistent_id: None, composer: None, is_part_of_compilation: None, is_like_supported: None, is_ban_supported: None, is_liked: None, is_banned: None, is_resident_on_device: None, artwork_file_transfer_id: Some(129), chapter_count: None }), playback_attributes: None }
            */
            // _handle.send(&WirelessCarPlayUpdate { available: false })?;

            // _handle.send(&StartExternalAccessoryProtocolSession {
            //     external_accessory_protocol_identifier: 1,
            //     external_accessory_protocol_session_identifier: 3,
            // })?;
        };

        if let Some(awci) = packet.cast::<AccessoryWiFiConfigurationInformation>() {
            debug!("Got Wi-Fi details: {awci:?}");
        }

        if let Some(snpu) = packet.cast::<StartNowPlayingUpdates>() {
            debug!("Got now playing request??: {snpu:?}");
            self.send_now_playing(_handle.clone())?;
            self.publish(CarPlayClientSessionEventTx::NowPlayingRequested);
        }

        if packet.cast::<StopNowPlayingUpdates>().is_some() {
            self.publish(CarPlayClientSessionEventTx::NowPlayingStopped);
        }

        if let Some(_spu) = packet.cast::<StartPowerUpdates>() {
            debug!("Got StartPowerUpdates {_spu:?}");
            self.power_sub = true;
            if self.power_draw.is_none() {
                self.power_draw.replace(500);
            }

            if self.power_draw.is_some() {
                self.send(
                    &PowerUpdate {
                        maximum_current_drawn_from_accessory: Some(1000),
                        device_battery_will_charge_if_power_is_present: None,
                        ..PowerUpdate::default()
                    },
                    &_handle,
                )?;
                self.send(
                    &PowerUpdate {
                        maximum_current_drawn_from_accessory: None,
                        device_battery_will_charge_if_power_is_present: Some(true),
                        ..PowerUpdate::default()
                    },
                    &_handle,
                )?;
                self.flushed_power = true;
            }
        }

        if let Some(_pu) = packet.cast::<PowerSourceUpdate>() {
            self.power_draw
                .replace(_pu.available_current_for_device.unwrap_or(500)); // TODO proper fallback to prev power_draw

            if self.power_sub && !self.flushed_power {
                self.flushed_power = true;
                self.send(
                    &PowerUpdate {
                        maximum_current_drawn_from_accessory: Some(1000),
                        device_battery_will_charge_if_power_is_present: None,
                        ..PowerUpdate::default()
                    },
                    &_handle,
                )?;
                self.send(
                    &PowerUpdate {
                        maximum_current_drawn_from_accessory: None,
                        device_battery_will_charge_if_power_is_present: Some(true),
                        ..PowerUpdate::default()
                    },
                    &_handle,
                )?;
            }
        }

        if let Some(_scu) = packet.cast::<StartCallStateUpdates>() {
            debug!("Got call state request");
            self.send(&CallStateUpdate::default(), &_handle)?;
        }

        if let Some(_scu) = packet.cast::<StartListUpdates>() {
            debug!("Got call list request");
            self.send(
                &ListUpdate {
                    recents_list_available: Some(true),
                    recents_list_count: Some(0),
                    ..ListUpdate::default()
                },
                &_handle,
            )?;
        }

        if let Some(cpss) = CarPlayStartSession::cast(&packet) {
            // TODO: trigger connection to CarPlay server now
            debug!("Got modern CarPlay handshake: {cpss:?}");
        }

        Ok(())
    }
}

impl Drop for CarPlayClientSession {
    fn drop(&mut self) {
        self.publish(CarPlayClientSessionEventTx::SessionEnded);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Head-unit handle for tests that must not touch the link.
    struct NoLinkHandle(catplay_iap2_client::CsmRemote);

    impl catplay_iap2_client::CsmClientHandle for NoLinkHandle {
        fn disconnect(&self) {}
        fn is_server(&self) -> bool {
            false
        }
        fn is_closed(&self) -> bool {
            false
        }
        fn is_writable(&self) -> bool {
            true
        }
        fn remote(&self) -> &catplay_iap2_client::CsmRemote {
            &self.0
        }
        fn status(&self) -> catplay_iap2_client::CsmSessionStatus {
            Default::default()
        }
        fn send(&self, _packet: &dyn AsCsmPacket) -> CsmSessionResult<()> {
            panic!("unexpected send to the head unit")
        }
        fn send_all(&self, _packets: &[CsmPacketBox]) -> CsmSessionResult<()> {
            panic!("unexpected send to the head unit")
        }
        fn send_file_reserve(&self) -> Option<u8> {
            None
        }
        fn send_file(&self, _file_id: u8, _file_type: u16, _setup_data: &[u8], _source: Vec<u8>) {}
    }

    #[test]
    fn stop_now_playing_updates_publishes_now_playing_stopped() {
        let (tx, mut rx) = catplay_util::mpsc::unbounded();
        let mut session = CarPlayClientSession::new(Some(tx));
        let handle: CsmClientHandleRef = std::sync::Arc::new(NoLinkHandle(catplay_iap2_client::CsmRemote::airplay()));

        futures::executor::block_on(session.respond(StopNowPlayingUpdates {}.into(), handle)).unwrap();

        assert!(matches!(rx.try_next(), Ok(Some(CarPlayClientSessionEventTx::NowPlayingStopped))));
    }

    #[test]
    fn cleared_now_playing_is_stopped_empty_and_has_no_artwork() {
        let np = now_playing_cleared();
        let item = np.media_item.expect("media item");
        assert_eq!(item.title, Some("".into()));
        assert_eq!(item.artwork_file_transfer_id, None);
        assert_eq!(
            np.playback_attributes.expect("attributes").playback_status,
            Some(PlaybackStatus::Stopped)
        );
    }

    #[test]
    fn dropping_session_publishes_session_ended() {
        let (tx, mut rx) = catplay_util::mpsc::unbounded();
        drop(CarPlayClientSession::new(Some(tx)));
        assert!(matches!(rx.try_next(), Ok(Some(CarPlayClientSessionEventTx::SessionEnded))));
    }
}
