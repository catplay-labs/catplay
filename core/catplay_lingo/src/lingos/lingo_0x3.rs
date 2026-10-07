// @generated from Apple ATS Default.lktspec
// Do not edit manually. Regenerate with catplay_lingo.

#[allow(unused_imports)]
use crate::{__String as String, __Vec as Vec};
#[allow(unused_imports)]
use crate::{
    FlatPredicateToken, Iap1WireSpec, iap1, iap1_bitfield, iap1_bitflags, iap1_disjoint, iap1_enum,
    iap1_enum_open, iap1_enum_tokens, iap1_record, iap1_registry, predicate_eval::*,
};

use super::strings::LingoString;

// Lingo 0x03: Display Remote
iap1_enum! {
    strings = LingoString;
    pub enum IPodAckCommandResult: u8 {
        /// OK
        OK = 0,
        /// unknown database category or session ID
        UnknownDatabaseCategoryOrSessionID = 1,
        /// command failed
        CommandFailed = 2,
        /// Apple device out of resources
        AppleDeviceOutOfResources = 3,
        /// bad parameter
        BadParameter = 4,
        /// unknown ID
        UnknownID = 5,
        /// command pending
        CommandPending = 6,
        /// not authenticated
        NotAuthenticated = 7,
        /// bad authentication version
        BadAuthenticationVersion = 8,
        /// accessory power mode request failed
        AccessoryPowerModeRequestFailed = 9,
        /// certificate invalid
        CertificateInvalid = 10,
        /// certificate permissions invalid
        CertificatePermissionsInvalid = 11,
        /// file in use
        FileInUse = 12,
        /// invalid file handle
        InvalidFileHandle = 13,
        /// directory not empty
        DirectoryNotEmpty = 14,
        /// operation timed out
        OperationTimedOut = 15,
        /// command unavailable in this iPod mode
        CommandUnavailableInThisIPodMode = 16,
        /// Accessory Detect not grounded or invalid Accessory Identify Resistor detected
        AccessoryDetectNotGroundedOrInvalidAccessoryIdentifyResistorDetected = 17,
        /// selection not genius
        SelectionNotGenius = 18,
        /// multisection data section received successfully
        MultisectionDataSectionReceivedSuccessfully = 19,
        /// lingo busy
        LingoBusy = 20,
        /// maximum number of accessory connections already reached
        MaximumNumberOfAccessoryConnectionsAlreadyReached = 21,
        /// HID descriptor index already in use
        HIDDescriptorIndexAlreadyInUse = 22,
        /// dropped data (SessionWriteFailure)
        DroppedDataSessionWriteFailure = 23,
        /// attempt to enter iPod Out mode with incompatible video settings
        AttemptToEnterIPodOutModeWithIncompatibleVideoSettings = 24,
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x03, command = 0x0000,
        source = device,
        response = true,
        ack = true,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct IPodAck {
        fields {
            /// Command Result
            // Virtual schema indices: 1
            required pub command_result: IPodAckCommandResult => { bit: 0, key: "commandResult", when: (truthy(&true)), predicate_value: true },
            /// Command ID
            // Virtual schema indices: 2
            required pub acked_command_id: u8 => { bit: 1, key: "ackedCommandID", when: (truthy(&true)), predicate_value: false },
            /// Maximum Pending Wait
            // Virtual schema indices: 3
            optional pub maximum_pending_wait: u32 => { bit: 2, key: "maximumPendingWait", when: (eq(&command_result, &6u64)), predicate_value: false },
            /// Session ID
            // Virtual schema indices: 4
            optional pub session_id: u16 => { bit: 3, key: "sessionID", when: (eq(&command_result, &23u64)), predicate_value: false },
            /// Number of Bytes Dropped
            // Virtual schema indices: 5
            optional pub num_bytes_dropped: u32 => { bit: 4, key: "numBytesDropped", when: (eq(&command_result, &23u64)), predicate_value: false },
        }
        steps {
            1 => required command_result: IPodAckCommandResult => { bit: 0, key: "commandResult", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: true, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            2 => required acked_command_id: u8 => { bit: 1, key: "ackedCommandID", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            3 => optional maximum_pending_wait: u32 => { bit: 2, key: "maximumPendingWait", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&command_result, &6u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(6u8)] } },
            4 => optional session_id: u16 => { bit: 3, key: "sessionID", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&command_result, &23u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(23u8)] } },
            5 => optional num_bytes_dropped: u32 => { bit: 4, key: "numBytesDropped", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&command_result, &23u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(23u8)] } },
        }
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x03, command = 0x0001,
        source = accessory,
        response = false,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct GetCurrentEQProfileIndex {
        fields {
        }
        steps {
        }
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x03, command = 0x0002,
        source = device,
        response = true,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct RetCurrentEQProfileIndex {
        fields {
            /// Current Equalizer Index
            // Virtual schema indices: 1
            required pub current_equalizer_index: u32 => { bit: 0, key: "currentEqualizerIndex", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required current_equalizer_index: u32 => { bit: 0, key: "currentEqualizerIndex", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x03, command = 0x0003,
        source = accessory,
        response = false,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct SetCurrentEQProfileIndex {
        fields {
            /// Current Equalizer Index
            // Virtual schema indices: 1
            required pub current_equalizer_index: u32 => { bit: 0, key: "currentEqualizerIndex", when: (truthy(&true)), predicate_value: false },
            /// Restore on Exit
            // Virtual schema indices: 2
            required pub restore_on_exit: bool => { bit: 1, key: "restoreOnExit", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required current_equalizer_index: u32 => { bit: 0, key: "currentEqualizerIndex", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            2 => required restore_on_exit: bool => { bit: 1, key: "restoreOnExit", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x03, command = 0x0004,
        source = accessory,
        response = false,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct GetNumEQProfiles {
        fields {
        }
        steps {
        }
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x03, command = 0x0005,
        source = device,
        response = true,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct RetNumEQProfiles {
        fields {
            /// Equalizer Profile Count
            // Virtual schema indices: 1
            required pub profile_count: u32 => { bit: 0, key: "profileCount", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required profile_count: u32 => { bit: 0, key: "profileCount", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x03, command = 0x0006,
        source = accessory,
        response = false,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct GetIndexedEQProfileName {
        fields {
            /// Equalizer Profile Index
            // Virtual schema indices: 1
            required pub equalizer_profile_index: u32 => { bit: 0, key: "equalizerProfileIndex", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required equalizer_profile_index: u32 => { bit: 0, key: "equalizerProfileIndex", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x03, command = 0x0007,
        source = device,
        response = true,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct RetIndexedEQProfileName {
        fields {
            /// Equalizer Profile Name
            // Virtual schema indices: 1
            required pub equalizer_profile_name: String => { bit: 0, key: "equalizerProfileName", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required equalizer_profile_name: String => { bit: 0, key: "equalizerProfileName", wire: [raw_string], project_wire: Iap1WireSpec::Raw, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1_bitflags! {
    strings = LingoString;
    pub struct SetRemoteEventNotificationRemoteEventMask: u32 {
        /// track time position in milliseconds
        const TRACK_TIME_POSITION_IN_MILLISECONDS = 1 << 0;
        /// track playback index
        const TRACK_PLAYBACK_INDEX = 1 << 1;
        /// chapter index
        const CHAPTER_INDEX = 1 << 2;
        /// play status
        const PLAY_STATUS = 1 << 3;
        /// mute/UI volume
        const MUTE_UI_VOLUME = 1 << 4;
        /// power/battery
        const POWER_BATTERY = 1 << 5;
        /// equalizer setting
        const EQUALIZER_SETTING = 1 << 6;
        /// shuffle setting
        const SHUFFLE_SETTING = 1 << 7;
        /// repeat setting
        const REPEAT_SETTING = 1 << 8;
        /// date and time setting
        const DATE_AND_TIME_SETTING = 1 << 9;
        /// alarm setting
        const ALARM_SETTING = 1 << 10;
        /// backlight level
        const BACKLIGHT_LEVEL = 1 << 11;
        /// hold switch state
        const HOLD_SWITCH_STATE = 1 << 12;
        /// sound check state
        const SOUND_CHECK_STATE = 1 << 13;
        /// audiobook state
        const AUDIOBOOK_STATE = 1 << 14;
        /// track time position in seconds
        const TRACK_TIME_POSITION_IN_SECONDS = 1 << 15;
        /// mute/UI/absolute volume
        const MUTE_UI_ABSOLUTE_VOLUME = 1 << 16;
        /// track capabilities
        const TRACK_CAPABILITIES = 1 << 17;
        /// playback engine notification
        const PLAYBACK_ENGINE_NOTIFICATION = 1 << 18;
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x03, command = 0x0008,
        source = accessory,
        response = false,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct SetRemoteEventNotification {
        fields {
            /// Remote Event Mask
            // Virtual schema indices: 1
            required pub remote_event_mask: SetRemoteEventNotificationRemoteEventMask => { bit: 0, key: "remoteEventMask", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required remote_event_mask: SetRemoteEventNotificationRemoteEventMask => { bit: 0, key: "remoteEventMask", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum RemoteEventNotificationEventNum: u8 {
        /// track position in milliseconds
        TrackPositionInMilliseconds = 0,
        /// track playback index
        TrackPlaybackIndex = 1,
        /// chapter info
        ChapterInfo = 2,
        /// play status
        PlayStatus = 3,
        /// mute and volume info
        MuteAndVolumeInfo = 4,
        /// power and battery status
        PowerAndBatteryStatus = 5,
        /// equalizer setting
        EqualizerSetting = 6,
        /// shuffle setting
        ShuffleSetting = 7,
        /// repeat setting
        RepeatSetting = 8,
        /// date and time
        DateAndTime = 9,
        /// alarm state and time
        AlarmStateAndTime = 10,
        /// backlight level
        BacklightLevel = 11,
        /// hold switch state
        HoldSwitchState = 12,
        /// sound check state
        SoundCheckState = 13,
        /// audiobook speed
        AudiobookSpeed = 14,
        /// track position in seconds
        TrackPositionInSeconds = 15,
        /// mute/UI/absolute volume
        MuteUIAbsoluteVolume = 16,
        /// track capabilities
        TrackCapabilities = 17,
        /// playback engine contents changed
        PlaybackEngineContentsChanged = 18,
    }
}

iap1_disjoint! {
    strings = LingoString;
    #[iap1_disjoint(predicate = integer)]
    pub enum RemoteEventNotificationTrackPositionDisjoint {
        #[iap1_disjoint(decode = decode_track_position, encode = encode_track_position)]
        /// Track Position
        TrackPosition(u32),
        #[iap1_disjoint(decode = decode_track_position2, encode = encode_track_position2)]
        /// Track Position
        TrackPosition2(u16),
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum RemoteEventNotificationPlayStatus: u8 {
        /// playback stopped
        PlaybackStopped = 0,
        /// playing
        Playing = 1,
        /// playback paused
        PlaybackPaused = 2,
        /// fast forward
        FastForward = 3,
        /// fast rewind
        FastRewind = 4,
        /// end fast forward or rewind mode
        EndFastForwardOrRewindMode = 5,
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum RemoteEventNotificationPowerState: u8 {
        /// internal battery power, low power (< 30%)
        InternalBatteryPowerLowPower30 = 0,
        /// internal battery power
        InternalBatteryPower = 1,
        /// external power, battery pack, no charging
        ExternalPowerBatteryPackNoCharging = 2,
        /// external power, battery charging
        ExternalPowerBatteryCharging = 3,
        /// external power, battery charged
        ExternalPowerBatteryCharged = 4,
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum RemoteEventNotificationShuffleSetting: u8 {
        /// off
        Off = 0,
        /// tracks and songs
        TracksAndSongs = 1,
        /// albums
        Albums = 2,
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum RemoteEventNotificationRepeatSetting: u8 {
        /// off
        Off = 0,
        /// one track
        OneTrack = 1,
        /// all tracks
        AllTracks = 2,
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum RemoteEventNotificationAlarmState: u8 {
        /// off
        Off = 0,
        /// enabled
        Enabled = 1,
        /// triggered
        Triggered = 2,
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum RemoteEventNotificationAudiobookSpeed: u8 {
        /// slower (-1)
        Slower1 = 255,
        /// normal
        Normal = 0,
        /// faster (+1)
        Faster1 = 1,
    }
}

iap1_bitflags! {
    strings = LingoString;
    pub struct RemoteEventNotificationTrackCapabilityBits: u32 {
        /// is audiobook
        const IS_AUDIOBOOK = 1 << 0;
        /// has chapters
        const HAS_CHAPTERS = 1 << 1;
        /// has album artwork
        const HAS_ALBUM_ARTWORK = 1 << 2;
        /// has song lyrics
        const HAS_SONG_LYRICS = 1 << 3;
        /// is a podcast episode
        const IS_A_PODCAST_EPISODE = 1 << 4;
        /// has release date
        const HAS_RELEASE_DATE = 1 << 5;
        /// has description
        const HAS_DESCRIPTION = 1 << 6;
        /// contains video
        const CONTAINS_VIDEO = 1 << 7;
        /// queued to play as video
        const QUEUED_TO_PLAY_AS_VIDEO = 1 << 8;
        /// capable of generating a Genius playlist
        const CAPABLE_OF_GENERATING_A_GENIUS_PLAYLIST = 1 << 13;
        /// iTunes U episode
        const I_TUNES_U_EPISODE = 1 << 14;
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x03, command = 0x0009,
        source = device,
        response = false,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct RemoteEventNotification {
        fields {
            /// Event Number
            // Virtual schema indices: 1
            required pub event_num: RemoteEventNotificationEventNum => { bit: 0, key: "eventNum", when: (truthy(&true)), predicate_value: true },
            /// Track Position
            // Virtual schema indices: 2, 27
            optional pub track_position: RemoteEventNotificationTrackPositionDisjoint => { bit: 1, key: "trackPosition", when: (eq(&event_num, &0u64)) || (eq(&event_num, &15u64)), predicate_value: false },
            /// Track Playback Index
            // Virtual schema indices: 3, 4
            optional pub track_playback_index: u32 => { bit: 2, key: "trackPlaybackIndex", when: (eq(&event_num, &1u64)) || (eq(&event_num, &2u64)), predicate_value: false },
            /// Chapter Count
            // Virtual schema indices: 5
            optional pub chapter_count: u16 => { bit: 3, key: "chapterCount", when: (eq(&event_num, &2u64)), predicate_value: false },
            /// Chapter Index
            // Virtual schema indices: 6
            optional pub chapter_index: u16 => { bit: 4, key: "chapterIndex", when: (eq(&event_num, &2u64)), predicate_value: false },
            /// Play Status
            // Virtual schema indices: 7
            optional pub play_status: RemoteEventNotificationPlayStatus => { bit: 5, key: "playStatus", when: (eq(&event_num, &3u64)), predicate_value: false },
            /// Mute State
            // Virtual schema indices: 8, 28
            optional pub mute_state: bool => { bit: 6, key: "muteState", when: (eq(&event_num, &4u64)) || (eq(&event_num, &16u64)), predicate_value: false },
            /// UI Volume Level
            // Virtual schema indices: 9, 29
            optional pub ui_volume_level: u8 => { bit: 7, key: "uiVolumeLevel", when: (eq(&event_num, &4u64)) || (eq(&event_num, &16u64)), predicate_value: false },
            /// Power State
            // Virtual schema indices: 10
            optional pub power_state: RemoteEventNotificationPowerState => { bit: 8, key: "powerState", when: (eq(&event_num, &5u64)), predicate_value: false },
            /// Battery Level
            // Virtual schema indices: 11
            optional pub battery_level: u8 => { bit: 9, key: "batteryLevel", when: (eq(&event_num, &5u64)), predicate_value: false },
            /// Equalizer Index
            // Virtual schema indices: 12
            optional pub equalizer_index: u32 => { bit: 10, key: "equalizerIndex", when: (eq(&event_num, &6u64)), predicate_value: false },
            /// Shuffle Setting
            // Virtual schema indices: 13
            optional pub shuffle_setting: RemoteEventNotificationShuffleSetting => { bit: 11, key: "shuffleSetting", when: (eq(&event_num, &7u64)), predicate_value: false },
            /// Repeat Setting
            // Virtual schema indices: 14
            optional pub repeat_setting: RemoteEventNotificationRepeatSetting => { bit: 12, key: "repeatSetting", when: (eq(&event_num, &8u64)), predicate_value: false },
            /// Year
            // Virtual schema indices: 15
            optional pub year: u16 => { bit: 13, key: "year", when: (eq(&event_num, &9u64)), predicate_value: false },
            /// Month
            // Virtual schema indices: 16
            optional pub month: u8 => { bit: 14, key: "month", when: (eq(&event_num, &9u64)), predicate_value: false },
            /// Day
            // Virtual schema indices: 17
            optional pub day: u8 => { bit: 15, key: "day", when: (eq(&event_num, &9u64)), predicate_value: false },
            /// Hour
            // Virtual schema indices: 18
            optional pub hour: u8 => { bit: 16, key: "hour", when: (eq(&event_num, &9u64)), predicate_value: false },
            /// Minute
            // Virtual schema indices: 19
            optional pub minute: u8 => { bit: 17, key: "minute", when: (eq(&event_num, &9u64)), predicate_value: false },
            /// Alarm State
            // Virtual schema indices: 20
            optional pub alarm_state: RemoteEventNotificationAlarmState => { bit: 18, key: "alarmState", when: (eq(&event_num, &10u64)), predicate_value: false },
            /// Alarm Hour
            // Virtual schema indices: 21
            optional pub alarm_hour: u8 => { bit: 19, key: "alarmHour", when: (eq(&event_num, &10u64)), predicate_value: false },
            /// Alarm Minute
            // Virtual schema indices: 22
            optional pub alarm_minute: u8 => { bit: 20, key: "alarmMinute", when: (eq(&event_num, &10u64)), predicate_value: false },
            /// Backlight Level
            // Virtual schema indices: 23
            optional pub backlight_level: u8 => { bit: 21, key: "backlightLevel", when: (eq(&event_num, &11u64)), predicate_value: false },
            /// Hold Switch
            // Virtual schema indices: 24
            optional pub hold_switch: bool => { bit: 22, key: "holdSwitch", when: (eq(&event_num, &12u64)), predicate_value: false },
            /// Sound Check
            // Virtual schema indices: 25
            optional pub sound_check: bool => { bit: 23, key: "soundCheck", when: (eq(&event_num, &13u64)), predicate_value: false },
            /// Audiobook Speed
            // Virtual schema indices: 26
            optional pub audiobook_speed: RemoteEventNotificationAudiobookSpeed => { bit: 24, key: "audiobookSpeed", when: (eq(&event_num, &14u64)), predicate_value: false },
            /// Absolute Volume Level
            // Virtual schema indices: 30
            optional pub absolute_volume_level: u8 => { bit: 25, key: "absoluteVolumeLevel", when: (eq(&event_num, &16u64)), predicate_value: false },
            /// Track Capability Bits
            // Virtual schema indices: 31
            optional pub track_capability_bits: RemoteEventNotificationTrackCapabilityBits => { bit: 26, key: "trackCapabilityBits", when: (eq(&event_num, &17u64)), predicate_value: false },
            /// Number of Tracks in New Playlist
            // Virtual schema indices: 32
            optional pub number_of_tracks_in_new_playlist: u32 => { bit: 27, key: "numberOfTracksInNewPlaylist", when: (eq(&event_num, &18u64)), predicate_value: false },
        }
        steps {
            1 => required event_num: RemoteEventNotificationEventNum => { bit: 0, key: "eventNum", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: true, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            2 => optional track_position: RemoteEventNotificationTrackPositionDisjoint => { bit: 1, key: "trackPosition", wire: [disjoint(decode_track_position, encode_track_position, u32, [scalar])], project_wire: Iap1WireSpec::Disjoint { variant: const { LingoString::require_raw(stringify!(decode_track_position)) }, child: &Iap1WireSpec::Scalar }, predicate_value: false, predicate: { expr: eq(&event_num, &0u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(0u8)] } },
            3 => optional track_playback_index: u32 => { bit: 2, key: "trackPlaybackIndex", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&event_num, &1u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(1u8)] } },
            4 => optional track_playback_index: u32 => { bit: 2, key: "trackPlaybackIndex", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&event_num, &2u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(2u8)] } },
            5 => optional chapter_count: u16 => { bit: 3, key: "chapterCount", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&event_num, &2u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(2u8)] } },
            6 => optional chapter_index: u16 => { bit: 4, key: "chapterIndex", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&event_num, &2u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(2u8)] } },
            7 => optional play_status: RemoteEventNotificationPlayStatus => { bit: 5, key: "playStatus", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&event_num, &3u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(3u8)] } },
            8 => optional mute_state: bool => { bit: 6, key: "muteState", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&event_num, &4u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(4u8)] } },
            9 => optional ui_volume_level: u8 => { bit: 7, key: "uiVolumeLevel", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&event_num, &4u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(4u8)] } },
            10 => optional power_state: RemoteEventNotificationPowerState => { bit: 8, key: "powerState", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&event_num, &5u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(5u8)] } },
            11 => optional battery_level: u8 => { bit: 9, key: "batteryLevel", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&event_num, &5u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(5u8)] } },
            12 => optional equalizer_index: u32 => { bit: 10, key: "equalizerIndex", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&event_num, &6u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(6u8)] } },
            13 => optional shuffle_setting: RemoteEventNotificationShuffleSetting => { bit: 11, key: "shuffleSetting", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&event_num, &7u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(7u8)] } },
            14 => optional repeat_setting: RemoteEventNotificationRepeatSetting => { bit: 12, key: "repeatSetting", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&event_num, &8u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(8u8)] } },
            15 => optional year: u16 => { bit: 13, key: "year", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&event_num, &9u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(9u8)] } },
            16 => optional month: u8 => { bit: 14, key: "month", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&event_num, &9u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(9u8)] } },
            17 => optional day: u8 => { bit: 15, key: "day", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&event_num, &9u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(9u8)] } },
            18 => optional hour: u8 => { bit: 16, key: "hour", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&event_num, &9u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(9u8)] } },
            19 => optional minute: u8 => { bit: 17, key: "minute", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&event_num, &9u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(9u8)] } },
            20 => optional alarm_state: RemoteEventNotificationAlarmState => { bit: 18, key: "alarmState", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&event_num, &10u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(10u8)] } },
            21 => optional alarm_hour: u8 => { bit: 19, key: "alarmHour", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&event_num, &10u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(10u8)] } },
            22 => optional alarm_minute: u8 => { bit: 20, key: "alarmMinute", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&event_num, &10u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(10u8)] } },
            23 => optional backlight_level: u8 => { bit: 21, key: "backlightLevel", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&event_num, &11u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(11u8)] } },
            24 => optional hold_switch: bool => { bit: 22, key: "holdSwitch", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&event_num, &12u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(12u8)] } },
            25 => optional sound_check: bool => { bit: 23, key: "soundCheck", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&event_num, &13u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(13u8)] } },
            26 => optional audiobook_speed: RemoteEventNotificationAudiobookSpeed => { bit: 24, key: "audiobookSpeed", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&event_num, &14u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(14u8)] } },
            27 => optional track_position: RemoteEventNotificationTrackPositionDisjoint => { bit: 1, key: "trackPosition", wire: [disjoint(decode_track_position2, encode_track_position2, u16, [scalar])], project_wire: Iap1WireSpec::Disjoint { variant: const { LingoString::require_raw(stringify!(decode_track_position2)) }, child: &Iap1WireSpec::Scalar }, predicate_value: false, predicate: { expr: eq(&event_num, &15u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(15u8)] } },
            28 => optional mute_state: bool => { bit: 6, key: "muteState", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&event_num, &16u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(16u8)] } },
            29 => optional ui_volume_level: u8 => { bit: 7, key: "uiVolumeLevel", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&event_num, &16u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(16u8)] } },
            30 => optional absolute_volume_level: u8 => { bit: 25, key: "absoluteVolumeLevel", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&event_num, &16u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(16u8)] } },
            31 => optional track_capability_bits: RemoteEventNotificationTrackCapabilityBits => { bit: 26, key: "trackCapabilityBits", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&event_num, &17u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(17u8)] } },
            32 => optional number_of_tracks_in_new_playlist: u32 => { bit: 27, key: "numberOfTracksInNewPlaylist", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&event_num, &18u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(18u8)] } },
        }
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x03, command = 0x000a,
        source = accessory,
        response = false,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct GetRemoteEventStatus {
        fields {
        }
        steps {
        }
    }
}

