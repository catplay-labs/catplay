use std::{fmt, time::Instant};

use async_trait::async_trait;
use catplay_iap2_client::{CsmClientHandleRef, CsmFileTransferEvent, CsmSession, CsmSessionResult, tokio::ClientSessionHelper};
use catplay_mfi::MfiDeficeRef;
use catplay_util::mpsc;
use log::{debug, info, warn};
use macaddr::MacAddr6;

use catplay_csm::{
    decoder::{CsmFlag, CsmPacketBox, CsmPacketId},
    msg::*,
};

/// A CSM session to connect with iPhone's iAP2 over USB, Bluetooth(Wi-Fi details advertisement only), or AirPlay RTSP transports.
pub struct CarPlayServerSession {
    mfi: Option<MfiDeficeRef>,
    identity: CarPlaySessionIdentity,
    start: Instant,

    id: IdentificationInformation,
    wifi: Option<AccessoryWiFiConfigurationInformation>,

    now_playing: Option<NowPlayingUpdate>,
    now_playing_summary: NowPlayingSummary,
    now_playing_next_cover_art: Option<u8>,
    cover_art_transfer_start: Option<Instant>,

    gps_sub: bool,

    events_rx: mpsc::UnboundedReceiver<CarPlayServerSessionEventRx>,
    events_tx: mpsc::UnboundedSender<CarPlayServerSessionEventTx>,
}

#[derive(Debug, Clone)]
pub struct CarPlaySessionIdentity {
    pub name: String,             //  "MB Infotainment"
    pub model_identifier: String, // "MB_B-H-3"
    pub manufacturer: String,     // "Mercedes-Benz"
    pub serial_number: String,
    pub firmware_version: String,
    pub hardware_version: String,
    pub display_name: String, //  "MB Infotainment 29172"

    // USB transport (value should be 1; 1st interface on the gadget)
    pub ncm_iface: Option<u8>,

    // Wireless transport
    pub bt_mac: Option<MacAddr6>,
    pub wifi_ssid: Option<String>,
    pub wifi_passphrase: Option<String>,
    pub wifi_is_wpa: bool,
    pub wifi_channel: Option<u8>,

    /// Whether current session will be USB or Bluetooth/AirPlay one; some operations are illegal depending on the mode and will kill the connection.
    pub is_usb_transport: bool,

    pub has_gps: bool,

    /// Whether NowPlaying should be subscribed; useless for a temporary Bluetooth connection during CarPlay pairing.
    pub wants_now_playing: bool,
}

impl Default for CarPlaySessionIdentity {
    fn default() -> Self {
        Self {
            name: "CatPlay".into(),
            model_identifier: "CatPlay".into(),
            manufacturer: "CatPlay".into(),
            serial_number: "123456789".into(),
            firmware_version: "1.0".into(),
            hardware_version: "1.0".into(),
            display_name: "CatPlay HU".into(),
            ncm_iface: None,
            bt_mac: None,
            wifi_ssid: None,
            wifi_passphrase: None,
            wifi_is_wpa: true,
            wifi_channel: None,
            is_usb_transport: false,
            has_gps: false,
            wants_now_playing: false,
        }
    }
}

#[allow(clippy::large_enum_variant)]
pub enum CarPlayServerSessionEventTx {
    /// iPhone requests regular GPS updates.
    StartLocationInformation(StartLocationInformation),
    /// iPhone no longer needs regular GPS updates.
    StopLocationInformat(StopLocationInformation),

    /// Current status of NowPlaying; deltas are already automatically merged for convenience.
    NowPlayingMerged(NowPlayingUpdate),
    /// Transfer of new cover art was started.
    ///
    /// User can decide if the previous cover art should be shown temporarly or a placeholder should be shown instead.
    CoverArtInvalidated,
    /// Transfer of cover art has failed(for example, the size was insane).
    /// No automatic retries are expected until the current track changes.
    CoverArtFailure,
    /// JPEG data of latest cover art.
    ///
    /// Empty buffer may mean "no cover art available" or that it's being fetched from the internet and a real one will follow.
    CoverArt(Vec<u8>),
}

pub enum CarPlayServerSessionEventRx {
    /// Feed GPS GPRMC data.
    FeedGpsGprmc(GPRMCDataStatusValuesNotification),
    /// Feed GPS NMEA data.
    FeedGpsNmea(LocationInformation),
}

#[derive(Default, Eq, PartialEq)]
struct NowPlayingSummary {
    title: Option<String>,
    artists: Option<String>,
    album: Option<String>,
}

