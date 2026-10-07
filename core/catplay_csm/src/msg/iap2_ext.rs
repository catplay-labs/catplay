extern crate alloc;

use alloc::vec::Vec;

use crate::decoder::{CsmFlag, CsmPacked, CsmString};

use super::iap2::*;

impl StartLocationInformation {
    pub fn all() -> Self {
        Self {
            global_positioning_system_fix_data: CsmFlag::Yes,
            recommended_minimum_specific_gps_transit_data: CsmFlag::Yes,
            gps_satellites_in_view: CsmFlag::Yes,
            vehicle_speed_data: CsmFlag::Yes,
            vehicle_gyro_data: CsmFlag::Yes,
            vehicle_accelerometer_data: CsmFlag::Yes,
            vehicle_heading_data: CsmFlag::Yes,
            ..Self::default()
        }
    }
}

impl IdentificationInformation {
    pub fn pack_ids(ids: &[u16]) -> CsmPacked<u16> {
        ids.into()
    }

    pub fn unpack_ids(ids: &CsmPacked<u16>) -> Vec<u16> {
        ids.to_vec()
    }

    pub fn has_id(ids: &[u16], id: u16) -> bool {
        ids.contains(&id)
    }

    pub fn wants_rx(&self, id: u16) -> bool {
        Self::has_id(&self.messages_received_from_device, id)
    }

    pub fn wants_tx(&self, id: u16) -> bool {
        Self::has_id(&self.messages_sent_by_accessory, id)
    }
}

impl IdentificationRejected {
    pub fn reason(&self) -> CsmString {
        let mut reasons = Vec::new();

        push_flag(&mut reasons, "name", self.name);
        push_flag(&mut reasons, "model_identifier", self.model_identifier);
        push_flag(&mut reasons, "manufacturer", self.manufacturer);
        push_flag(&mut reasons, "serial_number", self.serial_number);
        push_flag(&mut reasons, "firmware_version", self.firmware_version);
        push_flag(&mut reasons, "hardware_version", self.hardware_version);
        push_present(
            &mut reasons,
            "messages_sent_by_accessory",
            self.messages_sent_by_accessory.is_some(),
        );
        push_present(
            &mut reasons,
            "messages_received_from_device",
            self.messages_received_from_device.is_some(),
        );
        push_flag(&mut reasons, "power_providing_capability", self.power_providing_capability);
        push_flag(
            &mut reasons,
            "maximum_current_drawn_from_device",
            self.maximum_current_drawn_from_device,
        );
        push_flag(
            &mut reasons,
            "supported_external_accessory_protocol",
            self.supported_external_accessory_protocol,
        );
        push_flag(&mut reasons, "app_match_team_id", self.app_match_team_id);
        push_flag(&mut reasons, "current_language", self.current_language);
        push_flag(&mut reasons, "supported_language", self.supported_language);
        push_flag(&mut reasons, "uart_transport_component", self.uart_transport_component);
        push_flag(&mut reasons, "usb_device_transport_component", self.usb_device_transport_component);
        push_flag(&mut reasons, "usb_host_transport_component", self.usb_host_transport_component);
        push_flag(&mut reasons, "bluetooth_transport_component", self.bluetooth_transport_component);
        push_present(&mut reasons, "iap2_hid_component", !self.iap2_hid_component.is_empty());
        push_flag(&mut reasons, "vehicle_information_component", self.vehicle_information_component);
        push_flag(&mut reasons, "vehicle_status_component", self.vehicle_status_component);
        push_flag(&mut reasons, "location_information_component", self.location_information_component);
        push_present(&mut reasons, "usb_host_hid_component", !self.usb_host_hid_component.is_empty());
        push_present(
            &mut reasons,
            "wireless_car_play_transport_component",
            !self.wireless_car_play_transport_component.is_empty(),
        );
        push_flag(&mut reasons, "ble_transport_component", self.ble_transport_component);
        push_flag(&mut reasons, "bluetooth_hid_component", self.bluetooth_hid_component);
        push_present(
            &mut reasons,
            "route_guidance_display_component",
            !self.route_guidance_display_component.is_empty(),
        );
        push_present(
            &mut reasons,
            "road_object_detection_component",
            !self.road_object_detection_component.is_empty(),
        );
        push_flag(&mut reasons, "product_plan_uid", self.product_plan_uid);
        push_present(
            &mut reasons,
            "maximum_current_drawn_in_uhpm",
            self.maximum_current_drawn_in_uhpm.is_some(),
        );
        push_flag(&mut reasons, "power_during_sleep", self.power_during_sleep);

        reasons.join(",")
    }
}

fn push_flag(reasons: &mut Vec<&'static str>, name: &'static str, flag: CsmFlag) {
    push_present(reasons, name, flag == CsmFlag::Yes);
}

fn push_present(reasons: &mut Vec<&'static str>, name: &'static str, present: bool) {
    if present {
        reasons.push(name);
    }
}

impl StartNowPlayingUpdates {
    pub fn all() -> Self {
        Self {
            media_item_attributes: Some(StartNowPlayingUpdatesMediaItemAttributes::all()),
            playback_attributes: Some(StartNowPlayingUpdatesPlaybackAttributes::all()),
            playback_queue_list_content_transfer_info_request: Some(PlaybackQueueListContentTransferInfoRequest::all()),
        }
    }
}