iap1_bitflags! {
    strings = LingoString;
    pub struct RetRemoteEventStatusRemoteEventMask: u32 {
        /// track time position in milliseconds
        const TRACK_TIME_POSITION_IN_MILLISECONDS = 1 << 0;
        /// track playback index
        const TRACK_PLAYBACK_INDEX = 1 << 1;
        /// chapter index
        const CHAPTER_INDEX = 1 << 2;
        /// play status
        const PLAY_STATUS = 1 << 3;
        /// mute/UI volume
        const MUTE_UI_VOLUME = 1 << 4;
        /// power/battery
        const POWER_BATTERY = 1 << 5;
        /// equalizer setting
        const EQUALIZER_SETTING = 1 << 6;
        /// shuffle setting
        const SHUFFLE_SETTING = 1 << 7;
        /// repeat setting
        const REPEAT_SETTING = 1 << 8;
        /// date and time setting
        const DATE_AND_TIME_SETTING = 1 << 9;
        /// alarm setting
        const ALARM_SETTING = 1 << 10;
        /// backlight level
        const BACKLIGHT_LEVEL = 1 << 11;
        /// hold switch state
        const HOLD_SWITCH_STATE = 1 << 12;
        /// sound check state
        const SOUND_CHECK_STATE = 1 << 13;
        /// audiobook state
        const AUDIOBOOK_STATE = 1 << 14;
        /// track time position in seconds
        const TRACK_TIME_POSITION_IN_SECONDS = 1 << 15;
        /// mute/UI/absolute volume
        const MUTE_UI_ABSOLUTE_VOLUME = 1 << 16;
        /// track capabilities
        const TRACK_CAPABILITIES = 1 << 17;
        /// playback engine contents changed
        const PLAYBACK_ENGINE_CONTENTS_CHANGED = 1 << 18;
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x03, command = 0x000b,
        source = device,
        response = true,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct RetRemoteEventStatus {
        fields {
            /// Remote Event Mask
            // Virtual schema indices: 1
            required pub remote_event_mask: RetRemoteEventStatusRemoteEventMask => { bit: 0, key: "remoteEventMask", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required remote_event_mask: RetRemoteEventStatusRemoteEventMask => { bit: 0, key: "remoteEventMask", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum GetiPodStateInfoInfoType: u8 {
        /// track time position in milliseconds
        TrackTimePositionInMilliseconds = 0,
        /// track playback index
        TrackPlaybackIndex = 1,
        /// chapter information
        ChapterInformation = 2,
        /// play status
        PlayStatus = 3,
        /// mute and UI volume information
        MuteAndUIVolumeInformation = 4,
        /// power and battery status
        PowerAndBatteryStatus = 5,
        /// equalizer setting
        EqualizerSetting = 6,
        /// shuffle setting
        ShuffleSetting = 7,
        /// repeat setting
        RepeatSetting = 8,
        /// date and time
        DateAndTime = 9,
        /// alarm state and time
        AlarmStateAndTime = 10,
        /// backlight level
        BacklightLevel = 11,
        /// hold switch state
        HoldSwitchState = 12,
        /// sound check state
        SoundCheckState = 13,
        /// audiobook speed
        AudiobookSpeed = 14,
        /// track time position in seconds
        TrackTimePositionInSeconds = 15,
        /// mute/UI/absolute volume
        MuteUIAbsoluteVolume = 16,
        /// track capabilities
        TrackCapabilities = 17,
        /// playback engine contents changed
        PlaybackEngineContentsChanged = 18,
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x03, command = 0x000c,
        source = accessory,
        response = false,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct GetiPodStateInfo {
        fields {
            /// Info Type
            // Virtual schema indices: 1
            required pub info_type: GetiPodStateInfoInfoType => { bit: 0, key: "infoType", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required info_type: GetiPodStateInfoInfoType => { bit: 0, key: "infoType", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum RetiPodStateInfoInfoType: u8 {
        /// track time position in milliseconds
        TrackTimePositionInMilliseconds = 0,
        /// track playback index
        TrackPlaybackIndex = 1,
        /// chapter information
        ChapterInformation = 2,
        /// play status
        PlayStatus = 3,
        /// mute and UI volume information
        MuteAndUIVolumeInformation = 4,
        /// power and battery status
        PowerAndBatteryStatus = 5,
        /// equalizer setting
        EqualizerSetting = 6,
        /// shuffle setting
        ShuffleSetting = 7,
        /// repeat setting
        RepeatSetting = 8,
        /// date and time
        DateAndTime = 9,
        /// alarm state and time
        AlarmStateAndTime = 10,
        /// backlight level
        BacklightLevel = 11,
        /// hold switch state
        HoldSwitchState = 12,
        /// sound check state
        SoundCheckState = 13,
        /// audiobook speed
        AudiobookSpeed = 14,
        /// track time position in seconds
        TrackTimePositionInSeconds = 15,
        /// mute/UI/absolute volume
        MuteUIAbsoluteVolume = 16,
        /// track capabilities
        TrackCapabilities = 17,
        /// playback engine contents changed
        PlaybackEngineContentsChanged = 18,
    }
}

iap1_disjoint! {
    strings = LingoString;
    #[iap1_disjoint(predicate = integer)]
    pub enum RetiPodStateInfoTrackPositionDisjoint {
        #[iap1_disjoint(decode = decode_track_position, encode = encode_track_position)]
        /// Track Position
        TrackPosition(u32),
        #[iap1_disjoint(decode = decode_track_position2, encode = encode_track_position2)]
        /// Track Position
        TrackPosition2(u16),
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum RetiPodStateInfoPlayStatus: u8 {
        /// playback stopped
        PlaybackStopped = 0,
        /// playing
        Playing = 1,
        /// playback paused
        PlaybackPaused = 2,
        /// fast forward
        FastForward = 3,
        /// fast rewind
        FastRewind = 4,
        /// end fast forward or rewind mode
        EndFastForwardOrRewindMode = 5,
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum RetiPodStateInfoPowerState: u8 {
        /// internal battery power, low power (< 30%)
        InternalBatteryPowerLowPower30 = 0,
        /// internal battery power
        InternalBatteryPower = 1,
        /// external power, battery pack, no charging
        ExternalPowerBatteryPackNoCharging = 2,
        /// external power, battery charging
        ExternalPowerBatteryCharging = 3,
        /// external power, battery charged
        ExternalPowerBatteryCharged = 4,
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum RetiPodStateInfoShuffleSetting: u8 {
        /// off
        Off = 0,
        /// tracks and songs
        TracksAndSongs = 1,
        /// albums
        Albums = 2,
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum RetiPodStateInfoRepeatSetting: u8 {
        /// off
        Off = 0,
        /// one track or song
        OneTrackOrSong = 1,
        /// all tracks
        AllTracks = 2,
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum RetiPodStateInfoAlarmState: u8 {
        /// off
        Off = 0,
        /// enabled
        Enabled = 1,
        /// triggered
        Triggered = 2,
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum RetiPodStateInfoAudiobookSpeed: u8 {
        /// slower (-1)
        Slower1 = 255,
        /// normal
        Normal = 0,
        /// faster (+1)
        Faster1 = 1,
    }
}

iap1_bitflags! {
    strings = LingoString;
    pub struct RetiPodStateInfoTrackCapabilityBits: u32 {
        /// is audiobook
        const IS_AUDIOBOOK = 1 << 0;
        /// has chapters
        const HAS_CHAPTERS = 1 << 1;
        /// has album artwork
        const HAS_ALBUM_ARTWORK = 1 << 2;
        /// has song lyrics
        const HAS_SONG_LYRICS = 1 << 3;
        /// is a podcast episode
        const IS_A_PODCAST_EPISODE = 1 << 4;
        /// has release date
        const HAS_RELEASE_DATE = 1 << 5;
        /// has description
        const HAS_DESCRIPTION = 1 << 6;
        /// contains video
        const CONTAINS_VIDEO = 1 << 7;
        /// queued to play as video
        const QUEUED_TO_PLAY_AS_VIDEO = 1 << 8;
        /// capable of generating a Genius playlist
        const CAPABLE_OF_GENERATING_A_GENIUS_PLAYLIST = 1 << 13;
        /// iTunes U episode
        const I_TUNES_U_EPISODE = 1 << 14;
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x03, command = 0x000d,
        source = device,
        response = true,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct RetiPodStateInfo {
        fields {
            /// Info Type
            // Virtual schema indices: 1
            required pub info_type: RetiPodStateInfoInfoType => { bit: 0, key: "infoType", when: (truthy(&true)), predicate_value: true },
            /// Track Position
            // Virtual schema indices: 2, 27
            optional pub track_position: RetiPodStateInfoTrackPositionDisjoint => { bit: 1, key: "trackPosition", when: (eq(&info_type, &0u64)) || (eq(&info_type, &15u64)), predicate_value: false },
            /// Currently Playing Track Index
            // Virtual schema indices: 3, 4
            optional pub track_index: i32 => { bit: 2, key: "trackIndex", when: (eq(&info_type, &1u64)) || (eq(&info_type, &2u64)), predicate_value: false },
            /// Chapter Count
            // Virtual schema indices: 5
            optional pub chapter_count: i16 => { bit: 3, key: "chapterCount", when: (eq(&info_type, &2u64)), predicate_value: false },
            /// Chapter Index
            // Virtual schema indices: 6
            optional pub chapter_index: i16 => { bit: 4, key: "chapterIndex", when: (eq(&info_type, &2u64)), predicate_value: false },
            /// Play Status
            // Virtual schema indices: 7
            optional pub play_status: RetiPodStateInfoPlayStatus => { bit: 5, key: "playStatus", when: (eq(&info_type, &3u64)), predicate_value: false },
            /// Mute State
            // Virtual schema indices: 8, 28
            optional pub mute_state: bool => { bit: 6, key: "muteState", when: (eq(&info_type, &4u64)) || (eq(&info_type, &16u64)), predicate_value: false },
            /// UI Volume Level
            // Virtual schema indices: 9, 29
            optional pub ui_volume_level: u8 => { bit: 7, key: "uiVolumeLevel", when: (eq(&info_type, &4u64)) || (eq(&info_type, &16u64)), predicate_value: false },
            /// Power State
            // Virtual schema indices: 10
            optional pub power_state: RetiPodStateInfoPowerState => { bit: 8, key: "powerState", when: (eq(&info_type, &5u64)), predicate_value: false },
            /// Battery Level
            // Virtual schema indices: 11
            optional pub battery_level: u8 => { bit: 9, key: "batteryLevel", when: (eq(&info_type, &5u64)), predicate_value: false },
            /// Equalizer Index
            // Virtual schema indices: 12
            optional pub equalizer_index: u32 => { bit: 10, key: "equalizerIndex", when: (eq(&info_type, &6u64)), predicate_value: false },
            /// Shuffle Setting
            // Virtual schema indices: 13
            optional pub shuffle_setting: RetiPodStateInfoShuffleSetting => { bit: 11, key: "shuffleSetting", when: (eq(&info_type, &7u64)), predicate_value: false },
            /// Repeat Setting
            // Virtual schema indices: 14
            optional pub repeat_setting: RetiPodStateInfoRepeatSetting => { bit: 12, key: "repeatSetting", when: (eq(&info_type, &8u64)), predicate_value: false },
            /// Year
            // Virtual schema indices: 15
            optional pub year: u16 => { bit: 13, key: "year", when: (eq(&info_type, &9u64)), predicate_value: false },
            /// Month
            // Virtual schema indices: 16
            optional pub month: u8 => { bit: 14, key: "month", when: (eq(&info_type, &9u64)), predicate_value: false },
            /// Day
            // Virtual schema indices: 17
            optional pub day: u8 => { bit: 15, key: "day", when: (eq(&info_type, &9u64)), predicate_value: false },
            /// Hour
            // Virtual schema indices: 18
            optional pub hour: u8 => { bit: 16, key: "hour", when: (eq(&info_type, &9u64)), predicate_value: false },
            /// Minute
            // Virtual schema indices: 19
            optional pub minute: u8 => { bit: 17, key: "minute", when: (eq(&info_type, &9u64)), predicate_value: false },
            /// Alarm State
            // Virtual schema indices: 20
            optional pub alarm_state: RetiPodStateInfoAlarmState => { bit: 18, key: "alarmState", when: (eq(&info_type, &10u64)), predicate_value: false },
            /// Alarm Hour
            // Virtual schema indices: 21
            optional pub alarm_hour: u8 => { bit: 19, key: "alarmHour", when: (eq(&info_type, &10u64)), predicate_value: false },
            /// Alarm Minute
            // Virtual schema indices: 22
            optional pub alarm_minute: u8 => { bit: 20, key: "alarmMinute", when: (eq(&info_type, &10u64)), predicate_value: false },
            /// Backlight Level
            // Virtual schema indices: 23
            optional pub backlight_level: u8 => { bit: 21, key: "backlightLevel", when: (eq(&info_type, &11u64)), predicate_value: false },
            /// Hold Switch
            // Virtual schema indices: 24
            optional pub hold_switch: bool => { bit: 22, key: "holdSwitch", when: (eq(&info_type, &12u64)), predicate_value: false },
            /// Sound Check
            // Virtual schema indices: 25
            optional pub sound_check: bool => { bit: 23, key: "soundCheck", when: (eq(&info_type, &13u64)), predicate_value: false },
            /// Audiobook Speed
            // Virtual schema indices: 26
            optional pub audiobook_speed: RetiPodStateInfoAudiobookSpeed => { bit: 24, key: "audiobookSpeed", when: (eq(&info_type, &14u64)), predicate_value: false },
            /// Absolute Volume Level
            // Virtual schema indices: 30
            optional pub absolute_volume_level: u8 => { bit: 25, key: "absoluteVolumeLevel", when: (eq(&info_type, &16u64)), predicate_value: false },
            /// Track Capability Bits
            // Virtual schema indices: 31
            optional pub track_capability_bits: RetiPodStateInfoTrackCapabilityBits => { bit: 26, key: "trackCapabilityBits", when: (eq(&info_type, &17u64)), predicate_value: false },
            /// Number of Tracks in New Playlist
            // Virtual schema indices: 32
            optional pub number_of_tracks_in_new_playlist: u32 => { bit: 27, key: "numberOfTracksInNewPlaylist", when: (eq(&info_type, &18u64)), predicate_value: false },
        }
        steps {
            1 => required info_type: RetiPodStateInfoInfoType => { bit: 0, key: "infoType", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: true, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            2 => optional track_position: RetiPodStateInfoTrackPositionDisjoint => { bit: 1, key: "trackPosition", wire: [disjoint(decode_track_position, encode_track_position, u32, [scalar])], project_wire: Iap1WireSpec::Disjoint { variant: const { LingoString::require_raw(stringify!(decode_track_position)) }, child: &Iap1WireSpec::Scalar }, predicate_value: false, predicate: { expr: eq(&info_type, &0u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(0u8)] } },
            3 => optional track_index: i32 => { bit: 2, key: "trackIndex", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&info_type, &1u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(1u8)] } },
            4 => optional track_index: i32 => { bit: 2, key: "trackIndex", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&info_type, &2u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(2u8)] } },
            5 => optional chapter_count: i16 => { bit: 3, key: "chapterCount", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&info_type, &2u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(2u8)] } },
            6 => optional chapter_index: i16 => { bit: 4, key: "chapterIndex", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&info_type, &2u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(2u8)] } },
            7 => optional play_status: RetiPodStateInfoPlayStatus => { bit: 5, key: "playStatus", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&info_type, &3u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(3u8)] } },
            8 => optional mute_state: bool => { bit: 6, key: "muteState", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&info_type, &4u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(4u8)] } },
            9 => optional ui_volume_level: u8 => { bit: 7, key: "uiVolumeLevel", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&info_type, &4u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(4u8)] } },
            10 => optional power_state: RetiPodStateInfoPowerState => { bit: 8, key: "powerState", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&info_type, &5u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(5u8)] } },
            11 => optional battery_level: u8 => { bit: 9, key: "batteryLevel", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&info_type, &5u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(5u8)] } },
            12 => optional equalizer_index: u32 => { bit: 10, key: "equalizerIndex", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&info_type, &6u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(6u8)] } },
            13 => optional shuffle_setting: RetiPodStateInfoShuffleSetting => { bit: 11, key: "shuffleSetting", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&info_type, &7u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(7u8)] } },
            14 => optional repeat_setting: RetiPodStateInfoRepeatSetting => { bit: 12, key: "repeatSetting", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&info_type, &8u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(8u8)] } },
            15 => optional year: u16 => { bit: 13, key: "year", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&info_type, &9u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(9u8)] } },
            16 => optional month: u8 => { bit: 14, key: "month", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&info_type, &9u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(9u8)] } },
            17 => optional day: u8 => { bit: 15, key: "day", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&info_type, &9u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(9u8)] } },
            18 => optional hour: u8 => { bit: 16, key: "hour", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&info_type, &9u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(9u8)] } },
            19 => optional minute: u8 => { bit: 17, key: "minute", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&info_type, &9u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(9u8)] } },
            20 => optional alarm_state: RetiPodStateInfoAlarmState => { bit: 18, key: "alarmState", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&info_type, &10u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(10u8)] } },
            21 => optional alarm_hour: u8 => { bit: 19, key: "alarmHour", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&info_type, &10u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(10u8)] } },
            22 => optional alarm_minute: u8 => { bit: 20, key: "alarmMinute", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&info_type, &10u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(10u8)] } },
            23 => optional backlight_level: u8 => { bit: 21, key: "backlightLevel", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&info_type, &11u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(11u8)] } },
            24 => optional hold_switch: bool => { bit: 22, key: "holdSwitch", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&info_type, &12u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(12u8)] } },
            25 => optional sound_check: bool => { bit: 23, key: "soundCheck", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&info_type, &13u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(13u8)] } },
            26 => optional audiobook_speed: RetiPodStateInfoAudiobookSpeed => { bit: 24, key: "audiobookSpeed", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&info_type, &14u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(14u8)] } },
            27 => optional track_position: RetiPodStateInfoTrackPositionDisjoint => { bit: 1, key: "trackPosition", wire: [disjoint(decode_track_position2, encode_track_position2, u16, [scalar])], project_wire: Iap1WireSpec::Disjoint { variant: const { LingoString::require_raw(stringify!(decode_track_position2)) }, child: &Iap1WireSpec::Scalar }, predicate_value: false, predicate: { expr: eq(&info_type, &15u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(15u8)] } },
            28 => optional mute_state: bool => { bit: 6, key: "muteState", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&info_type, &16u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(16u8)] } },
            29 => optional ui_volume_level: u8 => { bit: 7, key: "uiVolumeLevel", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&info_type, &16u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(16u8)] } },
            30 => optional absolute_volume_level: u8 => { bit: 25, key: "absoluteVolumeLevel", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&info_type, &16u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(16u8)] } },
            31 => optional track_capability_bits: RetiPodStateInfoTrackCapabilityBits => { bit: 26, key: "trackCapabilityBits", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&info_type, &17u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(17u8)] } },
            32 => optional number_of_tracks_in_new_playlist: u32 => { bit: 27, key: "numberOfTracksInNewPlaylist", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&info_type, &18u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(18u8)] } },
        }
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum SetiPodStateInfoInfoType: u8 {
        /// track time position in milliseconds
        TrackTimePositionInMilliseconds = 0,
        /// track playback index
        TrackPlaybackIndex = 1,
        /// chapter information
        ChapterInformation = 2,
        /// play status
        PlayStatus = 3,
        /// mute and UI volume information
        MuteAndUIVolumeInformation = 4,
        /// equalizer setting
        EqualizerSetting = 6,
        /// shuffle setting
        ShuffleSetting = 7,
        /// repeat setting
        RepeatSetting = 8,
        /// date and time
        DateAndTime = 9,
        /// alarm state and time
        AlarmStateAndTime = 10,
        /// backlight level
        BacklightLevel = 11,
        /// audiobook speed
        AudiobookSpeed = 14,
        /// track time position in seconds
        TrackTimePositionInSeconds = 15,
        /// mute/UI/absolute volume
        MuteUIAbsoluteVolume = 16,
    }
}