impl NowPlayingSummary {
    fn from_update(update: &NowPlayingUpdate) -> Self {
        let Some(media_item) = &update.media_item else {
            return Self::default();
        };

        Self {
            title: media_item.title.clone(),
            artists: media_item.artist.clone(),
            album: media_item.album_title.clone(),
        }
    }
}

impl fmt::Display for NowPlayingSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let title = self.title.as_deref().filter(|s| !s.is_empty());
        let artists = self.artists.as_deref().filter(|s| !s.is_empty());
        let album = self.album.as_deref().filter(|s| !s.is_empty());

        match (title, artists, album) {
            (Some(title), Some(artists), Some(album)) => write!(f, "{title} - {artists} ({album})"),
            (Some(title), Some(artists), None) => write!(f, "{title} - {artists}"),
            (Some(title), None, Some(album)) => write!(f, "{title} ({album})"),
            (Some(title), None, None) => f.write_str(title),
            (None, Some(artists), Some(album)) => write!(f, "{artists} ({album})"),
            (None, Some(artists), None) => f.write_str(artists),
            (None, None, Some(album)) => f.write_str(album),
            (None, None, None) => f.write_str("<unknown>"),
        }
    }
}

impl CarPlayServerSession {
    fn publish(&mut self, event: CarPlayServerSessionEventTx) {
        let _ = self.events_tx.unbounded_send(event);
    }

    pub fn on_event(&mut self, ev: CarPlayServerSessionEventRx, _handle: CsmClientHandleRef) {
        match ev {
            CarPlayServerSessionEventRx::FeedGpsGprmc(packet) => {
                if self.gps_sub {
                    _handle.send(&packet);
                } else {
                    warn!("Ignoring GPRMC because GPS sub is inactive");
                }
            }
            CarPlayServerSessionEventRx::FeedGpsNmea(packet) => {
                if self.gps_sub {
                    _handle.send(&packet);
                } else {
                    warn!("Ignoring NMEA because GPS sub is inactive");
                }
            }
        }
    }

    pub fn on_now_playing(&mut self, np: NowPlayingUpdate) {
        if self.now_playing.is_none() {
            self.now_playing.replace(np);
        } else {
            if let Some(Some(artwork_file_transfer_id)) = np.media_item.as_ref().map(|m| m.artwork_file_transfer_id)
                && Some(artwork_file_transfer_id) != self.now_playing_next_cover_art
            {
                info!("Cover art invalidated; waiting for transfer ID {artwork_file_transfer_id}");
                self.publish(CarPlayServerSessionEventTx::CoverArtInvalidated);

                self.now_playing_next_cover_art
                    .replace(artwork_file_transfer_id);
                self.cover_art_transfer_start.replace(Instant::now());
            }

            let latest = self.now_playing.as_mut().unwrap();
            latest.deep_merge(np);
            debug!("Now playing: {latest:?}");
        }

        if let Some(now_playing) = &self.now_playing {
            let summary = NowPlayingSummary::from_update(now_playing);
            if summary != self.now_playing_summary {
                info!("Now playing summary: {summary}");
                self.now_playing_summary = summary;
            }
        }

        self.publish(CarPlayServerSessionEventTx::NowPlayingMerged(self.now_playing.clone().unwrap()));
    }

    pub fn new(
        mfi: Option<MfiDeficeRef>,
        identity: CarPlaySessionIdentity,
    ) -> (
        Self,
        mpsc::UnboundedSender<CarPlayServerSessionEventRx>,
        mpsc::UnboundedReceiver<CarPlayServerSessionEventTx>,
    ) {
        let (events_tx0, events_rx0) = mpsc::unbounded();
        let (events_tx1, events_rx1) = mpsc::unbounded();

        (
            Self {
                mfi,
                start: Instant::now(),
                id: Self::id(&identity),
                wifi: Self::wifi(&identity),
                identity,
                now_playing: None,
                now_playing_summary: NowPlayingSummary::default(),
                now_playing_next_cover_art: None,
                cover_art_transfer_start: None,
                events_rx: events_rx0,
                events_tx: events_tx1,
                gps_sub: false,
            },
            events_tx0,
            events_rx1,
        )
    }

