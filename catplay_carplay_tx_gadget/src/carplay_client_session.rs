use std::time::Instant;

use async_trait::async_trait;
use catplay_csm::{
    decoder::{AsCsmPacket, CsmFlag, CsmPacketBox, CsmPacketId, CsmPacketUtil},
    files::FileTransferSetupMetaType,
    msg::*,
};
use catplay_iap2_client::{CsmClientHandleRef, CsmSession, CsmSessionResult};
use catplay_lingo::{
    CommandKey, Iap1Message, lingo_0x0 as general,
    lingos::{LingoMessage, lingo_0x3 as display, lingo_0x4 as extended, lingo_0xa as audio},
    wire::{Outgoing, TransactionIntent},
};
use log::{debug, warn};

#[derive(Default)]
pub struct CarPlayClientSession {
    power_sub: bool,
    power_draw: Option<u16>,
    flushed_power: bool,
    start: Option<Instant>,
    artwork_tx: Option<u8>,
}

impl CarPlayClientSession {
    pub fn send(&mut self, packet: &dyn AsCsmPacket, handle: &CsmClientHandleRef) -> CsmSessionResult<()> {
        warn!(
            "<- Outgoing packet @ {:?}: {:?}",
            Instant::now() - self.start.unwrap(),
            packet.as_csm()
        );

        handle.send(packet)
    }

    fn send_legacy(handle: &CsmClientHandleRef, outgoing: Outgoing) -> CsmSessionResult<()> {
        warn!("iAP1 TX: {:?} transaction={:?}", outgoing.message, outgoing.transaction);
        handle.send_lingo(outgoing)
    }