iap1_disjoint! {
    strings = LingoString;
    #[iap1_disjoint(predicate = integer)]
    pub enum SetiPodStateInfoTrackPositionDisjoint {
        #[iap1_disjoint(decode = decode_track_position, encode = encode_track_position)]
        /// Track Position
        TrackPosition(u32),
        #[iap1_disjoint(decode = decode_track_position2, encode = encode_track_position2)]
        /// Track Position
        TrackPosition2(u16),
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum SetiPodStateInfoPlayStatus: u8 {
        /// playback stopped
        PlaybackStopped = 0,
        /// playing
        Playing = 1,
        /// playback paused
        PlaybackPaused = 2,
        /// fast forward
        FastForward = 3,
        /// fast rewind
        FastRewind = 4,
        /// end fast forward or rewind mode
        EndFastForwardOrRewindMode = 5,
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum SetiPodStateInfoShuffleSetting: u8 {
        /// off
        Off = 0,
        /// tracks and songs
        TracksAndSongs = 1,
        /// albums
        Albums = 2,
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum SetiPodStateInfoRepeatSetting: u8 {
        /// off
        Off = 0,
        /// one track or song
        OneTrackOrSong = 1,
        /// all tracks
        AllTracks = 2,
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum SetiPodStateInfoAudiobookSpeed: u8 {
        /// slower (-1)
        Slower1 = 255,
        /// normal
        Normal = 0,
        /// faster (+1)
        Faster1 = 1,
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x03, command = 0x000e,
        source = accessory,
        response = false,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct SetiPodStateInfo {
        fields {
            /// Info Type
            // Virtual schema indices: 1
            required pub info_type: SetiPodStateInfoInfoType => { bit: 0, key: "infoType", when: (truthy(&true)), predicate_value: true },
            /// Track Position
            // Virtual schema indices: 2, 30
            optional pub track_position: SetiPodStateInfoTrackPositionDisjoint => { bit: 1, key: "trackPosition", when: (eq(&info_type, &0u64)) || (eq(&info_type, &15u64)), predicate_value: false },
            /// Currently Playing Track Index
            // Virtual schema indices: 3
            optional pub track_index: u32 => { bit: 2, key: "trackIndex", when: (eq(&info_type, &1u64)), predicate_value: false },
            /// Chapter Index
            // Virtual schema indices: 4
            optional pub chapter_index: u16 => { bit: 3, key: "chapterIndex", when: (eq(&info_type, &2u64)), predicate_value: false },
            /// Play Status
            // Virtual schema indices: 5
            optional pub play_status: SetiPodStateInfoPlayStatus => { bit: 4, key: "playStatus", when: (eq(&info_type, &3u64)), predicate_value: false },
            /// Mute State
            // Virtual schema indices: 6, 31
            optional pub mute_state: bool => { bit: 5, key: "muteState", when: (eq(&info_type, &4u64)) || (eq(&info_type, &16u64)), predicate_value: false },
            /// UI Volume Level
            // Virtual schema indices: 7, 32
            optional pub ui_volume_level: u8 => { bit: 6, key: "uiVolumeLevel", when: (eq(&info_type, &4u64)) || (eq(&info_type, &16u64)), predicate_value: false },
            /// Restore on Exit
            // Virtual schema indices: 8, 10, 12, 14, 23, 27, 29, 34
            optional pub restore_on_exit: bool => { bit: 7, key: "restoreOnExit", when: (eq(&info_type, &4u64)) || (eq(&info_type, &6u64)) || (eq(&info_type, &7u64)) || (eq(&info_type, &8u64)) || (eq(&info_type, &10u64)) || (eq(&info_type, &13u64)) || (eq(&info_type, &14u64)) || (eq(&info_type, &16u64)), predicate_value: false },
            /// Equalizer Index
            // Virtual schema indices: 9
            optional pub equalizer_index: u32 => { bit: 8, key: "equalizerIndex", when: (eq(&info_type, &6u64)), predicate_value: false },
            /// Shuffle Setting
            // Virtual schema indices: 11
            optional pub shuffle_setting: SetiPodStateInfoShuffleSetting => { bit: 9, key: "shuffleSetting", when: (eq(&info_type, &7u64)), predicate_value: false },
            /// Repeat Setting
            // Virtual schema indices: 13
            optional pub repeat_setting: SetiPodStateInfoRepeatSetting => { bit: 10, key: "repeatSetting", when: (eq(&info_type, &8u64)), predicate_value: false },
            /// Year
            // Virtual schema indices: 15
            optional pub year: u16 => { bit: 11, key: "year", when: (eq(&info_type, &9u64)), predicate_value: false },
            /// Month
            // Virtual schema indices: 16
            optional pub month: u8 => { bit: 12, key: "month", when: (eq(&info_type, &9u64)), predicate_value: false },
            /// Day
            // Virtual schema indices: 17
            optional pub day: u8 => { bit: 13, key: "day", when: (eq(&info_type, &9u64)), predicate_value: false },
            /// Hour
            // Virtual schema indices: 18
            optional pub hour: u8 => { bit: 14, key: "hour", when: (eq(&info_type, &9u64)), predicate_value: false },
            /// Minute
            // Virtual schema indices: 19
            optional pub minute: u8 => { bit: 15, key: "minute", when: (eq(&info_type, &9u64)), predicate_value: false },
            /// Alarm State
            // Virtual schema indices: 20
            optional pub alarm_state: bool => { bit: 16, key: "alarmState", when: (eq(&info_type, &10u64)), predicate_value: false },
            /// Alarm Hour
            // Virtual schema indices: 21
            optional pub alarm_hour: u8 => { bit: 17, key: "alarmHour", when: (eq(&info_type, &10u64)), predicate_value: false },
            /// Alarm Minute
            // Virtual schema indices: 22
            optional pub alarm_minute: u8 => { bit: 18, key: "alarmMinute", when: (eq(&info_type, &10u64)), predicate_value: false },
            /// Backlight Level
            // Virtual schema indices: 24
            optional pub backlight_level: u8 => { bit: 19, key: "backlightLevel", when: (eq(&info_type, &11u64)), predicate_value: false },
            /// Reserved
            // Virtual schema indices: 25
            optional pub reserved: u8 => { bit: 20, key: "reserved", when: (eq(&info_type, &11u64)), predicate_value: false },
            /// Sound Check
            // Virtual schema indices: 26
            optional pub sound_check: bool => { bit: 21, key: "soundCheck", when: (eq(&info_type, &13u64)), predicate_value: false },
            /// Audiobook Speed
            // Virtual schema indices: 28
            optional pub audiobook_speed: SetiPodStateInfoAudiobookSpeed => { bit: 22, key: "audiobookSpeed", when: (eq(&info_type, &14u64)), predicate_value: false },
            /// Absolute Volume Level
            // Virtual schema indices: 33
            optional pub absolute_volume_level: u8 => { bit: 23, key: "absoluteVolumeLevel", when: (eq(&info_type, &16u64)), predicate_value: false },
        }
        steps {
            1 => required info_type: SetiPodStateInfoInfoType => { bit: 0, key: "infoType", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: true, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            2 => optional track_position: SetiPodStateInfoTrackPositionDisjoint => { bit: 1, key: "trackPosition", wire: [disjoint(decode_track_position, encode_track_position, u32, [scalar])], project_wire: Iap1WireSpec::Disjoint { variant: const { LingoString::require_raw(stringify!(decode_track_position)) }, child: &Iap1WireSpec::Scalar }, predicate_value: false, predicate: { expr: eq(&info_type, &0u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(0u8)] } },
            3 => optional track_index: u32 => { bit: 2, key: "trackIndex", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&info_type, &1u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(1u8)] } },
            4 => optional chapter_index: u16 => { bit: 3, key: "chapterIndex", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&info_type, &2u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(2u8)] } },
            5 => optional play_status: SetiPodStateInfoPlayStatus => { bit: 4, key: "playStatus", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&info_type, &3u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(3u8)] } },
            6 => optional mute_state: bool => { bit: 5, key: "muteState", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&info_type, &4u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(4u8)] } },
            7 => optional ui_volume_level: u8 => { bit: 6, key: "uiVolumeLevel", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&info_type, &4u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(4u8)] } },
            8 => optional restore_on_exit: bool => { bit: 7, key: "restoreOnExit", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&info_type, &4u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(4u8)] } },
            9 => optional equalizer_index: u32 => { bit: 8, key: "equalizerIndex", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&info_type, &6u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(6u8)] } },
            10 => optional restore_on_exit: bool => { bit: 7, key: "restoreOnExit", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&info_type, &6u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(6u8)] } },
            11 => optional shuffle_setting: SetiPodStateInfoShuffleSetting => { bit: 9, key: "shuffleSetting", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&info_type, &7u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(7u8)] } },
            12 => optional restore_on_exit: bool => { bit: 7, key: "restoreOnExit", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&info_type, &7u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(7u8)] } },
            13 => optional repeat_setting: SetiPodStateInfoRepeatSetting => { bit: 10, key: "repeatSetting", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&info_type, &8u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(8u8)] } },
            14 => optional restore_on_exit: bool => { bit: 7, key: "restoreOnExit", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&info_type, &8u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(8u8)] } },
            15 => optional year: u16 => { bit: 11, key: "year", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&info_type, &9u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(9u8)] } },
            16 => optional month: u8 => { bit: 12, key: "month", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&info_type, &9u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(9u8)] } },
            17 => optional day: u8 => { bit: 13, key: "day", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&info_type, &9u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(9u8)] } },
            18 => optional hour: u8 => { bit: 14, key: "hour", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&info_type, &9u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(9u8)] } },
            19 => optional minute: u8 => { bit: 15, key: "minute", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&info_type, &9u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(9u8)] } },
            20 => optional alarm_state: bool => { bit: 16, key: "alarmState", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&info_type, &10u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(10u8)] } },
            21 => optional alarm_hour: u8 => { bit: 17, key: "alarmHour", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&info_type, &10u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(10u8)] } },
            22 => optional alarm_minute: u8 => { bit: 18, key: "alarmMinute", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&info_type, &10u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(10u8)] } },
            23 => optional restore_on_exit: bool => { bit: 7, key: "restoreOnExit", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&info_type, &10u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(10u8)] } },
            24 => optional backlight_level: u8 => { bit: 19, key: "backlightLevel", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&info_type, &11u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(11u8)] } },
            25 => optional reserved: u8 => { bit: 20, key: "reserved", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&info_type, &11u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(11u8)] } },
            26 => optional sound_check: bool => { bit: 21, key: "soundCheck", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&info_type, &13u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(13u8)] } },
            27 => optional restore_on_exit: bool => { bit: 7, key: "restoreOnExit", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&info_type, &13u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(13u8)] } },
            28 => optional audiobook_speed: SetiPodStateInfoAudiobookSpeed => { bit: 22, key: "audiobookSpeed", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&info_type, &14u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(14u8)] } },
            29 => optional restore_on_exit: bool => { bit: 7, key: "restoreOnExit", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&info_type, &14u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(14u8)] } },
            30 => optional track_position: SetiPodStateInfoTrackPositionDisjoint => { bit: 1, key: "trackPosition", wire: [disjoint(decode_track_position2, encode_track_position2, u16, [scalar])], project_wire: Iap1WireSpec::Disjoint { variant: const { LingoString::require_raw(stringify!(decode_track_position2)) }, child: &Iap1WireSpec::Scalar }, predicate_value: false, predicate: { expr: eq(&info_type, &15u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(15u8)] } },
            31 => optional mute_state: bool => { bit: 5, key: "muteState", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&info_type, &16u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(16u8)] } },
            32 => optional ui_volume_level: u8 => { bit: 6, key: "uiVolumeLevel", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&info_type, &16u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(16u8)] } },
            33 => optional absolute_volume_level: u8 => { bit: 23, key: "absoluteVolumeLevel", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&info_type, &16u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(16u8)] } },
            34 => optional restore_on_exit: bool => { bit: 7, key: "restoreOnExit", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&info_type, &16u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(16u8)] } },
        }
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x03, command = 0x000f,
        source = accessory,
        response = false,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct GetPlayStatus {
        fields {
        }
        steps {
        }
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum RetPlayStatusPlayState: u8 {
        /// playback stopped
        PlaybackStopped = 0,
        /// playing
        Playing = 1,
        /// playback paused
        PlaybackPaused = 2,
        /// fast forward
        FastForward = 3,
        /// fast rewind
        FastRewind = 4,
        /// end fast forward or rewind mode
        EndFastForwardOrRewindMode = 5,
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x03, command = 0x0010,
        source = device,
        response = true,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct RetPlayStatus {
        fields {
            /// Playback Engine State
            // Virtual schema indices: 1
            required pub play_state: RetPlayStatusPlayState => { bit: 0, key: "playState", when: (truthy(&true)), predicate_value: false },
            /// Currently Playing Track Index
            // Virtual schema indices: 2
            required pub track_index: u32 => { bit: 1, key: "trackIndex", when: (truthy(&true)), predicate_value: false },
            /// Total Length of Track
            // Virtual schema indices: 3
            required pub track_total_length: u32 => { bit: 2, key: "trackTotalLength", when: (truthy(&true)), predicate_value: false },
            /// Current Position of Track
            // Virtual schema indices: 4
            required pub track_position: u32 => { bit: 3, key: "trackPosition", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required play_state: RetPlayStatusPlayState => { bit: 0, key: "playState", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            2 => required track_index: u32 => { bit: 1, key: "trackIndex", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            3 => required track_total_length: u32 => { bit: 2, key: "trackTotalLength", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            4 => required track_position: u32 => { bit: 3, key: "trackPosition", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x03, command = 0x0011,
        source = accessory,
        response = false,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct SetCurrentPlayingTrack {
        fields {
            /// Track Index to Play
            // Virtual schema indices: 1
            required pub track_index_to_play: u32 => { bit: 0, key: "trackIndexToPlay", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required track_index_to_play: u32 => { bit: 0, key: "trackIndexToPlay", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum GetIndexedPlayingTrackInfoInfoType: u8 {
        /// track caps/info
        TrackCapsInfo = 0,
        /// chapter time/name
        ChapterTimeName = 1,
        /// artist name
        ArtistName = 2,
        /// album name
        AlbumName = 3,
        /// genre name
        GenreName = 4,
        /// track title
        TrackTitle = 5,
        /// composer name
        ComposerName = 6,
        /// lyrics
        Lyrics = 7,
        /// artwork count
        ArtworkCount = 8,
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x03, command = 0x0012,
        source = accessory,
        response = false,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct GetIndexedPlayingTrackInfo {
        fields {
            /// Info Type
            // Virtual schema indices: 1
            required pub info_type: GetIndexedPlayingTrackInfoInfoType => { bit: 0, key: "infoType", when: (truthy(&true)), predicate_value: false },
            /// Track Index
            // Virtual schema indices: 2
            required pub track_index: u32 => { bit: 1, key: "trackIndex", when: (truthy(&true)), predicate_value: false },
            /// Chapter Index
            // Virtual schema indices: 3
            required pub chapter_index: u16 => { bit: 2, key: "chapterIndex", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required info_type: GetIndexedPlayingTrackInfoInfoType => { bit: 0, key: "infoType", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            2 => required track_index: u32 => { bit: 1, key: "trackIndex", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            3 => required chapter_index: u16 => { bit: 2, key: "chapterIndex", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum RetIndexedPlayingTrackInfoInfoType: u8 {
        /// track caps/info
        TrackCapsInfo = 0,
        /// chapter time/name
        ChapterTimeName = 1,
        /// artist name
        ArtistName = 2,
        /// album name
        AlbumName = 3,
        /// genre name
        GenreName = 4,
        /// track title
        TrackTitle = 5,
        /// composer name
        ComposerName = 6,
        /// lyrics
        Lyrics = 7,
        /// artwork count
        ArtworkCount = 8,
    }
}

iap1_bitflags! {
    strings = LingoString;
    pub struct RetIndexedPlayingTrackInfoTrackCaps: u32 {
        /// audiobook
        const AUDIOBOOK = 1 << 0;
        /// has chapters
        const HAS_CHAPTERS = 1 << 1;
        /// album artwork
        const ALBUM_ARTWORK = 1 << 2;
        /// lyrics
        const LYRICS = 1 << 3;
        /// contains video
        const CONTAINS_VIDEO = 1 << 7;
        /// queued to play as video
        const QUEUED_TO_PLAY_AS_VIDEO = 1 << 8;
        /// capable of generating a Genius playlist
        const CAPABLE_OF_GENERATING_A_GENIUS_PLAYLIST = 1 << 13;
        /// iTunes U episode
        const I_TUNES_U_EPISODE = 1 << 14;
    }
}

iap1_bitflags! {
    strings = LingoString;
    pub struct RetIndexedPlayingTrackInfoPacketInfoBits: u8 {
        /// one of multiple packets
        const ONE_OF_MULTIPLE_PACKETS = 1 << 0;
        /// last packet
        const LAST_PACKET = 1 << 1;
    }
}

iap1_record! {
    strings = LingoString;
    pub struct RetIndexedPlayingTrackInfoArtworkFormats {
        fields {
            /// Format ID
            // Virtual schema indices: 1
            required pub format_id: u16 => { bit: 0, key: "formatID", when: (truthy(&true)), predicate_value: false },
            /// Image Count
            // Virtual schema indices: 2
            required pub image_count: u16 => { bit: 1, key: "imageCount", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required format_id: u16 => { bit: 0, key: "formatID", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            2 => required image_count: u16 => { bit: 1, key: "imageCount", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x03, command = 0x0013,
        source = device,
        response = true,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct RetIndexedPlayingTrackInfo {
        fields {
            /// Info Type
            // Virtual schema indices: 1
            required pub info_type: RetIndexedPlayingTrackInfoInfoType => { bit: 0, key: "infoType", when: (truthy(&true)), predicate_value: true },
            /// Track Capabilities Bits
            // Virtual schema indices: 2
            optional pub track_caps: RetIndexedPlayingTrackInfoTrackCaps => { bit: 1, key: "trackCaps", when: (eq(&info_type, &0u64)), predicate_value: false },
            /// Total Length of Track
            // Virtual schema indices: 3
            optional pub track_total_length: u32 => { bit: 2, key: "trackTotalLength", when: (eq(&info_type, &0u64)), predicate_value: false },
            /// Chapter Count
            // Virtual schema indices: 4
            optional pub chapter_count: u16 => { bit: 3, key: "chapterCount", when: (eq(&info_type, &0u64)), predicate_value: false },
            /// Chapter Time
            // Virtual schema indices: 5
            optional pub chapter_time: u32 => { bit: 4, key: "chapterTime", when: (eq(&info_type, &1u64)), predicate_value: false },
            /// Chapter Name
            // Virtual schema indices: 6
            optional pub chapter_name: String => { bit: 5, key: "chapterName", when: (eq(&info_type, &1u64)), predicate_value: false },
            /// Artist Name
            // Virtual schema indices: 7
            optional pub artist_name: String => { bit: 6, key: "artistName", when: (eq(&info_type, &2u64)), predicate_value: false },
            /// Album Name
            // Virtual schema indices: 8
            optional pub album_name: String => { bit: 7, key: "albumName", when: (eq(&info_type, &3u64)), predicate_value: false },
            /// Genre Name
            // Virtual schema indices: 9
            optional pub genre_name: String => { bit: 8, key: "genreName", when: (eq(&info_type, &4u64)), predicate_value: false },
            /// Track Title
            // Virtual schema indices: 10
            optional pub track_title: String => { bit: 9, key: "trackTitle", when: (eq(&info_type, &5u64)), predicate_value: false },
            /// Composer Name
            // Virtual schema indices: 11
            optional pub composer_name: String => { bit: 10, key: "composerName", when: (eq(&info_type, &6u64)), predicate_value: false },
            /// Packet Information Bits
            // Virtual schema indices: 12
            optional pub packet_info_bits: RetIndexedPlayingTrackInfoPacketInfoBits => { bit: 11, key: "packetInfoBits", when: (eq(&info_type, &7u64)), predicate_value: false },
            /// Packet Index
            // Virtual schema indices: 13
            optional pub packet_index: u16 => { bit: 12, key: "packetIndex", when: (eq(&info_type, &7u64)), predicate_value: false },
            /// Lyrics Data
            // Virtual schema indices: 14
            optional pub track_lyrics: Vec<u8> => { bit: 13, key: "trackLyrics", when: (eq(&info_type, &7u64)), predicate_value: false },
            /// Artwork Formats
            // Virtual schema indices: 15
            optional pub artwork_formats: Vec<RetIndexedPlayingTrackInfoArtworkFormats> => { bit: 14, key: "artworkFormats", when: (eq(&info_type, &8u64)), predicate_value: false },
        }
        steps {
            1 => required info_type: RetIndexedPlayingTrackInfoInfoType => { bit: 0, key: "infoType", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: true, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            2 => optional track_caps: RetIndexedPlayingTrackInfoTrackCaps => { bit: 1, key: "trackCaps", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&info_type, &0u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(0u8)] } },
            3 => optional track_total_length: u32 => { bit: 2, key: "trackTotalLength", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&info_type, &0u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(0u8)] } },
            4 => optional chapter_count: u16 => { bit: 3, key: "chapterCount", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&info_type, &0u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(0u8)] } },
            5 => optional chapter_time: u32 => { bit: 4, key: "chapterTime", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&info_type, &1u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(1u8)] } },
            6 => optional chapter_name: String => { bit: 5, key: "chapterName", wire: [raw_string], project_wire: Iap1WireSpec::Raw, predicate_value: false, predicate: { expr: eq(&info_type, &1u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(1u8)] } },
            7 => optional artist_name: String => { bit: 6, key: "artistName", wire: [raw_string], project_wire: Iap1WireSpec::Raw, predicate_value: false, predicate: { expr: eq(&info_type, &2u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(2u8)] } },
            8 => optional album_name: String => { bit: 7, key: "albumName", wire: [raw_string], project_wire: Iap1WireSpec::Raw, predicate_value: false, predicate: { expr: eq(&info_type, &3u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(3u8)] } },
            9 => optional genre_name: String => { bit: 8, key: "genreName", wire: [raw_string], project_wire: Iap1WireSpec::Raw, predicate_value: false, predicate: { expr: eq(&info_type, &4u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(4u8)] } },
            10 => optional track_title: String => { bit: 9, key: "trackTitle", wire: [raw_string], project_wire: Iap1WireSpec::Raw, predicate_value: false, predicate: { expr: eq(&info_type, &5u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(5u8)] } },
            11 => optional composer_name: String => { bit: 10, key: "composerName", wire: [raw_string], project_wire: Iap1WireSpec::Raw, predicate_value: false, predicate: { expr: eq(&info_type, &6u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(6u8)] } },
            12 => optional packet_info_bits: RetIndexedPlayingTrackInfoPacketInfoBits => { bit: 11, key: "packetInfoBits", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&info_type, &7u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(7u8)] } },
            13 => optional packet_index: u16 => { bit: 12, key: "packetIndex", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&info_type, &7u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(7u8)] } },
            14 => optional track_lyrics: Vec<u8> => { bit: 13, key: "trackLyrics", wire: [raw_bytes], project_wire: Iap1WireSpec::Raw, predicate_value: false, predicate: { expr: eq(&info_type, &7u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(7u8)] } },
            15 => optional artwork_formats: Vec<RetIndexedPlayingTrackInfoArtworkFormats> => { bit: 14, key: "artworkFormats", wire: [counted_remaining], project_wire: Iap1WireSpec::CountedRemaining, predicate_value: false, predicate: { expr: eq(&info_type, &8u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(8u8)] } },
        }
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x03, command = 0x0014,
        source = accessory,
        response = false,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct GetNumPlayingTracks {
        fields {
        }
        steps {
        }
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x03, command = 0x0015,
        source = device,
        response = true,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct RetNumPlayingTracks {
        fields {
            /// Total Number of Queued Playing Tracks
            // Virtual schema indices: 1
            required pub num_playing_tracks: u32 => { bit: 0, key: "numPlayingTracks", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required num_playing_tracks: u32 => { bit: 0, key: "numPlayingTracks", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x03, command = 0x0016,
        source = accessory,
        response = false,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct GetArtworkFormats {
        fields {
        }
        steps {
        }
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum RetArtworkFormatsArtworkFormatsPixelFormat: u8 {
        /// monochrome, 2 bits per pixel
        Monochrome2BitsPerPixel = 1,
        /// RGB 565 color, little-endian, 16 bpp
        RGB565ColorLittleEndian16Bpp = 2,
        /// RGB 565 color, big-endian, 16 bpp
        RGB565ColorBigEndian16Bpp = 3,
    }
}

iap1_record! {
    strings = LingoString;
    pub struct RetArtworkFormatsArtworkFormats {
        fields {
            /// Format ID
            // Virtual schema indices: 1
            required pub format_id: u16 => { bit: 0, key: "formatID", when: (truthy(&true)), predicate_value: false },
            /// Display Pixel Format
            // Virtual schema indices: 2
            required pub pixel_format: RetArtworkFormatsArtworkFormatsPixelFormat => { bit: 1, key: "pixelFormat", when: (truthy(&true)), predicate_value: false },
            /// Image Width
            // Virtual schema indices: 3
            required pub image_width: u16 => { bit: 2, key: "imageWidth", when: (truthy(&true)), predicate_value: false },
            /// Image Height
            // Virtual schema indices: 4
            required pub image_height: u16 => { bit: 3, key: "imageHeight", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required format_id: u16 => { bit: 0, key: "formatID", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            2 => required pixel_format: RetArtworkFormatsArtworkFormatsPixelFormat => { bit: 1, key: "pixelFormat", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            3 => required image_width: u16 => { bit: 2, key: "imageWidth", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            4 => required image_height: u16 => { bit: 3, key: "imageHeight", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x03, command = 0x0017,
        source = device,
        response = true,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct RetArtworkFormats {
        fields {
            /// Artwork Formats
            // Virtual schema indices: 1
            required pub artwork_formats: Vec<RetArtworkFormatsArtworkFormats> => { bit: 0, key: "artworkFormats", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required artwork_formats: Vec<RetArtworkFormatsArtworkFormats> => { bit: 0, key: "artworkFormats", wire: [counted_remaining], project_wire: Iap1WireSpec::CountedRemaining, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x03, command = 0x0018,
        source = accessory,
        response = false,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct GetTrackArtworkData {
        fields {
            /// Track Index
            // Virtual schema indices: 1
            required pub track_index: u32 => { bit: 0, key: "trackIndex", when: (truthy(&true)), predicate_value: false },
            /// Format ID
            // Virtual schema indices: 2
            required pub format_id: u16 => { bit: 1, key: "formatID", when: (truthy(&true)), predicate_value: false },
            /// Time Offset
            // Virtual schema indices: 3
            required pub time_offset: u32 => { bit: 2, key: "timeOffset", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required track_index: u32 => { bit: 0, key: "trackIndex", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            2 => required format_id: u16 => { bit: 1, key: "formatID", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            3 => required time_offset: u32 => { bit: 2, key: "timeOffset", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum RetTrackArtworkDataPixelFormat: u8 {
        /// monochrome, 2 bits per pixel
        Monochrome2BitsPerPixel = 1,
        /// RGB 565 color, little-endian, 16 bpp
        RGB565ColorLittleEndian16Bpp = 2,
        /// RGB 565 color, big-endian, 16 bpp
        RGB565ColorBigEndian16Bpp = 3,
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x03, command = 0x0019,
        source = device,
        response = true,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct RetTrackArtworkData {
        fields {
            /// Descriptor Packet Index
            // Virtual schema indices: 1
            required pub descriptor_packet_index: u16 => { bit: 0, key: "descriptorPacketIndex", when: (truthy(&true)), predicate_value: true },
            /// Display Pixel Format
            // Virtual schema indices: 2
            optional pub pixel_format: RetTrackArtworkDataPixelFormat => { bit: 1, key: "pixelFormat", when: (eq(&descriptor_packet_index, &0u64)), predicate_value: false },
            /// Image Width
            // Virtual schema indices: 3
            optional pub image_width: u16 => { bit: 2, key: "imageWidth", when: (eq(&descriptor_packet_index, &0u64)), predicate_value: false },
            /// Image Height
            // Virtual schema indices: 4
            optional pub image_height: u16 => { bit: 3, key: "imageHeight", when: (eq(&descriptor_packet_index, &0u64)), predicate_value: false },
            /// Inset Rectangle, Top-Left Point, X Value
            // Virtual schema indices: 5
            optional pub x_value_of_top_left_point_of_inset_rectangle: u16 => { bit: 4, key: "xValueOfTopLeftPointOfInsetRectangle", when: (eq(&descriptor_packet_index, &0u64)), predicate_value: false },
            /// Inset Rectangle, Top-Left Point, Y Value
            // Virtual schema indices: 6
            optional pub y_value_of_top_left_point_of_inset_rectangle: u16 => { bit: 5, key: "yValueOfTopLeftPointOfInsetRectangle", when: (eq(&descriptor_packet_index, &0u64)), predicate_value: false },
            /// Inset Rectangle, Bottom-Right Point, X Value
            // Virtual schema indices: 7
            optional pub x_value_of_bottom_right_point_of_inset_rectangle: u16 => { bit: 6, key: "xValueOfBottomRightPointOfInsetRectangle", when: (eq(&descriptor_packet_index, &0u64)), predicate_value: false },
            /// Inset Rectangle, Bottom-Right Point, Y Value
            // Virtual schema indices: 8
            optional pub y_value_of_bottom_right_point_of_inset_rectangle: u16 => { bit: 7, key: "yValueOfBottomRightPointOfInsetRectangle", when: (eq(&descriptor_packet_index, &0u64)), predicate_value: false },
            /// Row Size
            // Virtual schema indices: 9
            optional pub row_size: u32 => { bit: 8, key: "rowSize", when: (eq(&descriptor_packet_index, &0u64)), predicate_value: false },
            /// Image Pixel Data
            // Virtual schema indices: 10
            required pub image_pixel_data: Vec<u8> => { bit: 9, key: "imagePixelData", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required descriptor_packet_index: u16 => { bit: 0, key: "descriptorPacketIndex", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: true, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            2 => optional pixel_format: RetTrackArtworkDataPixelFormat => { bit: 1, key: "pixelFormat", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&descriptor_packet_index, &0u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(0u8)] } },
            3 => optional image_width: u16 => { bit: 2, key: "imageWidth", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&descriptor_packet_index, &0u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(0u8)] } },
            4 => optional image_height: u16 => { bit: 3, key: "imageHeight", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&descriptor_packet_index, &0u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(0u8)] } },
            5 => optional x_value_of_top_left_point_of_inset_rectangle: u16 => { bit: 4, key: "xValueOfTopLeftPointOfInsetRectangle", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&descriptor_packet_index, &0u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(0u8)] } },
            6 => optional y_value_of_top_left_point_of_inset_rectangle: u16 => { bit: 5, key: "yValueOfTopLeftPointOfInsetRectangle", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&descriptor_packet_index, &0u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(0u8)] } },
            7 => optional x_value_of_bottom_right_point_of_inset_rectangle: u16 => { bit: 6, key: "xValueOfBottomRightPointOfInsetRectangle", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&descriptor_packet_index, &0u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(0u8)] } },
            8 => optional y_value_of_bottom_right_point_of_inset_rectangle: u16 => { bit: 7, key: "yValueOfBottomRightPointOfInsetRectangle", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&descriptor_packet_index, &0u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(0u8)] } },
            9 => optional row_size: u32 => { bit: 8, key: "rowSize", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&descriptor_packet_index, &0u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(0u8)] } },
            10 => required image_pixel_data: Vec<u8> => { bit: 9, key: "imagePixelData", wire: [raw_bytes], project_wire: Iap1WireSpec::Raw, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x03, command = 0x001a,
        source = accessory,
        response = false,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct GetPowerBatteryState {
        fields {
        }
        steps {
        }
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum RetPowerBatteryStatePowerState: u8 {
        /// internal battery power, low power (< 30%)
        InternalBatteryPowerLowPower30 = 0,
        /// internal battery power
        InternalBatteryPower = 1,
        /// external power, battery pack, no charging
        ExternalPowerBatteryPackNoCharging = 2,
        /// external power, battery charging
        ExternalPowerBatteryCharging = 3,
        /// external power, battery charged
        ExternalPowerBatteryCharged = 4,
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x03, command = 0x001b,
        source = device,
        response = true,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct RetPowerBatteryState {
        fields {
            /// Power State
            // Virtual schema indices: 1
            required pub power_state: RetPowerBatteryStatePowerState => { bit: 0, key: "powerState", when: (truthy(&true)), predicate_value: false },
            /// Battery Level
            // Virtual schema indices: 2
            required pub battery_level: u8 => { bit: 1, key: "batteryLevel", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required power_state: RetPowerBatteryStatePowerState => { bit: 0, key: "powerState", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            2 => required battery_level: u8 => { bit: 1, key: "batteryLevel", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x03, command = 0x001c,
        source = accessory,
        response = false,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct GetSoundCheckState {
        fields {
        }
        steps {
        }
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x03, command = 0x001d,
        source = device,
        response = true,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct RetSoundCheckState {
        fields {
            /// Sound Check State
            // Virtual schema indices: 1
            required pub sound_check_state: bool => { bit: 0, key: "soundCheckState", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required sound_check_state: bool => { bit: 0, key: "soundCheckState", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x03, command = 0x001e,
        source = accessory,
        response = false,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct SetSoundCheckState {
        fields {
            /// Sound Check State
            // Virtual schema indices: 1
            required pub sound_check_state: bool => { bit: 0, key: "soundCheckState", when: (truthy(&true)), predicate_value: false },
            /// Restore on Exit
            // Virtual schema indices: 2
            required pub restore_on_exit: bool => { bit: 1, key: "restoreOnExit", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required sound_check_state: bool => { bit: 0, key: "soundCheckState", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            2 => required restore_on_exit: bool => { bit: 1, key: "restoreOnExit", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x03, command = 0x001f,
        source = accessory,
        response = false,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct GetTrackArtworkTimes {
        fields {
            /// Track Index
            // Virtual schema indices: 1
            required pub track_index: u32 => { bit: 0, key: "trackIndex", when: (truthy(&true)), predicate_value: false },
            /// Format ID
            // Virtual schema indices: 2
            required pub format_id: u16 => { bit: 1, key: "formatID", when: (truthy(&true)), predicate_value: false },
            /// Artwork Index
            // Virtual schema indices: 3
            required pub artwork_index: u16 => { bit: 2, key: "artworkIndex", when: (truthy(&true)), predicate_value: false },
            /// Artwork Count
            // Virtual schema indices: 4
            required pub artwork_count: u16 => { bit: 3, key: "artworkCount", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required track_index: u32 => { bit: 0, key: "trackIndex", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            2 => required format_id: u16 => { bit: 1, key: "formatID", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            3 => required artwork_index: u16 => { bit: 2, key: "artworkIndex", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            4 => required artwork_count: u16 => { bit: 3, key: "artworkCount", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1_record! {
    strings = LingoString;
    pub struct RetTrackArtworkTimesArtworkTimes {
        fields {
            /// Time Offset
            // Virtual schema indices: 1
            required pub time_offset: u32 => { bit: 0, key: "timeOffset", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required time_offset: u32 => { bit: 0, key: "timeOffset", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x03, command = 0x0020,
        source = device,
        response = true,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct RetTrackArtworkTimes {
        fields {
            /// Artwork Times
            // Virtual schema indices: 1
            required pub artwork_times: Vec<RetTrackArtworkTimesArtworkTimes> => { bit: 0, key: "artworkTimes", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required artwork_times: Vec<RetTrackArtworkTimesArtworkTimes> => { bit: 0, key: "artworkTimes", wire: [counted_remaining], project_wire: Iap1WireSpec::CountedRemaining, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x03, command = 0x0021,
        source = accessory,
        response = false,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct CreateGeniusPlaylist {
        fields {
            /// Track Index
            // Virtual schema indices: 1
            required pub track_index: u32 => { bit: 0, key: "trackIndex", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required track_index: u32 => { bit: 0, key: "trackIndex", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x03, command = 0x0022,
        source = accessory,
        response = false,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct IsGeniusAvailableForTrack {
        fields {
            /// Track Index
            // Virtual schema indices: 1
            required pub track_index: u32 => { bit: 0, key: "trackIndex", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required track_index: u32 => { bit: 0, key: "trackIndex", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1_registry! {
    0x03;
    IPodAck => 0x0000,
    GetCurrentEQProfileIndex => 0x0001,
    RetCurrentEQProfileIndex => 0x0002,
    SetCurrentEQProfileIndex => 0x0003,
    GetNumEQProfiles => 0x0004,
    RetNumEQProfiles => 0x0005,
    GetIndexedEQProfileName => 0x0006,
    RetIndexedEQProfileName => 0x0007,
    SetRemoteEventNotification => 0x0008,
    RemoteEventNotification => 0x0009,
    GetRemoteEventStatus => 0x000a,
    RetRemoteEventStatus => 0x000b,
    GetiPodStateInfo => 0x000c,
    RetiPodStateInfo => 0x000d,
    SetiPodStateInfo => 0x000e,
    GetPlayStatus => 0x000f,
    RetPlayStatus => 0x0010,
    SetCurrentPlayingTrack => 0x0011,
    GetIndexedPlayingTrackInfo => 0x0012,
    RetIndexedPlayingTrackInfo => 0x0013,
    GetNumPlayingTracks => 0x0014,
    RetNumPlayingTracks => 0x0015,
    GetArtworkFormats => 0x0016,
    RetArtworkFormats => 0x0017,
    GetTrackArtworkData => 0x0018,
    RetTrackArtworkData => 0x0019,
    GetPowerBatteryState => 0x001a,
    RetPowerBatteryState => 0x001b,
    GetSoundCheckState => 0x001c,
    RetSoundCheckState => 0x001d,
    SetSoundCheckState => 0x001e,
    GetTrackArtworkTimes => 0x001f,
    RetTrackArtworkTimes => 0x0020,
    CreateGeniusPlaylist => 0x0021,
    IsGeniusAvailableForTrack => 0x0022,
}