    pub fn id(identity: &CarPlaySessionIdentity) -> IdentificationInformation {
        let mut id = IdentificationInformation {
            name: identity.name.clone(),
            model_identifier: identity.model_identifier.clone(),
            manufacturer: identity.manufacturer.clone(),
            serial_number: identity.serial_number.clone(),
            firmware_version: identity.firmware_version.clone(),
            hardware_version: identity.hardware_version.clone(),
            ..IdentificationInformation::default()
        };

        fn flatten_wants(groups: &[(bool, &[u16])]) -> Vec<u16> {
            let len = groups
                .iter()
                .filter_map(|(want, ids)| want.then_some(ids.len()))
                .sum();
            let mut out = Vec::with_capacity(len);
            for (want, ids) in groups {
                if *want {
                    out.extend_from_slice(ids);
                }
            }
            out
        }

        let wants_tx = [
            (
                true,
                &[
                    StartNowPlayingUpdates::PACKET_ID,
                    StopNowPlayingUpdates::PACKET_ID,
                    SetNowPlayingInformation::PACKET_ID,
                    StartCallStateUpdates::PACKET_ID,
                    StopCallStateUpdates::PACKET_ID,
                    InitiateCall::PACKET_ID,
                    StartListUpdates::PACKET_ID,
                    StopListUpdates::PACKET_ID,
                ][..],
            ),
            /* Illegal if GPS is not declared */
            (identity.has_gps, &[LocationInformation::PACKET_ID][..]),
            /* Illegal if live transport is not USB */
            (
                identity.is_usb_transport,
                &[
                    StartPowerUpdates::PACKET_ID,
                    StopPowerUpdates::PACKET_ID,
                    PowerSourceUpdate::PACKET_ID,
                ][..],
            ),
            /* Illegal if Wi-Fi capability is not declared */
            (
                identity.wifi_ssid.is_some(),
                &[AccessoryWiFiConfigurationInformation::PACKET_ID][..],
            ),
        ];

        let wants_rx = [
            (
                true,
                &[
                    NowPlayingUpdate::PACKET_ID,
                    DeviceInformationUpdate::PACKET_ID,
                    DeviceUUIDUpdate::PACKET_ID,
                    CallStateUpdate::PACKET_ID,
                    ListUpdate::PACKET_ID,
                ][..],
            ),
            /* Illegal if GPS is not declared */
            (
                identity.has_gps,
                &[StartLocationInformation::PACKET_ID, StopLocationInformation::PACKET_ID][..],
            ),
            /* Illegal if EA session is not declared */
            (
                false,
                &[
                    StartExternalAccessoryProtocolSession::PACKET_ID,
                    StopExternalAccessoryProtocolSession::PACKET_ID,
                ][..],
            ),
            /* Illegal if Wi-Fi capability is not declared */
            (
                identity.wifi_ssid.is_some(),
                &[RequestAccessoryWiFiConfigurationInformation::PACKET_ID][..],
            ),
        ];

        id.messages_sent_by_accessory = IdentificationInformation::pack_ids(&flatten_wants(&wants_tx));
        id.messages_received_from_device = IdentificationInformation::pack_ids(&flatten_wants(&wants_rx));
        id.power_providing_capability = PowerProvidingCapability::Advanced;
        id.maximum_current_drawn_from_device = 0u16;
        id.supported_language = vec!["en".into()];
        id.current_language = "en".into();
        id.vehicle_information_component = Some(VehicleInformationComponent {
            identifier: 1,
            name: "VEH_INFO_USE".into(),
            display_name: identity.display_name.clone(),
            engine_type: EngineTypes::Gasoline,
        });

        if identity.has_gps {
            id.location_information_component = Some(LocationInformationComponent {
                identifier: 4001,
                name: "LOC_USE".into(),
                global_positioning_system_fix_data: CsmFlag::Yes,
                recommended_minimum_specific_gpstransit_data: CsmFlag::Yes,
                gpssatellites_in_view: CsmFlag::No,
                vehicle_speed_data: CsmFlag::Yes,
                vehicle_gyro_data: CsmFlag::No,
                vehicle_accelerometer_data: CsmFlag::No,
                vehicle_heading_data: CsmFlag::No,
            });
        }

        if let Some(ncm_iface) = identity.ncm_iface {
            id.usbhost_transport_component = Some(USBHostTransportComponent {
                transport_component_identifier: 1001,
                transport_component_name: "USB_USE".into(),
                transport_supports_iap2_connection: CsmFlag::Yes,
                usbhost_transport_car_play_interface_number: Some(ncm_iface),
                transport_supports_car_play: CsmFlag::Yes,
            });
        }

        if let Some(bt_mac) = identity.bt_mac {
            let mac_bytes = bt_mac.as_bytes().into();

            id.bluetooth_transport_component = vec![BluetoothTransportComponent {
                transport_component_identifier: 1,
                transport_component_name: "IAP2-Bluetooth".into(),
                transport_supports_iap2_connection: CsmFlag::Yes,
                bluetooth_transport_mac_address: mac_bytes,
            }];
        }

        if identity.wifi_ssid.is_some() {
            id.wireless_car_play_transport_component = vec![WirelessCarPlayTransportComponent {
                transport_component_identifier: 2,
                transport_component_name: "IAP2-Wireless".into(),
                transport_supports_iap2_connection: CsmFlag::Yes,
                transport_supports_car_play: CsmFlag::Yes,
                transport_supports_mutual_auth: CsmFlag::No,
            }];
        }

        id
    }