    pub fn send_now_playing(&mut self, handle: CsmClientHandleRef) -> CsmSessionResult<()> {
        let first_tx = handle.send_file_reserve();
        let next_tx = handle.send_file_reserve();

        let reset = [
            NowPlayingUpdate {
                media_item_attributes: Some(NowPlayingUpdateMediaItemAttributes {
                    media_item_persistent_identifier: Some(0),
                    media_item_title: Some("".into()),
                    media_item_playback_duration_in_milliseconds: Some(0),
                    media_item_album_title: Some("".into()),
                    media_item_album_track_number: Some(0),
                    media_item_album_track_count: Some(0),
                    media_item_album_disc_number: Some(0),
                    media_item_album_disc_count: Some(0),
                    media_item_artist: Some("".into()),
                    media_item_genre: Some("".into()),
                    media_item_composer: Some("".into()),
                    media_item_is_like_supported: Some(false),
                    media_item_is_ban_supported: Some(false),
                    media_item_is_liked: Some(false),
                    media_item_is_banned: Some(false),
                    media_item_artwork_file_transfer_identifier: first_tx,
                    media_item_chapter_count: Some(0),
                    ..NowPlayingUpdateMediaItemAttributes::default()
                }),
                playback_attributes: Some(NowPlayingUpdatePlaybackAttributes {
                    playback_status: Some(PlaybackStatus::Stopped),
                    playback_elapsed_time_in_milliseconds: Some(0),
                    playback_queue_index: Some(0),
                    playback_queue_count: Some(0),
                    playback_queue_chapter_index: Some(0),
                    playback_shuffle_mode: Some(PlaybackShuffle::Off),
                    playback_repeat_mode: Some(PlaybackRepeat::Off),
                    playback_app_name: Some("".into()),
                    playback_media_library_unique_identifier: Some("".into()),
                    playback_apple_music_radio_ad: Some(false),
                    playback_apple_music_radio_station_name: Some("".into()),
                    playback_apple_music_radio_station_media_playlist_id: Some(0),
                    playback_speed: Some(0),
                    playback_set_elapsed_time_available: Some(false),
                    playback_queue_list_avail: Some(false),
                    playback_queue_list_transfer_id: None,
                    playback_app_bundle_id: Some("".into()),
                    ..NowPlayingUpdatePlaybackAttributes::default()
                }),
            },
            NowPlayingUpdate {
                media_item_attributes: None,
                playback_attributes: Some(NowPlayingUpdatePlaybackAttributes {
                    playback_status: Some(PlaybackStatus::Stopped),
                    playback_elapsed_time_in_milliseconds: Some(0),
                    playback_queue_index: Some(0),
                    playback_queue_count: Some(0),
                    playback_queue_chapter_index: Some(0),
                    playback_shuffle_mode: Some(PlaybackShuffle::Off),
                    playback_repeat_mode: Some(PlaybackRepeat::Off),
                    playback_app_name: Some("".into()),
                    playback_media_library_unique_identifier: Some("".into()),
                    playback_apple_music_radio_ad: Some(false),
                    playback_apple_music_radio_station_name: Some("".into()),
                    playback_apple_music_radio_station_media_playlist_id: Some(0),
                    playback_speed: Some(100),
                    playback_set_elapsed_time_available: Some(false),
                    playback_app_bundle_id: Some("".into()),
                    ..NowPlayingUpdatePlaybackAttributes::default()
                }),
            },
            NowPlayingUpdate {
                media_item_attributes: Some(NowPlayingUpdateMediaItemAttributes {
                    media_item_persistent_identifier: Some(0),
                    media_item_title: Some("".into()),
                    media_item_playback_duration_in_milliseconds: Some(0),
                    media_item_album_title: Some("".into()),
                    media_item_album_track_number: Some(0),
                    media_item_album_track_count: Some(0),
                    media_item_album_disc_number: Some(0),
                    media_item_album_disc_count: Some(0),
                    media_item_artist: Some("".into()),
                    media_item_genre: Some("".into()),
                    media_item_composer: Some("".into()),
                    media_item_is_like_supported: Some(false),
                    media_item_is_ban_supported: Some(false),
                    media_item_is_liked: Some(false),
                    media_item_is_banned: Some(false),
                    media_item_chapter_count: Some(0),
                    ..NowPlayingUpdateMediaItemAttributes::default()
                }),
                playback_attributes: None,
            },
            NowPlayingUpdate {
                media_item_attributes: None,
                playback_attributes: Some(NowPlayingUpdatePlaybackAttributes {
                    playback_queue_list_avail: Some(false),
                    ..NowPlayingUpdatePlaybackAttributes::default()
                }),
            },
            NowPlayingUpdate {
                media_item_attributes: Some(NowPlayingUpdateMediaItemAttributes {
                    media_item_artwork_file_transfer_identifier: next_tx,
                    ..NowPlayingUpdateMediaItemAttributes::default()
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

    async fn handle_legacy_message(
        &mut self,
        message: LingoMessage,
        transaction_id: Option<u16>,
        pending_command: Option<CommandKey>,
        handle: CsmClientHandleRef,
    ) -> CsmSessionResult<()> {
        let reply = |message: LingoMessage, handle: &CsmClientHandleRef| {
            let intent = transaction_id.map_or(TransactionIntent::None, |id| TransactionIntent::Existing {
                id,
                start_transaction: false,
            });
            Self::send_legacy(handle, Outgoing::new(message, intent))
        };
        warn!("iAP1 RX: {message:?} transaction_id={transaction_id:?}");

        if message.cast::<general::StartIDPS>().is_some() {
            reply(
                general::LingoMessage::IPodAck(general::IPodAck {
                    command_result: general::IPodAckCommandResult::OK,
                    acked_command_id: general::StartIDPS::META.command as u8,
                    maximum_pending_wait: None,
                    session_id: None,
                    num_bytes_dropped: None,
                })
                .into(),
                &handle,
            )?;
        } else if message
            .cast::<general::RequestTransportMaxPayloadSize>()
            .is_some()
        {
            reply(
                general::LingoMessage::ReturnTransportMaxPayloadSize(general::ReturnTransportMaxPayloadSize { max_payload: 65529 }).into(),
                &handle,
            )?;
        } else if let Some(request) = message.cast::<general::GetiPodOptionsForLingo>() {
            use general::RetiPodOptionsForLingoOptionBitsDisjoint as Bits;
            let option_bits = match request.lingo_id {
                0 => Bits::GeneralOptions(general::RetiPodOptionsForLingoGeneralOptions::I_POD_NOTIFICATIONS),
                1 => Bits::MicrophoneOptions(general::RetiPodOptionsForLingoMicrophoneOptions::empty()),
                2 => Bits::SimpleRemoteOptions(general::RetiPodOptionsForLingoSimpleRemoteOptions::AUDIO_MEDIA_CONTROLS),
                3 => Bits::DisplayRemoteOptions(general::RetiPodOptionsForLingoDisplayRemoteOptions::UI_VOLUME_CONTROL),
                4 => Bits::ExtendedInterfaceOptions(
                    general::RetiPodOptionsForLingoExtendedInterfaceOptions::EXTENDED_INTERFACE_ENHANCEMENTS
                        | general::RetiPodOptionsForLingoExtendedInterfaceOptions::CATEGORY_DISCOVERY,
                ),
                5 => Bits::AccessoryPowerOptions(general::RetiPodOptionsForLingoAccessoryPowerOptions::empty()),
                6 => Bits::USBHostModeOptions(general::RetiPodOptionsForLingoUSBHostModeOptions::empty()),
                7 => Bits::RFTunerOptions(general::RetiPodOptionsForLingoRFTunerOptions::empty()),
                8 => Bits::AccessoryEqualizerOptions(general::RetiPodOptionsForLingoAccessoryEqualizerOptions::empty()),
                9 => Bits::SportsOptions(general::RetiPodOptionsForLingoSportsOptions::empty()),
                10 => Bits::DigitalAudioOptions(
                    general::RetiPodOptionsForLingoDigitalAudioOptions::SUPPORTS_ACCESSORY_DIGITAL_AUDIO_SAMPLE_RATES_TOKEN,
                ),
                11 => Bits::TestOptions(general::RetiPodOptionsForLingoTestOptions::empty()),
                12 => Bits::StorageOptions(general::RetiPodOptionsForLingoStorageOptions::empty()),
                13 => Bits::IPodOutOptions(general::RetiPodOptionsForLingoIPodOutOptions::empty()),
                14 => Bits::LocationOptions(general::RetiPodOptionsForLingoLocationOptions::empty()),
                _ => Bits::UnknownLingoOptions(general::RetiPodOptionsForLingoUnknownLingoOptions::from_bits_retain(0)),
            };
            reply(
                general::LingoMessage::RetiPodOptionsForLingo(general::RetiPodOptionsForLingo {
                    lingo_id: request.lingo_id,
                    option_bits,
                })
                .into(),
                &handle,
            )?;
        } else if let Some(request) = message.cast::<general::SetFIDTokenValues>() {
            let token_ac_ks = request
                .tokens
                .iter()
                .filter_map(|token| {
                    use general::{AckFIDTokenValuesTokenACKs as Out, SetFIDTokenValuesTokens as In};
                    Some(match token {
                        In::IdentifyToken(_) => Out::IdentifyToken(general::AckFIDTokenValuesTokenACKsIdentifyToken {
                            ack_status: general::AckFIDTokenValuesTokenACKsIdentifyTokenAckStatus::Accepted,
                            busy_lingoes: None,
                        }),
                        In::AccessoryCapsToken(_) => Out::AccessoryCapsToken(general::AckFIDTokenValuesTokenACKsAccessoryCapsToken {
                            ack_status: general::AckFIDTokenValuesTokenACKsAccessoryCapsTokenAckStatus::Accepted,
                        }),
                        In::AccessoryInfoToken(info) => {
                            use general::AckFIDTokenValuesTokenACKsAccessoryInfoTokenAccInfoType as Type;
                            let kind = match info.acc_info_type.raw() {
                                1 => Type::Name,
                                4 => Type::FirmwareVersion,
                                5 => Type::HardwareVersion,
                                6 => Type::Manufacturer,
                                7 => Type::ModelNumber,
                                8 => Type::SerialNumber,
                                9 => Type::MaximumPayloadSize,
                                11 => Type::StatusTypes,
                                12 => Type::RFCertifications,
                                _ => return None,
                            };
                            Out::AccessoryInfoToken(general::AckFIDTokenValuesTokenACKsAccessoryInfoToken {
                                ack_status: general::AckFIDTokenValuesTokenACKsAccessoryInfoTokenAckStatus::Accepted,
                                acc_info_type: kind,
                            })
                        }
                        _ => return None,
                    })
                })
                .collect::<Vec<_>>();
            reply(
                general::LingoMessage::AckFIDTokenValues(general::AckFIDTokenValues {
                    num_fid_token_value_ac_ks: token_ac_ks.len() as u8,
                    token_ac_ks,
                })
                .into(),
                &handle,
            )?;
        } else if message.cast::<general::EndIDPS>().is_some() {
            reply(
                general::LingoMessage::IDPSStatus(general::IDPSStatus {
                    status: general::IDPSStatusStatus::AllRequiredTokenValueFieldsReceivedProceedToAuthentication,
                })
                .into(),
                &handle,
            )?;
            Self::send_legacy(
                &handle,
                Outgoing::new(
                    general::LingoMessage::GetAccessoryAuthenticationInfo(general::GetAccessoryAuthenticationInfo {}),
                    TransactionIntent::New { start_transaction: true },
                ),
            )?;
        } else if let Some(cert) = message.cast::<general::RetAccessoryAuthenticationInfo>() {
            if cert.certificate_current_section_index == Some(cert.certificate_maximum_section_index.unwrap_or(0)) {
                reply(
                    general::LingoMessage::AckAccessoryAuthenticationInfo(general::AckAccessoryAuthenticationInfo {
                        authentication_status: general::AckAccessoryAuthenticationInfoAuthenticationStatus::AuthenticationInfoSupported,
                    })
                    .into(),
                    &handle,
                )?;
                if let Some(id) = transaction_id {
                    handle.finish_lingo_transaction(id);
                }
                Self::send_legacy(
                    &handle,
                    Outgoing::new(
                        general::LingoMessage::GetAccessoryAuthenticationSignature(general::GetAccessoryAuthenticationSignature {
                            challenge: Some(general::GetAccessoryAuthenticationSignatureChallengeDisjoint::Value20ByteChallenge(
                                [0x42; 20],
                            )),
                            authentication_retry_counter: Some(1),
                        }),
                        TransactionIntent::New { start_transaction: true },
                    ),
                )?;
                Self::send_legacy(
                    &handle,
                    Outgoing::new(
                        audio::LingoMessage::GetAccessorySampleRateCaps(audio::GetAccessorySampleRateCaps {}),
                        TransactionIntent::New { start_transaction: true },
                    ),
                )?;
            } else {
                reply(
                    general::LingoMessage::IPodAck(general::IPodAck {
                        command_result: general::IPodAckCommandResult::OK,
                        acked_command_id: general::RetAccessoryAuthenticationInfo::META.command as u8,
                        maximum_pending_wait: None,
                        session_id: None,
                        num_bytes_dropped: None,
                    })
                    .into(),
                    &handle,
                )?;
            }
        } else if let Some(signature) = message.cast::<general::RetAccessoryAuthenticationSignature>() {
            if !signature.calculated_signature.is_empty() {
                reply(
                    general::LingoMessage::AckAccessoryAuthenticationStatus(general::AckAccessoryAuthenticationStatus {
                        status: general::AckAccessoryAuthenticationStatusStatus::Passed,
                    })
                    .into(),
                    &handle,
                )?;
            }
            if pending_command == Some(general::GetAccessoryAuthenticationSignature::KEY) {
                if let Some(id) = transaction_id {
                    handle.finish_lingo_transaction(id);
                }
            }
        } else if message
            .cast::<audio::RetAccessorySampleRateCaps>()
            .is_some()
        {
            if pending_command == Some(audio::GetAccessorySampleRateCaps::KEY) {
                if let Some(id) = transaction_id {
                    handle.finish_lingo_transaction(id);
                }
            }
        } else if message.cast::<audio::TrackNewAudioAttributes>().is_some() {
            reply(
                audio::LingoMessage::AccessoryAck(audio::AccessoryAck {
                    command_result: audio::AccessoryAckCommandResult::OK,
                    acked_command_id: audio::TrackNewAudioAttributes::META.command as u8,
                })
                .into(),
                &handle,
            )?;
        } else if message
            .cast::<general::GetSupportedEventNotification>()
            .is_some()
        {
            reply(
                general::LingoMessage::RetSupportedEventNotification(general::RetSupportedEventNotification {
                    event_notification_mask: general::RetSupportedEventNotificationEventNotificationMask::DATABASE_CHANGED
                        | general::RetSupportedEventNotificationEventNotificationMask::NOW_PLAYING_APP_BUNDLE_NAME
                        | general::RetSupportedEventNotificationEventNotificationMask::NOW_PLAYING_APPLICATION_DISPLAY_NAME,
                })
                .into(),
                &handle,
            )?;
        } else if message.cast::<general::SetEventNotification>().is_some() || message.cast::<general::SetUIMode>().is_some() {
            let subscription = message.cast::<general::SetEventNotification>();
            let command = if message.cast::<general::SetUIMode>().is_some() {
                general::SetUIMode::META.command
            } else {
                general::SetEventNotification::META.command
            };
            reply(
                general::LingoMessage::IPodAck(general::IPodAck {
                    command_result: general::IPodAckCommandResult::OK,
                    acked_command_id: command as u8,
                    maximum_pending_wait: None,
                    session_id: None,
                    num_bytes_dropped: None,
                })
                .into(),
                &handle,
            )?;
            if let Some(subscription) = subscription {
                use general::SetEventNotificationEventNotificationMask as Mask;
                if subscription
                    .event_notification_mask
                    .contains(Mask::NOW_PLAYING_APP_BUNDLE_NAME)
                {
                    Self::send_legacy(
                        &handle,
                        Outgoing::new(
                            general::LingoMessage::IPodNotification(general::IPodNotification {
                                notification_type: general::IPodNotificationNotificationType::NowPlayingAppBundleName,
                                bundle_seed_id: Some("com.apple.mobileipod".into()),
                                ..Default::default()
                            }),
                            TransactionIntent::New { start_transaction: false },
                        ),
                    )?;
                }
                if subscription
                    .event_notification_mask
                    .contains(Mask::NOW_PLAYING_APPLICATION_DISPLAY_NAME)
                {
                    Self::send_legacy(
                        &handle,
                        Outgoing::new(
                            general::LingoMessage::IPodNotification(general::IPodNotification {
                                notification_type: general::IPodNotificationNotificationType::NowPlayingApplicationDisplayName,
                                app_name: Some(String::new()),
                                ..Default::default()
                            }),
                            TransactionIntent::New { start_transaction: false },
                        ),
                    )?;
                }
            }
        } else if message.cast::<general::RequestiPodName>().is_some() {
            reply(
                general::LingoMessage::ReturniPodName(general::ReturniPodName {
                    i_pod_name: "CatPlay".into(),
                })
                .into(),
                &handle,
            )?;
        } else if message.cast::<general::RequestiPodSerialNum>().is_some() {
            reply(
                general::LingoMessage::ReturniPodSerialNum(general::ReturniPodSerialNum {
                    i_pod_serial_number: "CATPLAY0001".into(),
                })
                .into(),
                &handle,
            )?;
        } else if message
            .cast::<general::RequestiPodSoftwareVersion>()
            .is_some()
        {
            reply(
                general::LingoMessage::ReturniPodSoftwareVersion(general::ReturniPodSoftwareVersion {
                    i_pod_major_version_number: 27,
                    i_pod_minor_version_number: 0,
                    i_pod_revision_version_number: 1,
                })
                .into(),
                &handle,
            )?;
        } else if message.cast::<general::RequestiPodModelNum>().is_some() {
            reply(
                general::LingoMessage::ReturniPodModelNum(general::ReturniPodModelNum {
                    i_pod_model_id: 2490368,
                    i_pod_model_number: "CatPlay".into(),
                })
                .into(),
                &handle,
            )?;
        } else if message
            .cast::<general::GetNowPlayingApplicationBundleName>()
            .is_some()
        {
            reply(
                general::LingoMessage::RetNowPlayingApplicationBundleName(general::RetNowPlayingApplicationBundleName {
                    application_id: "com.apple.mobileipod".into(),
                })
                .into(),
                &handle,
            )?;
        } else if message.cast::<extended::GetPlayStatus>().is_some() {
            reply(
                extended::LingoMessage::ReturnPlayStatus(extended::ReturnPlayStatus {
                    track_length: 0,
                    track_position: 0,
                    player_state: extended::ReturnPlayStatusPlayerState::Stopped,
                })
                .into(),
                &handle,
            )?;
        } else if message.cast::<extended::GetRepeat>().is_some() {
            reply(
                extended::LingoMessage::ReturnRepeat(extended::ReturnRepeat {
                    repeat_state: extended::ReturnRepeatRepeatState::Off,
                })
                .into(),
                &handle,
            )?;
        } else if message.cast::<extended::GetShuffle>().is_some() {
            reply(
                extended::LingoMessage::ReturnShuffle(extended::ReturnShuffle {
                    shuffle_mode: extended::ReturnShuffleShuffleMode::Off,
                })
                .into(),
                &handle,
            )?;
        } else if message.cast::<extended::GetNumPlayingTracks>().is_some() {
            reply(
                extended::LingoMessage::ReturnNumPlayingTracks(extended::ReturnNumPlayingTracks {
                    number_of_tracks_playing: 0,
                })
                .into(),
                &handle,
            )?;
        } else if message
            .cast::<extended::GetNumberCategorizedDBRecords>()
            .is_some()
        {
            reply(
                extended::LingoMessage::ReturnNumberCategorizedDBRecords(extended::ReturnNumberCategorizedDBRecords {
                    database_record_count: 0,
                })
                .into(),
                &handle,
            )?;
        } else if message
            .cast::<extended::RetrieveCategorizedDatabaseRecords>()
            .is_some()
            || message.cast::<extended::GetPBTrackInfo>().is_some()
        {
            let command = if message.cast::<extended::GetPBTrackInfo>().is_some() {
                extended::GetPBTrackInfo::META.command
            } else {
                extended::RetrieveCategorizedDatabaseRecords::META.command
            };
            reply(
                extended::LingoMessage::IPodAck(extended::IPodAck {
                    command_result: extended::IPodAckCommandResult::BadParameter,
                    acked_command_id: command,
                    maximum_pending_wait: None,
                    session_id: None,
                    num_bytes_dropped: None,
                })
                .into(),
                &handle,
            )?;
        } else if message
            .cast::<extended::GetColorDisplayImageLimits>()
            .is_some()
        {
            use extended::ReturnColorDisplayImageLimitsColorDisplayImageLimitsDisplayPixelFormatCode as Format;
            let formats = [Format::RGB565ColorLittleEndian16Bpp, Format::RGB565ColorBigEndian16Bpp]
                .into_iter()
                .map(|format| extended::ReturnColorDisplayImageLimitsColorDisplayImageLimits {
                    max_image_width: 166,
                    max_image_height: 76,
                    display_pixel_format_code: format,
                })
                .collect();
            reply(
                extended::LingoMessage::ReturnColorDisplayImageLimits(extended::ReturnColorDisplayImageLimits {
                    color_display_image_limits: formats,
                })
                .into(),
                &handle,
            )?;
        } else if message
            .cast::<extended::GetMonoDisplayImageLimits>()
            .is_some()
        {
            reply(
                extended::LingoMessage::ReturnMonoDisplayImageLimits(extended::ReturnMonoDisplayImageLimits {
                    max_image_width: 166,
                    max_image_height: 76,
                    display_pixel_format_code: extended::ReturnMonoDisplayImageLimitsDisplayPixelFormatCode::Monochrome2BitsPerPixel,
                })
                .into(),
                &handle,
            )?;
        } else if let Some(request) = message.cast::<extended::GetDBiTunesInfo>() {
            use extended::RetDBiTunesInfoITunesDBMetadataType as Kind;
            let kind = match request.i_tunes_db_metadata_type.raw() {
                0 => Kind::DatabaseUID,
                1 => Kind::LastITunesSyncDateTime,
                2 => Kind::TotalAudioTrackCount,
                3 => Kind::TotalVideoTrackCount,
                4 => Kind::TotalAudiobookCount,
                _ => Kind::TotalPhotoCount,
            };
            reply(
                extended::LingoMessage::RetDBiTunesInfo(extended::RetDBiTunesInfo {
                    i_tunes_db_metadata_type: kind,
                    i_tunes_dbuid: (kind == Kind::DatabaseUID).then_some(0),
                    seconds: (kind == Kind::LastITunesSyncDateTime).then_some(0),
                    minute: (kind == Kind::LastITunesSyncDateTime).then_some(0),
                    hour: (kind == Kind::LastITunesSyncDateTime).then_some(0),
                    day: (kind == Kind::LastITunesSyncDateTime).then_some(1),
                    month: (kind == Kind::LastITunesSyncDateTime).then_some(1),
                    year: (kind == Kind::LastITunesSyncDateTime).then_some(2026),
                    total_audio_track_count: (kind == Kind::TotalAudioTrackCount).then_some(0),
                    total_video_track_count: (kind == Kind::TotalVideoTrackCount).then_some(0),
                    total_audiobook_count: (kind == Kind::TotalAudiobookCount).then_some(0),
                    total_photo_count: (kind == Kind::TotalPhotoCount).then_some(0),
                })
                .into(),
                &handle,
            )?;
        } else if message.cast::<extended::ResetDBSelection>().is_some()
            || message.cast::<extended::SelectDBRecord>().is_some()
            || message
                .cast::<extended::SetPlayStatusChangeNotification>()
                .is_some()
        {
            let command = if message.cast::<extended::ResetDBSelection>().is_some() {
                extended::ResetDBSelection::META.command
            } else if message.cast::<extended::SelectDBRecord>().is_some() {
                extended::SelectDBRecord::META.command
            } else {
                extended::SetPlayStatusChangeNotification::META.command
            };
            let result = if command == extended::SelectDBRecord::META.command {
                extended::IPodAckCommandResult::BadParameter
            } else {
                extended::IPodAckCommandResult::OK
            };
            reply(
                extended::LingoMessage::IPodAck(extended::IPodAck {
                    command_result: result,
                    acked_command_id: command,
                    maximum_pending_wait: None,
                    session_id: None,
                    num_bytes_dropped: None,
                })
                .into(),
                &handle,
            )?;
        } else if message
            .cast::<display::SetRemoteEventNotification>()
            .is_some()
        {
            reply(
                display::LingoMessage::IPodAck(display::IPodAck {
                    command_result: display::IPodAckCommandResult::OK,
                    acked_command_id: display::SetRemoteEventNotification::META.command as u8,
                    maximum_pending_wait: None,
                    session_id: None,
                    num_bytes_dropped: None,
                })
                .into(),
                &handle,
            )?;
        }
        Ok(())
    }
}
#[async_trait]
impl CsmSession for CarPlayClientSession {
    async fn on_legacy_message(
        &mut self,
        message: LingoMessage,
        transaction_id: Option<u16>,
        pending_command: Option<CommandKey>,
        handle: CsmClientHandleRef,
    ) -> CsmSessionResult<()> {
        self.handle_legacy_message(message, transaction_id, pending_command, handle)
            .await
    }

    async fn start(&mut self, _handle: CsmClientHandleRef) -> CsmSessionResult<()> {
        if self.start.is_none() {
            self.start.replace(Instant::now());
        }

        if !_handle.is_downgrade() {
            self.send(&StartIdentification::default(), &_handle)?;
        }
        Ok(())
    }

    async fn respond(&mut self, packet: CsmPacketBox, _handle: CsmClientHandleRef) -> CsmSessionResult<()> {
        if self.start.is_none() {
            self.start.replace(Instant::now());
        }
        warn!("-> Incoming packet @ {:?}: {:?}", Instant::now() - self.start.unwrap(), packet);

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
                &DeviceLanguageUpdate {
                    device_language: Some("en".into()),
                },
                &_handle,
            )?;
            self.send(
                &DeviceInformationUpdate {
                    device_name: Some("iPhone SIM".into()),
                },
                &_handle,
            )?;
            self.send(
                &DeviceTransportIdentifierNotification {
                    // TODO: get this from g_iphone later
                    usb_transport_identifier: Some("00008130000E044E384B1D3ADDDDDDD000BADCAD".into()),
                    // TODO: use real one later
                    bluetooth_transport_identifier: Some("aa:bb:cc:dd:ee:ff".into()),
                },
                &_handle,
            )?;

            self.send(
                &DeviceTimeUpdate {
                    seconds_since_reference_date: 1_445_470_140,
                    time_zone_offset_minutes: 0,
                    daylight_savings_offset_minutes: 0,
                },
                &_handle,
            )?;
            self.send(
                &StartLocationInformation {
                    global_positioning_system_fix_data: CsmFlag::Yes,
                    recommended_minimum_specific_gps_transit_data: CsmFlag::Yes,
                    gps_satellites_in_view: CsmFlag::No,
                    vehicle_speed_data: CsmFlag::Yes,
                    vehicle_gyro_data: CsmFlag::No,
                    vehicle_accelerometer_data: CsmFlag::No,
                    vehicle_heading_data: CsmFlag::No,
                    ..StartLocationInformation::default()
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

#[cfg(test)]
mod legacy_tests {
    use super::*;
    use catplay_iap2_client::{CsmClientHandle, CsmRemote, CsmSessionStatus};
    use catplay_lingo::link_layer::LinkLayer as LingoLinkLayer;
    use std::sync::{Arc, Mutex};

    struct Handle {
        downgrade: bool,
        remote: CsmRemote,
        csm_count: Mutex<usize>,
        lingo: Mutex<Vec<Outgoing>>,
        lingo_link: Mutex<LingoLinkLayer>,
        sent_ids: Mutex<Vec<Option<u16>>>,
    }

    impl Handle {
        fn new(downgrade: bool) -> Arc<Self> {
            let mut lingo_link = LingoLinkLayer::with_role(true);
            lingo_link.transactions_mut().set_ids_enabled(downgrade);
            Arc::new(Self {
                downgrade,
                remote: CsmRemote::usb_gadget(),
                csm_count: Mutex::new(0),
                lingo: Mutex::new(Vec::new()),
                lingo_link: Mutex::new(lingo_link),
                sent_ids: Mutex::new(Vec::new()),
            })
        }
    }

    impl CsmClientHandle for Handle {
        fn disconnect(&self) {}
        fn is_server(&self) -> bool {
            true
        }
        fn is_closed(&self) -> bool {
            false
        }
        fn is_downgrade(&self) -> bool {
            self.downgrade
        }
        fn is_writable(&self) -> bool {
            true
        }
        fn remote(&self) -> &CsmRemote {
            &self.remote
        }
        fn status(&self) -> CsmSessionStatus {
            if self.downgrade {
                CsmSessionStatus::Downgrade
            } else {
                CsmSessionStatus::Writable
            }
        }
        fn send(&self, _: &dyn AsCsmPacket) -> CsmSessionResult<()> {
            *self.csm_count.lock().unwrap() += 1;
            Ok(())
        }
        fn send_all(&self, packets: &[CsmPacketBox]) -> CsmSessionResult<()> {
            *self.csm_count.lock().unwrap() += packets.len();
            Ok(())
        }
        fn send_lingo(&self, outgoing: Outgoing) -> CsmSessionResult<()> {
            let packet = self.lingo_link.lock().unwrap().send(outgoing.clone())?;
            packet.encode_body()?;
            self.sent_ids.lock().unwrap().push(packet.transaction_id);
            self.lingo.lock().unwrap().push(outgoing);
            Ok(())
        }
        fn finish_lingo_transaction(&self, id: u16) {
            self.lingo_link
                .lock()
                .unwrap()
                .transactions_mut()
                .finish_transaction(id);
        }
        fn pending_lingo_transaction_count(&self) -> Option<usize> {
            Some(
                self.lingo_link
                    .lock()
                    .unwrap()
                    .transactions()
                    .pending_count(),
            )
        }
        fn send_file_reserve(&self) -> Option<u8> {
            None
        }
        fn send_file(&self, _: u8, _: u16, _: &[u8], _: Vec<u8>) {}
    }

    #[test]
    fn downgrade_waits_for_idps_and_replies_with_same_transaction() {
        let handle = Handle::new(true);
        let mut session = CarPlayClientSession::default();
        futures::executor::block_on(session.start(handle.clone())).unwrap();
        assert_eq!(*handle.csm_count.lock().unwrap(), 0);
        assert!(handle.lingo.lock().unwrap().is_empty());

        futures::executor::block_on(session.on_legacy_message(
            general::LingoMessage::StartIDPS(general::StartIDPS {}).into(),
            Some(7),
            None,
            handle.clone(),
        ))
        .unwrap();
        let outgoing = handle.lingo.lock().unwrap();
        assert_eq!(
            outgoing[0].transaction,
            TransactionIntent::Existing {
                id: 7,
                start_transaction: false
            }
        );
        let ack = outgoing[0].message.cast::<general::IPodAck>().unwrap();
        assert_eq!(ack.acked_command_id, general::StartIDPS::META.command as u8);
        assert_eq!(handle.pending_lingo_transaction_count(), Some(0));
    }

    #[test]
    fn modern_start_sends_identification() {
        let handle = Handle::new(false);
        futures::executor::block_on(CarPlayClientSession::default().start(handle.clone())).unwrap();
        assert_eq!(*handle.csm_count.lock().unwrap(), 1);
    }

    #[test]
    fn idps_completion_starts_certificate_query() {
        let handle = Handle::new(true);
        let mut session = CarPlayClientSession::default();
        futures::executor::block_on(
            session.on_legacy_message(
                general::LingoMessage::EndIDPS(general::EndIDPS {
                    acc_end_idps_status: general::EndIDPSAccEndIDPSStatus::Done,
                })
                .into(),
                Some(10),
                None,
                handle.clone(),
            ),
        )
        .unwrap();
        let outgoing = handle.lingo.lock().unwrap();
        assert_eq!(
            outgoing[0].transaction,
            TransactionIntent::Existing {
                id: 10,
                start_transaction: false
            }
        );
        assert!(outgoing[0].message.cast::<general::IDPSStatus>().is_some());
        assert_eq!(outgoing[1].transaction, TransactionIntent::New { start_transaction: true });
        assert!(
            outgoing[1]
                .message
                .cast::<general::GetAccessoryAuthenticationInfo>()
                .is_some()
        );
        assert_eq!(handle.pending_lingo_transaction_count(), Some(1));
    }

    #[test]
    fn authentication_and_audio_replies_close_all_local_transactions() {
        let handle = Handle::new(true);
        let mut session = CarPlayClientSession::default();
        futures::executor::block_on(
            session.on_legacy_message(
                general::LingoMessage::EndIDPS(general::EndIDPS {
                    acc_end_idps_status: general::EndIDPSAccEndIDPSStatus::Done,
                })
                .into(),
                Some(10),
                None,
                handle.clone(),
            ),
        )
        .unwrap();
        assert_eq!(handle.pending_lingo_transaction_count(), Some(1));
        let certificate_id = handle.sent_ids.lock().unwrap()[1].unwrap();

        for section in 0..=1 {
            futures::executor::block_on(
                session.on_legacy_message(
                    general::LingoMessage::RetAccessoryAuthenticationInfo(general::RetAccessoryAuthenticationInfo {
                        authentication_protocol_major_version_number: 2,
                        authentication_protocol_minor_version_number: 0,
                        certificate_current_section_index: Some(section),
                        certificate_maximum_section_index: Some(1),
                        certificate_data: Some(vec![section]),
                    })
                    .into(),
                    Some(certificate_id),
                    Some(general::GetAccessoryAuthenticationInfo::KEY),
                    handle.clone(),
                ),
            )
            .unwrap();
            assert_eq!(handle.pending_lingo_transaction_count(), Some(if section == 0 { 1 } else { 2 }));
        }

        let (signature_id, audio_id) = {
            let ids = handle.sent_ids.lock().unwrap();
            (ids[4].unwrap(), ids[5].unwrap())
        };
        futures::executor::block_on(
            session.on_legacy_message(
                general::LingoMessage::RetAccessoryAuthenticationSignature(general::RetAccessoryAuthenticationSignature {
                    calculated_signature: vec![1],
                })
                .into(),
                Some(signature_id),
                Some(general::GetAccessoryAuthenticationSignature::KEY),
                handle.clone(),
            ),
        )
        .unwrap();
        assert_eq!(handle.pending_lingo_transaction_count(), Some(1));

        futures::executor::block_on(session.on_legacy_message(
            audio::LingoMessage::RetAccessorySampleRateCaps(audio::RetAccessorySampleRateCaps { sample_rates: Vec::new() }).into(),
            Some(audio_id),
            Some(audio::GetAccessorySampleRateCaps::KEY),
            handle.clone(),
        ))
        .unwrap();
        assert_eq!(handle.pending_lingo_transaction_count(), Some(0));
    }

    #[test]
    fn lingo_options_use_requested_variant_and_transaction() {
        let handle = Handle::new(true);
        let mut session = CarPlayClientSession::default();
        futures::executor::block_on(session.on_legacy_message(
            general::LingoMessage::GetiPodOptionsForLingo(general::GetiPodOptionsForLingo { lingo_id: 4 }).into(),
            Some(4),
            None,
            handle.clone(),
        ))
        .unwrap();
        let outgoing = handle.lingo.lock().unwrap();
        assert_eq!(
            outgoing[0].transaction,
            TransactionIntent::Existing {
                id: 4,
                start_transaction: false
            }
        );
        let response = outgoing[0]
            .message
            .cast::<general::RetiPodOptionsForLingo>()
            .unwrap();
        assert_eq!(response.lingo_id, 4);
        assert!(matches!(
            response.option_bits,
            general::RetiPodOptionsForLingoOptionBitsDisjoint::ExtendedInterfaceOptions(_)
        ));
    }

    #[test]
    fn fid_ack_counts_accepted_tokens() {
        let handle = Handle::new(true);
        let mut session = CarPlayClientSession::default();
        futures::executor::block_on(
            session.on_legacy_message(
                general::LingoMessage::SetFIDTokenValues(general::SetFIDTokenValues {
                    num_fid_token_values: 1,
                    tokens: vec![general::SetFIDTokenValuesTokens::AccessoryCapsToken(
                        general::SetFIDTokenValuesTokensAccessoryCapsToken {
                            acc_caps: general::SetFIDTokenValuesTokensAccessoryCapsTokenAccCaps::USB_AUDIO,
                        },
                    )],
                })
                .into(),
                Some(9),
                None,
                handle.clone(),
            ),
        )
        .unwrap();
        let outgoing = handle.lingo.lock().unwrap();
        let response = outgoing[0]
            .message
            .cast::<general::AckFIDTokenValues>()
            .unwrap();
        assert_eq!(response.num_fid_token_value_ac_ks, 1);
        assert_eq!(response.token_ac_ks.len(), 1);
    }

    #[test]
    fn event_subscription_sends_ack_and_requested_notifications() {
        let handle = Handle::new(true);
        let mut session = CarPlayClientSession::default();
        futures::executor::block_on(
            session.on_legacy_message(
                general::LingoMessage::SetEventNotification(general::SetEventNotification {
                    event_notification_mask: general::SetEventNotificationEventNotificationMask::NOW_PLAYING_APP_BUNDLE_NAME
                        | general::SetEventNotificationEventNotificationMask::NOW_PLAYING_APPLICATION_DISPLAY_NAME,
                })
                .into(),
                Some(13),
                None,
                handle.clone(),
            ),
        )
        .unwrap();
        let outgoing = handle.lingo.lock().unwrap();
        assert_eq!(outgoing.len(), 3);
        assert!(outgoing[0].message.cast::<general::IPodAck>().is_some());
        assert_eq!(outgoing[1].transaction, TransactionIntent::New { start_transaction: false });
        assert_eq!(outgoing[2].transaction, TransactionIntent::New { start_transaction: false });
    }
}