impl StartNowPlayingUpdatesMediaItemAttributes {
    pub fn all() -> Self {
        Self {
            media_item_persistent_identifier: CsmFlag::Yes,
            media_item_title: CsmFlag::Yes,
            media_item_playback_duration_in_milliseconds: CsmFlag::Yes,
            media_item_album_title: CsmFlag::Yes,
            media_item_album_track_number: CsmFlag::Yes,
            media_item_album_track_count: CsmFlag::Yes,
            media_item_album_disc_number: CsmFlag::Yes,
            media_item_album_disc_count: CsmFlag::Yes,
            media_item_artist: CsmFlag::Yes,
            media_item_genre: CsmFlag::Yes,
            media_item_composer: CsmFlag::Yes,
            media_item_is_like_supported: CsmFlag::Yes,
            media_item_is_ban_supported: CsmFlag::Yes,
            media_item_is_liked: CsmFlag::Yes,
            media_item_is_banned: CsmFlag::Yes,
            media_item_artwork_file_transfer_identifier: CsmFlag::Yes,
            media_item_chapter_count: CsmFlag::Yes,
        }
    }
}

impl StartNowPlayingUpdatesPlaybackAttributes {
    pub fn all() -> Self {
        Self {
            playback_status: CsmFlag::Yes,
            playback_elapsed_time_in_milliseconds: CsmFlag::Yes,
            playback_queue_index: CsmFlag::Yes,
            playback_queue_count: CsmFlag::Yes,
            playback_queue_chapter_index: CsmFlag::Yes,
            playback_shuffle_mode: CsmFlag::Yes,
            playback_repeat_mode: CsmFlag::Yes,
            playback_app_name: CsmFlag::Yes,
            playback_media_library_unique_identifier: CsmFlag::Yes,
            playback_apple_music_radio_ad: CsmFlag::Yes,
            playback_apple_music_radio_station_name: CsmFlag::Yes,
            playback_apple_music_radio_station_media_playlist_id: CsmFlag::Yes,
            playback_speed: CsmFlag::Yes,
            playback_set_elapsed_time_available: CsmFlag::Yes,
            playback_queue_list_avail: CsmFlag::Yes,
            playback_queue_list_transfer_id: CsmFlag::Yes,
            playback_app_bundle_id: CsmFlag::Yes,
            playback_queue_list_content_transfer_size: Some(0),
        }
    }
}

impl PlaybackQueueListContentTransferInfoRequest {
    pub fn all() -> Self {
        Self {
            media_item_pid: CsmFlag::Yes,
            media_item_title: CsmFlag::Yes,
            media_item_album_title: CsmFlag::Yes,
            media_item_artist: CsmFlag::Yes,
            media_item_album_artist: CsmFlag::Yes,
            media_item_genre: CsmFlag::Yes,
            media_item_composer: CsmFlag::Yes,
        }
    }
}

impl NowPlayingUpdate {
    pub fn deep_merge(&mut self, update: Self) {
        if let Some(media_item_attributes) = update.media_item_attributes {
            match &mut self.media_item_attributes {
                Some(current) => merge_media_item_attributes(current, media_item_attributes),
                None => self.media_item_attributes = Some(media_item_attributes),
            }
        }

        if let Some(playback_attributes) = update.playback_attributes {
            match &mut self.playback_attributes {
                Some(current) => merge_playback_attributes(current, playback_attributes),
                None => self.playback_attributes = Some(playback_attributes),
            }
        }
    }
}

macro_rules! merge_some_fields {
    ($current:expr, $update:expr, $($field:ident),+ $(,)?) => {
        $(
            if let Some(value) = $update.$field {
                $current.$field = Some(value);
            }
        )+
    };
}

fn merge_media_item_attributes(current: &mut NowPlayingUpdateMediaItemAttributes, update: NowPlayingUpdateMediaItemAttributes) {
    merge_some_fields!(
        current,
        update,
        media_item_persistent_identifier,
        media_item_title,
        media_item_playback_duration_in_milliseconds,
        media_item_album_title,
        media_item_album_track_number,
        media_item_album_track_count,
        media_item_album_disc_number,
        media_item_album_disc_count,
        media_item_artist,
        media_item_genre,
        media_item_composer,
        media_item_is_like_supported,
        media_item_is_ban_supported,
        media_item_is_liked,
        media_item_is_banned,
        media_item_artwork_file_transfer_identifier,
        media_item_chapter_count,
    );
}

fn merge_playback_attributes(current: &mut NowPlayingUpdatePlaybackAttributes, update: NowPlayingUpdatePlaybackAttributes) {
    merge_some_fields!(
        current,
        update,
        playback_status,
        playback_elapsed_time_in_milliseconds,
        playback_queue_index,
        playback_queue_count,
        playback_queue_chapter_index,
        playback_shuffle_mode,
        playback_repeat_mode,
        playback_app_name,
        playback_media_library_unique_identifier,
        playback_apple_music_radio_ad,
        playback_apple_music_radio_station_name,
        playback_apple_music_radio_station_media_playlist_id,
        playback_speed,
        playback_set_elapsed_time_available,
        playback_queue_list_avail,
        playback_queue_list_transfer_id,
        playback_app_bundle_id,
    );

    if update.playback_queue_list_content_transfer == CsmFlag::Yes {
        current.playback_queue_list_content_transfer = CsmFlag::Yes;
    }
}