    pub fn wifi(identity: &CarPlaySessionIdentity) -> Option<AccessoryWiFiConfigurationInformation> {
        let wifi_ssid = identity.wifi_ssid.clone()?;

        Some(AccessoryWiFiConfigurationInformation {
            wifi_ssid: wifi_ssid.clone(),
            passphrase: identity.wifi_passphrase.clone(),
            security_type: match (identity.wifi_is_wpa, identity.wifi_passphrase.clone()) {
                (true, Some(_)) => Some(WiFiSecurityType::WpaOrWpa2),
                (false, Some(_)) => Some(WiFiSecurityType::WEP),
                _ => Some(WiFiSecurityType::None),
            },
            channel: identity.wifi_channel,
        })
    }
}

#[async_trait]
impl CsmSession for CarPlayServerSession {
    async fn start(&mut self, _handle: CsmClientHandleRef) -> CsmSessionResult<()> {
        Ok(())
    }

    async fn on_file_event(&mut self, id: u8, ev: CsmFileTransferEvent, _handle: CsmClientHandleRef) -> CsmSessionResult<()> {
        if Some(id) != self.now_playing_next_cover_art {
            debug!("Ignored event for file transfer id {id}");
            return Ok(());
        }

        match ev {
            CsmFileTransferEvent::Completed { data, size, .. } => {
                let transfer_start = self.cover_art_transfer_start.unwrap_or(Instant::now());
                info!("New cover art with size {size}; took {:?}", Instant::now() - transfer_start);
                self.publish(CarPlayServerSessionEventTx::CoverArt(data));
            }
            CsmFileTransferEvent::Cancelled => {
                warn!("Cover art transfer was cancelled at file id {id}");
                self.publish(CarPlayServerSessionEventTx::CoverArtFailure);
            }
            _ => {}
        }
        Ok(())
    }

    async fn respond(&mut self, packet: CsmPacketBox, _handle: CsmClientHandleRef) -> CsmSessionResult<()> {
        debug!("Received from iPhone @ {:?}: {:?}", Instant::now() - self.start, packet);

        // if let Some(_) = packet.cast::<IdentificationAccepted>() {
        //     _handle.send(&PowerSourceUpdate {
        //         available_current_for_device: Some(1000),
        //         device_battery_should_charge_if_power_is_present: Some(true),
        //     })?;
        // }

        if let Some(np) = packet.cast::<NowPlayingUpdate>() {
            self.on_now_playing(np.clone());
        }

        if let Some(start_location) = packet.cast::<StartLocationInformation>() {
            self.gps_sub = true;
            self.publish(CarPlayServerSessionEventTx::StartLocationInformation(start_location.clone()));
        }

        if let Some(stop_location) = packet.cast::<StopLocationInformation>() {
            self.gps_sub = false;
            self.publish(CarPlayServerSessionEventTx::StopLocationInformat(stop_location.clone()));
        }

        if packet.cast::<AuthenticationSucceeded>().is_some() {
            // _handle.send(&StartPowerUpdates {
            //     maximum_current_drawn_from_accessory: CsmFlag::Yes,
            //     device_battery_will_charge_if_power_is_present: CsmFlag::Yes,
            //     ..StartPowerUpdates::default()
            // })?;
            // _handle.send(&PowerSourceUpdate {
            //     available_current_for_device: Some(1000),
            //     device_battery_should_charge_if_power_is_present: Some(true),
            // })?;

            if self.identity.wants_now_playing {
                _handle.send(&StartNowPlayingUpdates::all())?;
            }
        }

        if ClientSessionHelper::handle_auth(self.mfi.clone(), &packet, _handle.clone()).await? {
            debug!("Handled auth");
            return Ok(());
        }

        let mut accepted = false;
        let mut responded_to_wifi = false;

        if ClientSessionHelper::handle_id(&packet, _handle.clone(), &mut accepted, &self.id).await? {
            debug!("Handled id");
            // if accepted {}
            return Ok(());
        }

        if let Some(wifi) = self.wifi.as_ref()
            && ClientSessionHelper::handle_wifi(&packet, _handle.clone(), &mut responded_to_wifi, wifi).await?
        {
            debug!("Handled wifi");
            return Ok(());
        }

        return Ok(());
    }
}
