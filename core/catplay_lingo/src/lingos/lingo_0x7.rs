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

// Lingo 0x07: RF Tuner
iap1_enum! {
    strings = LingoString;
    pub enum AccessoryAckCommandResult: u8 {
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
        context = ctx, strings = LingoString, lingo = 0x07, command = 0x0000,
        source = accessory,
        response = true,
        ack = true,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct AccessoryAck {
        fields {
            /// Command Result
            // Virtual schema indices: 1
            required pub command_result: AccessoryAckCommandResult => { bit: 0, key: "commandResult", when: (truthy(&true)), predicate_value: true },
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
            1 => required command_result: AccessoryAckCommandResult => { bit: 0, key: "commandResult", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: true, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            2 => required acked_command_id: u8 => { bit: 1, key: "ackedCommandID", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            3 => optional maximum_pending_wait: u32 => { bit: 2, key: "maximumPendingWait", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&command_result, &6u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(6u8)] } },
            4 => optional session_id: u16 => { bit: 3, key: "sessionID", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&command_result, &23u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(23u8)] } },
            5 => optional num_bytes_dropped: u32 => { bit: 4, key: "numBytesDropped", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&command_result, &23u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(23u8)] } },
        }
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x07, command = 0x0001,
        source = device,
        response = false,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct GetTunerCaps {
        fields {
        }
        steps {
        }
    }
}

iap1_bitflags! {
    strings = LingoString;
    pub struct RetTunerCapsTunerCapsFlags: u32 {
        /// AM band, Worldwide (520-1710 KHz)
        const AM_BAND_WORLDWIDE_520_1710_K_HZ = 1 << 0;
        /// FM band, Europe/US (87.5-108.0 MHz)
        const FM_BAND_EUROPE_US_87_5_108_0_M_HZ = 1 << 1;
        /// FM band, Japan (76.0-90.0 MHz)
        const FM_BAND_JAPAN_76_0_90_0_M_HZ = 1 << 2;
        /// FM band, Wide (76.0-108.0 MHz)
        const FM_BAND_WIDE_76_0_108_0_M_HZ = 1 << 3;
        /// HD Radio
        const HD_RADIO = 1 << 4;
        /// tuner power on/off control
        const TUNER_POWER_ON_OFF_CONTROL = 1 << 8;
        /// status change notification
        const STATUS_CHANGE_NOTIFICATION = 1 << 9;
        /// tuner seek up/down
        const TUNER_SEEK_UP_DOWN = 1 << 18;
        /// tuner seek RSSI threshold
        const TUNER_SEEK_RSSI_THRESHOLD = 1 << 19;
        /// force monophonic mode
        const FORCE_MONOPHONIC_MODE = 1 << 20;
        /// stereo blend
        const STEREO_BLEND = 1 << 21;
        /// FM tuner deemphasis select
        const FM_TUNER_DEEMPHASIS_SELECT = 1 << 22;
        /// AM tuner resolution 9KHz
        const AM_TUNER_RESOLUTION_9K_HZ = 1 << 23;
        /// Radio Data System (RDS/RBDS) data
        const RADIO_DATA_SYSTEM_RDS_RBDS_DATA = 1 << 24;
        /// tuner channel RSSI indication
        const TUNER_CHANNEL_RSSI_INDICATION = 1 << 25;
        /// stereo source indicator
        const STEREO_SOURCE_INDICATOR = 1 << 26;
        /// RDS/RDBS Raw mode
        const RDS_RDBS_RAW_MODE = 1 << 27;
    }
}

iap1_bitfield! {
    pub struct RetTunerCapsTunerCaps: u32 {
        #[flags(mask = 0xffc031f)]
        pub flags: RetTunerCapsTunerCapsFlags,
        #[group(mask = 0x30000, shift = 16)]
        /// minimum FM resolution
        pub minimum_fm_resolution_id: RetTunerCapsTunerCapsMinimumFMResolutionID {
            /// 200 kHz
            Value200KHz = 0,
            /// 100 kHz
            Value100KHz = 1,
            /// 50 kHz
            Value50KHz = 2,
        },
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x07, command = 0x0002,
        source = accessory,
        response = true,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct RetTunerCaps {
        fields {
            /// Tuner Caps
            // Virtual schema indices: 1
            required pub tuner_caps: RetTunerCapsTunerCaps => { bit: 0, key: "tunerCaps", when: (truthy(&true)), predicate_value: false },
            /// Reserved
            // Virtual schema indices: 2
            required pub reserved1: u8 => { bit: 1, key: "reserved1", when: (truthy(&true)), predicate_value: false },
            /// Reserved
            // Virtual schema indices: 3
            required pub reserved2: u8 => { bit: 2, key: "reserved2", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required tuner_caps: RetTunerCapsTunerCaps => { bit: 0, key: "tunerCaps", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            2 => required reserved1: u8 => { bit: 1, key: "reserved1", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            3 => required reserved2: u8 => { bit: 2, key: "reserved2", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x07, command = 0x0003,
        source = device,
        response = false,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct GetTunerCtrl {
        fields {
        }
        steps {
        }
    }
}

iap1_bitflags! {
    strings = LingoString;
    pub struct RetTunerCtrlTunerControlState: u8 {
        /// RF tuner device power on
        const RF_TUNER_DEVICE_POWER_ON = 1 << 0;
        /// status change notification enabled
        const STATUS_CHANGE_NOTIFICATION_ENABLED = 1 << 1;
        /// RDS/RBDS Raw mode enabled
        const RDS_RBDS_RAW_MODE_ENABLED = 1 << 3;
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x07, command = 0x0004,
        source = accessory,
        response = true,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct RetTunerCtrl {
        fields {
            /// Tuner Control State
            // Virtual schema indices: 1
            required pub tuner_control_state: RetTunerCtrlTunerControlState => { bit: 0, key: "tunerControlState", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required tuner_control_state: RetTunerCtrlTunerControlState => { bit: 0, key: "tunerControlState", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1_bitflags! {
    strings = LingoString;
    pub struct SetTunerCtrlTunerControlState: u8 {
        /// RF tuner device power on
        const RF_TUNER_DEVICE_POWER_ON = 1 << 0;
        /// status change notification enabled
        const STATUS_CHANGE_NOTIFICATION_ENABLED = 1 << 1;
        /// RDS/RBDS Raw mode enabled
        const RDS_RBDS_RAW_MODE_ENABLED = 1 << 3;
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x07, command = 0x0005,
        source = device,
        response = false,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct SetTunerCtrl {
        fields {
            /// Tuner Control State
            // Virtual schema indices: 1
            required pub tuner_control_state: SetTunerCtrlTunerControlState => { bit: 0, key: "tunerControlState", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required tuner_control_state: SetTunerCtrlTunerControlState => { bit: 0, key: "tunerControlState", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x07, command = 0x0006,
        source = device,
        response = false,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct GetTunerBand {
        fields {
        }
        steps {
        }
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum RetTunerBandTunerBandState: u8 {
        /// AM band worldwide (520-1710 kHz)
        AMBandWorldwide5201710KHz = 0,
        /// Europe/US FM band (87.5-108.0 MHz)
        EuropeUSFMBand8751080MHz = 1,
        /// Japan FM band (76.0-90.0 MHz)
        JapanFMBand760900MHz = 2,
        /// FM wide band (76.0-108.0 MHz)
        FMWideBand7601080MHz = 3,
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x07, command = 0x0007,
        source = accessory,
        response = true,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct RetTunerBand {
        fields {
            /// Tuner Band
            // Virtual schema indices: 1
            required pub tuner_band_state: RetTunerBandTunerBandState => { bit: 0, key: "tunerBandState", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required tuner_band_state: RetTunerBandTunerBandState => { bit: 0, key: "tunerBandState", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum SetTunerBandTunerBandState: u8 {
        /// AM band worldwide (520-1710 kHz)
        AMBandWorldwide5201710KHz = 0,
        /// Europe/US FM band (87.5-108.0 MHz)
        EuropeUSFMBand8751080MHz = 1,
        /// Japan FM band (76.0-90.0 MHz)
        JapanFMBand760900MHz = 2,
        /// FM wide band (76.0-108.0 MHz)
        FMWideBand7601080MHz = 3,
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x07, command = 0x0008,
        source = device,
        response = false,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct SetTunerBand {
        fields {
            /// Tuner Band
            // Virtual schema indices: 1
            required pub tuner_band_state: SetTunerBandTunerBandState => { bit: 0, key: "tunerBandState", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required tuner_band_state: SetTunerBandTunerBandState => { bit: 0, key: "tunerBandState", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x07, command = 0x0009,
        source = device,
        response = false,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct GetTunerFreq {
        fields {
        }
        steps {
        }
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x07, command = 0x000a,
        source = accessory,
        response = true,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct RetTunerFreq {
        fields {
            /// Tuner Frequency
            // Virtual schema indices: 1
            required pub tuner_frequency: u32 => { bit: 0, key: "tunerFrequency", when: (truthy(&true)), predicate_value: false },
            /// Received Signal Strength
            // Virtual schema indices: 2
            required pub rssi_level: u8 => { bit: 1, key: "rssiLevel", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required tuner_frequency: u32 => { bit: 0, key: "tunerFrequency", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            2 => required rssi_level: u8 => { bit: 1, key: "rssiLevel", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x07, command = 0x000b,
        source = device,
        response = false,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct SetTunerFreq {
        fields {
            /// Tuner Frequency
            // Virtual schema indices: 1
            required pub tuner_frequency: u32 => { bit: 0, key: "tunerFrequency", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required tuner_frequency: u32 => { bit: 0, key: "tunerFrequency", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x07, command = 0x000c,
        source = device,
        response = false,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct GetTunerMode {
        fields {
        }
        steps {
        }
    }
}

iap1_bitflags! {
    strings = LingoString;
    pub struct RetTunerModeTunerModeStatusFlags: u8 {
        /// tuner is currently seeking up or down
        const TUNER_IS_CURRENTLY_SEEKING_UP_OR_DOWN = 1 << 2;
        /// tuner is currently seeking with an RSSI minimum threshold enabled
        const TUNER_IS_CURRENTLY_SEEKING_WITH_AN_RSSI_MINIMUM_THRESHOLD_ENABLED = 1 << 3;
        /// monophonic mode forced
        const MONOPHONIC_MODE_FORCED = 1 << 4;
        /// stereo blend is enabled
        const STEREO_BLEND_IS_ENABLED = 1 << 5;
        /// FM tuner emphasis is 50 µs
        const FM_TUNER_EMPHASIS_IS_50_µS = 1 << 6;
        /// 9 kHz AM tuner resolution
        const VALUE_9_K_HZ_AM_TUNER_RESOLUTION = 1 << 7;
    }
}

iap1_bitfield! {
    pub struct RetTunerModeTunerModeStatus: u8 {
        #[flags(mask = 0xfc)]
        pub flags: RetTunerModeTunerModeStatusFlags,
        #[group(mask = 0x3, shift = 0)]
        /// FM tuner resolution
        pub fm_tuner_resolution: RetTunerModeTunerModeStatusFmTunerResolution {
            /// 200 kHz
            Value200KHz = 0,
            /// 100 kHz
            Value100KHz = 1,
            /// 50 kHz
            Value50KHz = 2,
        },
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x07, command = 0x000d,
        source = accessory,
        response = true,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct RetTunerMode {
        fields {
            /// Tuner Mode Status
            // Virtual schema indices: 1
            required pub tuner_mode_status: RetTunerModeTunerModeStatus => { bit: 0, key: "tunerModeStatus", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required tuner_mode_status: RetTunerModeTunerModeStatus => { bit: 0, key: "tunerModeStatus", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1_bitflags! {
    strings = LingoString;
    pub struct SetTunerModeTunerModeStatusFlags: u8 {
        /// monophonic mode forced
        const MONOPHONIC_MODE_FORCED = 1 << 4;
        /// stereo blend is enabled
        const STEREO_BLEND_IS_ENABLED = 1 << 5;
        /// FM tuner emphasis is 50 µs
        const FM_TUNER_EMPHASIS_IS_50_µS = 1 << 6;
        /// 9 kHz AM tuner resolution
        const VALUE_9_K_HZ_AM_TUNER_RESOLUTION = 1 << 7;
    }
}

iap1_bitfield! {
    pub struct SetTunerModeTunerModeStatus: u8 {
        #[flags(mask = 0xf0)]
        pub flags: SetTunerModeTunerModeStatusFlags,
        #[group(mask = 0x3, shift = 0)]
        /// FM tuner resolution
        pub fm_tuner_resolution: SetTunerModeTunerModeStatusFmTunerResolution {
            /// 200 kHz
            Value200KHz = 0,
            /// 100 kHz
            Value100KHz = 1,
            /// 50 kHz
            Value50KHz = 2,
        },
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x07, command = 0x000e,
        source = device,
        response = false,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct SetTunerMode {
        fields {
            /// Tuner Mode Status
            // Virtual schema indices: 1
            required pub tuner_mode_status: SetTunerModeTunerModeStatus => { bit: 0, key: "tunerModeStatus", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required tuner_mode_status: SetTunerModeTunerModeStatus => { bit: 0, key: "tunerModeStatus", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x07, command = 0x000f,
        source = device,
        response = false,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct GetTunerSeekRssi {
        fields {
        }
        steps {
        }
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x07, command = 0x0010,
        source = accessory,
        response = true,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct RetTunerSeekRssi {
        fields {
            /// RSSI Seek Threshold
            // Virtual schema indices: 1
            required pub rssi_seek_threshold: u8 => { bit: 0, key: "rssiSeekThreshold", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required rssi_seek_threshold: u8 => { bit: 0, key: "rssiSeekThreshold", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x07, command = 0x0011,
        source = device,
        response = false,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct SetTunerSeekRssi {
        fields {
            /// RSSI Seek Threshold
            // Virtual schema indices: 1
            required pub rssi_seek_threshold: u8 => { bit: 0, key: "rssiSeekThreshold", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required rssi_seek_threshold: u8 => { bit: 0, key: "rssiSeekThreshold", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum TunerSeekStartTunerSeekingOperation: u8 {
        /// no seek operation (cancel if active
        NoSeekOperationCancelIfActive = 0,
        /// seek up from beginning of band
        SeekUpFromBeginningOfBand = 1,
        /// seek down from end of band
        SeekDownFromEndOfBand = 2,
        /// seek up from current frequency
        SeekUpFromCurrentFrequency = 3,
        /// seek down from current frequency
        SeekDownFromCurrentFrequency = 4,
        /// seek up from beginning of band (use RSSI threshold)
        SeekUpFromBeginningOfBandUseRSSIThreshold = 5,
        /// seek down from end of band (use RSSI threshold)
        SeekDownFromEndOfBandUseRSSIThreshold = 6,
        /// seek up from current frequency (use RSSI threshold)
        SeekUpFromCurrentFrequencyUseRSSIThreshold = 7,
        /// seek down from current frequency (use RSSI threshold)
        SeekDownFromCurrentFrequencyUseRSSIThreshold = 8,
        /// seek up from beginning of band for an HD signal
        SeekUpFromBeginningOfBandForAnHDSignal = 9,
        /// seek down from end of band for an HD signal
        SeekDownFromEndOfBandForAnHDSignal = 10,
        /// seek up from current frequency for an HD signal
        SeekUpFromCurrentFrequencyForAnHDSignal = 11,
        /// seek down from current frequency for an HD signal
        SeekDownFromCurrentFrequencyForAnHDSignal = 12,
        /// seek up from beginning of band for an HD signal (use RSSI threshold)
        SeekUpFromBeginningOfBandForAnHDSignalUseRSSIThreshold = 13,
        /// seek down from end of band for an HD signal (use RSSI threshold)
        SeekDownFromEndOfBandForAnHDSignalUseRSSIThreshold = 14,
        /// seek up from current frequency for an HD signal (use RSSI threshold)
        SeekUpFromCurrentFrequencyForAnHDSignalUseRSSIThreshold = 15,
        /// seek down from current frequency for an HD signal (use RSSI threshold)
        SeekDownFromCurrentFrequencyForAnHDSignalUseRSSIThreshold = 16,
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x07, command = 0x0012,
        source = device,
        response = false,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct TunerSeekStart {
        fields {
            /// Tuner Seeking Operation
            // Virtual schema indices: 1
            required pub tuner_seeking_operation: TunerSeekStartTunerSeekingOperation => { bit: 0, key: "tunerSeekingOperation", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required tuner_seeking_operation: TunerSeekStartTunerSeekingOperation => { bit: 0, key: "tunerSeekingOperation", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x07, command = 0x0013,
        source = accessory,
        response = true,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct TunerSeekDone {
        fields {
            /// Tuner Frequency
            // Virtual schema indices: 1
            required pub tuner_frequency: u32 => { bit: 0, key: "tunerFrequency", when: (truthy(&true)), predicate_value: false },
            /// Received Signal Strength
            // Virtual schema indices: 2
            required pub rssi_level: u8 => { bit: 1, key: "rssiLevel", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required tuner_frequency: u32 => { bit: 0, key: "tunerFrequency", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            2 => required rssi_level: u8 => { bit: 1, key: "rssiLevel", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x07, command = 0x0014,
        source = device,
        response = false,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct GetTunerStatus {
        fields {
        }
        steps {
        }
    }
}

iap1_bitflags! {
    strings = LingoString;
    pub struct RetTunerStatusTunerStatus: u8 {
        /// RDS/RBDS data is received, ready to read
        const RDS_RBDS_DATA_IS_RECEIVED_READY_TO_READ = 1 << 0;
        /// tuner channel RSSI level has changed
        const TUNER_CHANNEL_RSSI_LEVEL_HAS_CHANGED = 1 << 1;
        /// stereo signal source
        const STEREO_SIGNAL_SOURCE = 1 << 2;
        /// HD signal present
        const HD_SIGNAL_PRESENT = 1 << 3;
        /// HD digital audio present
        const HD_DIGITAL_AUDIO_PRESENT = 1 << 4;
        /// HD data ready to read
        const HD_DATA_READY_TO_READ = 1 << 5;
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x07, command = 0x0015,
        source = accessory,
        response = true,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct RetTunerStatus {
        fields {
            /// Tuner Status
            // Virtual schema indices: 1
            required pub tuner_status: RetTunerStatusTunerStatus => { bit: 0, key: "tunerStatus", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required tuner_status: RetTunerStatusTunerStatus => { bit: 0, key: "tunerStatus", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x07, command = 0x0016,
        source = device,
        response = false,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct GetStatusNotifyMask {
        fields {
        }
        steps {
        }
    }
}

iap1_bitflags! {
    strings = LingoString;
    pub struct RetStatusNotifyMaskStatusNotificationMask: u8 {
        /// RDS/RBDS data-ready change notify enabled
        const RDS_RBDS_DATA_READY_CHANGE_NOTIFY_ENABLED = 1 << 0;
        /// tuner channel RSSI-level change notify enabled
        const TUNER_CHANNEL_RSSI_LEVEL_CHANGE_NOTIFY_ENABLED = 1 << 1;
        /// stereo indicator state change notify enabled
        const STEREO_INDICATOR_STATE_CHANGE_NOTIFY_ENABLED = 1 << 2;
        /// HD signal present notification enabled
        const HD_SIGNAL_PRESENT_NOTIFICATION_ENABLED = 1 << 3;
        /// HD digital audio present notification enabled
        const HD_DIGITAL_AUDIO_PRESENT_NOTIFICATION_ENABLED = 1 << 4;
        /// HD data ready notification enabled
        const HD_DATA_READY_NOTIFICATION_ENABLED = 1 << 5;
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x07, command = 0x0017,
        source = accessory,
        response = true,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct RetStatusNotifyMask {
        fields {
            /// Status Notification Mask
            // Virtual schema indices: 1
            required pub status_notification_mask: RetStatusNotifyMaskStatusNotificationMask => { bit: 0, key: "statusNotificationMask", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required status_notification_mask: RetStatusNotifyMaskStatusNotificationMask => { bit: 0, key: "statusNotificationMask", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1_bitflags! {
    strings = LingoString;
    pub struct SetStatusNotifyMaskStatusNotificationMask: u8 {
        /// RDS/RBDS data-ready change notify enabled
        const RDS_RBDS_DATA_READY_CHANGE_NOTIFY_ENABLED = 1 << 0;
        /// tuner channel RSSI-level change notify enabled
        const TUNER_CHANNEL_RSSI_LEVEL_CHANGE_NOTIFY_ENABLED = 1 << 1;
        /// stereo indicator state change notify enabled
        const STEREO_INDICATOR_STATE_CHANGE_NOTIFY_ENABLED = 1 << 2;
        /// HD signal present notification enabled
        const HD_SIGNAL_PRESENT_NOTIFICATION_ENABLED = 1 << 3;
        /// HD digital audio present notification enabled
        const HD_DIGITAL_AUDIO_PRESENT_NOTIFICATION_ENABLED = 1 << 4;
        /// HD data ready notification enabled
        const HD_DATA_READY_NOTIFICATION_ENABLED = 1 << 5;
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x07, command = 0x0018,
        source = device,
        response = false,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct SetStatusNotifyMask {
        fields {
            /// Status Notification Mask
            // Virtual schema indices: 1
            required pub status_notification_mask: SetStatusNotifyMaskStatusNotificationMask => { bit: 0, key: "statusNotificationMask", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required status_notification_mask: SetStatusNotifyMaskStatusNotificationMask => { bit: 0, key: "statusNotificationMask", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1_bitflags! {
    strings = LingoString;
    pub struct StatusChangeNotifyStatusChangeBits: u8 {
        /// RDS/RBDS data-ready change
        const RDS_RBDS_DATA_READY_CHANGE = 1 << 0;
        /// tuner channel RSSI-level change
        const TUNER_CHANNEL_RSSI_LEVEL_CHANGE = 1 << 1;
        /// stereo indicator state change
        const STEREO_INDICATOR_STATE_CHANGE = 1 << 2;
        /// HD signal present
        const HD_SIGNAL_PRESENT = 1 << 3;
        /// HD digital audio present
        const HD_DIGITAL_AUDIO_PRESENT = 1 << 4;
        /// HD data ready to read
        const HD_DATA_READY_TO_READ = 1 << 5;
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x07, command = 0x0019,
        source = accessory,
        response = false,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct StatusChangeNotify {
        fields {
            /// Status Change Bits
            // Virtual schema indices: 1
            required pub status_change_bits: StatusChangeNotifyStatusChangeBits => { bit: 0, key: "statusChangeBits", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required status_change_bits: StatusChangeNotifyStatusChangeBits => { bit: 0, key: "statusChangeBits", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x07, command = 0x001a,
        source = device,
        response = false,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct GetRdsReadyStatus {
        fields {
        }
        steps {
        }
    }
}

iap1_bitflags! {
    strings = LingoString;
    pub struct RetRdsReadyStatusRdsDataReadyStatus: u32 {
        /// RadioText (RT) data ready
        const RADIO_TEXT_RT_DATA_READY = 1 << 4;
        /// RDS/RBDS group data ready
        const RDS_RBDS_GROUP_DATA_READY = 1 << 5;
        /// Program Service Name (PSN) data ready
        const PROGRAM_SERVICE_NAME_PSN_DATA_READY = 1 << 30;
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x07, command = 0x001b,
        source = accessory,
        response = true,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct RetRdsReadyStatus {
        fields {
            /// RDS/RBDS Data Ready Status
            // Virtual schema indices: 1
            required pub rds_data_ready_status: RetRdsReadyStatusRdsDataReadyStatus => { bit: 0, key: "rdsDataReadyStatus", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required rds_data_ready_status: RetRdsReadyStatusRdsDataReadyStatus => { bit: 0, key: "rdsDataReadyStatus", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum GetRdsDataRdsDataType: u8 {
        /// RadioText (RT)
        RadioTextRT = 4,
        /// RDS/RBDS group data
        RDSRBDSGroupData = 5,
        /// Program Service Name (PSN)
        ProgramServiceNamePSN = 30,
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x07, command = 0x001c,
        source = device,
        response = false,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct GetRdsData {
        fields {
            /// RDS/RBDS Data Type
            // Virtual schema indices: 1
            required pub rds_data_type: GetRdsDataRdsDataType => { bit: 0, key: "rdsDataType", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required rds_data_type: GetRdsDataRdsDataType => { bit: 0, key: "rdsDataType", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum RetRdsDataRdsDataType: u8 {
        /// RadioText (RT)
        RadioTextRT = 4,
        /// RDS/RBDS group data
        RDSRBDSGroupData = 5,
        /// Program Service Name (PSN)
        ProgramServiceNamePSN = 30,
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum RetRdsDataCharSet: u8 {
        /// Latin-based languages
        LatinBasedLanguages = 0,
        /// Cyrillic- and Greek-based languages
        CyrillicAndGreekBasedLanguages = 1,
        /// Arabic- and Hebrew-based languages
        ArabicAndHebrewBasedLanguages = 2,
    }
}

iap1_bitfield! {
    pub struct RetRdsDataBlockErrors: u8 {
        #[group(mask = 0x3, shift = 0)]
        /// Block A Error Bits
        pub block_a_error_bits: RetRdsDataBlockErrorsBlockAErrorBits {
            /// no errors
            NoErrors = 0,
            /// 1-2 errors; slight chance of uncorrected errors
            Value12ErrorsSlightChanceOfUncorrectedErrors = 1,
            /// 3-5 errors; may have uncorrected errors
            Value35ErrorsMayHaveUncorrectedErrors = 2,
            /// 6 or more errors; uncorrectable block
            Value6OrMoreErrorsUncorrectableBlock = 3,
        },
        #[group(mask = 0xc, shift = 2)]
        /// Block B Error Bits
        pub block_b_error_bits: RetRdsDataBlockErrorsBlockBErrorBits {
            /// no errors
            NoErrors = 0,
            /// 1-2 errors; slight chance of uncorrected errors
            Value12ErrorsSlightChanceOfUncorrectedErrors = 1,
            /// 3-5 errors; may have uncorrected errors
            Value35ErrorsMayHaveUncorrectedErrors = 2,
            /// 6 or more errors; uncorrectable block
            Value6OrMoreErrorsUncorrectableBlock = 3,
        },
        #[group(mask = 0x30, shift = 4)]
        /// Block C Error Bits
        pub block_c_error_bits: RetRdsDataBlockErrorsBlockCErrorBits {
            /// no errors
            NoErrors = 0,
            /// 1-2 errors; slight chance of uncorrected errors
            Value12ErrorsSlightChanceOfUncorrectedErrors = 1,
            /// 3-5 errors; may have uncorrected errors
            Value35ErrorsMayHaveUncorrectedErrors = 2,
            /// 6 or more errors; uncorrectable block
            Value6OrMoreErrorsUncorrectableBlock = 3,
        },
        #[group(mask = 0xc0, shift = 6)]
        /// Block D Error Bits
        pub block_d_error_bits: RetRdsDataBlockErrorsBlockDErrorBits {
            /// no errors
            NoErrors = 0,
            /// 1-2 errors; slight chance of uncorrected errors
            Value12ErrorsSlightChanceOfUncorrectedErrors = 1,
            /// 3-5 errors; may have uncorrected errors
            Value35ErrorsMayHaveUncorrectedErrors = 2,
            /// 6 or more errors; uncorrectable block
            Value6OrMoreErrorsUncorrectableBlock = 3,
        },
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x07, command = 0x001d,
        source = accessory,
        response = true,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct RetRdsData {
        fields {
            /// RDS/RBDS Data Type
            // Virtual schema indices: 1
            required pub rds_data_type: RetRdsDataRdsDataType => { bit: 0, key: "rdsDataType", when: (truthy(&true)), predicate_value: true },
            /// Character Set
            // Virtual schema indices: 2, 4
            optional pub char_set: RetRdsDataCharSet => { bit: 1, key: "charSet", when: (eq(&rds_data_type, &4u64)) || (eq(&rds_data_type, &30u64)), predicate_value: false },
            /// Radio Text
            // Virtual schema indices: 3
            optional pub radio_text: Vec<u8> => { bit: 2, key: "radioText", when: (eq(&rds_data_type, &4u64)), predicate_value: false },
            /// Program Service Name
            // Virtual schema indices: 5
            optional pub program_service_name: Vec<u8> => { bit: 3, key: "programServiceName", when: (eq(&rds_data_type, &30u64)), predicate_value: false },
            /// Block A
            // Virtual schema indices: 6
            optional pub block_a: u16 => { bit: 4, key: "blockA", when: (eq(&rds_data_type, &5u64)), predicate_value: false },
            /// Block B
            // Virtual schema indices: 7
            optional pub block_b: u16 => { bit: 5, key: "blockB", when: (eq(&rds_data_type, &5u64)), predicate_value: false },
            /// Block C
            // Virtual schema indices: 8
            optional pub block_c: u16 => { bit: 6, key: "blockC", when: (eq(&rds_data_type, &5u64)), predicate_value: false },
            /// Block D
            // Virtual schema indices: 9
            optional pub block_d: u16 => { bit: 7, key: "blockD", when: (eq(&rds_data_type, &5u64)), predicate_value: false },
            /// Block Errors
            // Virtual schema indices: 10
            optional pub block_errors: RetRdsDataBlockErrors => { bit: 8, key: "blockErrors", when: (eq(&rds_data_type, &5u64)), predicate_value: false },
        }
        steps {
            1 => required rds_data_type: RetRdsDataRdsDataType => { bit: 0, key: "rdsDataType", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: true, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            2 => optional char_set: RetRdsDataCharSet => { bit: 1, key: "charSet", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&rds_data_type, &4u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(4u8)] } },
            3 => optional radio_text: Vec<u8> => { bit: 2, key: "radioText", wire: [raw_bytes], project_wire: Iap1WireSpec::Raw, predicate_value: false, predicate: { expr: eq(&rds_data_type, &4u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(4u8)] } },
            4 => optional char_set: RetRdsDataCharSet => { bit: 1, key: "charSet", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&rds_data_type, &30u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(30u8)] } },
            5 => optional program_service_name: Vec<u8> => { bit: 3, key: "programServiceName", wire: [raw_bytes], project_wire: Iap1WireSpec::Raw, predicate_value: false, predicate: { expr: eq(&rds_data_type, &30u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(30u8)] } },
            6 => optional block_a: u16 => { bit: 4, key: "blockA", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&rds_data_type, &5u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(5u8)] } },
            7 => optional block_b: u16 => { bit: 5, key: "blockB", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&rds_data_type, &5u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(5u8)] } },
            8 => optional block_c: u16 => { bit: 6, key: "blockC", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&rds_data_type, &5u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(5u8)] } },
            9 => optional block_d: u16 => { bit: 7, key: "blockD", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&rds_data_type, &5u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(5u8)] } },
            10 => optional block_errors: RetRdsDataBlockErrors => { bit: 8, key: "blockErrors", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&rds_data_type, &5u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(5u8)] } },
        }
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x07, command = 0x001e,
        source = device,
        response = false,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct GetRdsNotifyMask {
        fields {
        }
        steps {
        }
    }
}

iap1_bitflags! {
    strings = LingoString;
    pub struct RetRdsNotifyMaskRdsDataChangeNotificationMask: u32 {
        /// RadioText (RT)
        const RADIO_TEXT_RT = 1 << 4;
        /// RDS/RDBS group data
        const RDS_RDBS_GROUP_DATA = 1 << 5;
        /// Program Service Name (PSN)
        const PROGRAM_SERVICE_NAME_PSN = 1 << 30;
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x07, command = 0x001f,
        source = accessory,
        response = true,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct RetRdsNotifyMask {
        fields {
            /// RDS/RBDS Data Change Notification Mask
            // Virtual schema indices: 1
            required pub rds_data_change_notification_mask: RetRdsNotifyMaskRdsDataChangeNotificationMask => { bit: 0, key: "rdsDataChangeNotificationMask", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required rds_data_change_notification_mask: RetRdsNotifyMaskRdsDataChangeNotificationMask => { bit: 0, key: "rdsDataChangeNotificationMask", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1_bitflags! {
    strings = LingoString;
    pub struct SetRdsNotifyMaskRdsDataChangeNotificationMask: u32 {
        /// RadioText (RT)
        const RADIO_TEXT_RT = 1 << 4;
        /// RDS/RDBS group data
        const RDS_RDBS_GROUP_DATA = 1 << 5;
        /// Program Service Name (PSN)
        const PROGRAM_SERVICE_NAME_PSN = 1 << 30;
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x07, command = 0x0020,
        source = device,
        response = false,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct SetRdsNotifyMask {
        fields {
            /// RDS/RBDS Data Change Notification Mask
            // Virtual schema indices: 1
            required pub rds_data_change_notification_mask: SetRdsNotifyMaskRdsDataChangeNotificationMask => { bit: 0, key: "rdsDataChangeNotificationMask", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required rds_data_change_notification_mask: SetRdsNotifyMaskRdsDataChangeNotificationMask => { bit: 0, key: "rdsDataChangeNotificationMask", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum RdsReadyNotifyRdsDataType: u8 {
        /// RadioText (RT)
        RadioTextRT = 4,
        /// RDS/RBDS group data
        RDSRBDSGroupData = 5,
        /// Program Service Name (PSN)
        ProgramServiceNamePSN = 30,
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum RdsReadyNotifyCharSet: u8 {
        /// Latin-based languages
        LatinBasedLanguages = 0,
        /// Cyrillic- and Greek-based languages
        CyrillicAndGreekBasedLanguages = 1,
        /// Arabic- and Hebrew-based languages
        ArabicAndHebrewBasedLanguages = 2,
    }
}

iap1_bitfield! {
    pub struct RdsReadyNotifyBlockErrors: u8 {
        #[group(mask = 0x3, shift = 0)]
        /// Block A Error Bits
        pub block_a_error_bits: RdsReadyNotifyBlockErrorsBlockAErrorBits {
            /// no errors
            NoErrors = 0,
            /// 1-2 errors; slight chance of uncorrected errors
            Value12ErrorsSlightChanceOfUncorrectedErrors = 1,
            /// 3-5 errors; may have uncorrected errors
            Value35ErrorsMayHaveUncorrectedErrors = 2,
            /// 6 or more errors; uncorrectable block
            Value6OrMoreErrorsUncorrectableBlock = 3,
        },
        #[group(mask = 0xc, shift = 2)]
        /// Block B Error Bits
        pub block_b_error_bits: RdsReadyNotifyBlockErrorsBlockBErrorBits {
            /// no errors
            NoErrors = 0,
            /// 1-2 errors; slight chance of uncorrected errors
            Value12ErrorsSlightChanceOfUncorrectedErrors = 1,
            /// 3-5 errors; may have uncorrected errors
            Value35ErrorsMayHaveUncorrectedErrors = 2,
            /// 6 or more errors; uncorrectable block
            Value6OrMoreErrorsUncorrectableBlock = 3,
        },
        #[group(mask = 0x30, shift = 4)]
        /// Block C Error Bits
        pub block_c_error_bits: RdsReadyNotifyBlockErrorsBlockCErrorBits {
            /// no errors
            NoErrors = 0,
            /// 1-2 errors; slight chance of uncorrected errors
            Value12ErrorsSlightChanceOfUncorrectedErrors = 1,
            /// 3-5 errors; may have uncorrected errors
            Value35ErrorsMayHaveUncorrectedErrors = 2,
            /// 6 or more errors; uncorrectable block
            Value6OrMoreErrorsUncorrectableBlock = 3,
        },
        #[group(mask = 0xc0, shift = 6)]
        /// Block D Error Bits
        pub block_d_error_bits: RdsReadyNotifyBlockErrorsBlockDErrorBits {
            /// no errors
            NoErrors = 0,
            /// 1-2 errors; slight chance of uncorrected errors
            Value12ErrorsSlightChanceOfUncorrectedErrors = 1,
            /// 3-5 errors; may have uncorrected errors
            Value35ErrorsMayHaveUncorrectedErrors = 2,
            /// 6 or more errors; uncorrectable block
            Value6OrMoreErrorsUncorrectableBlock = 3,
        },
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x07, command = 0x0021,
        source = accessory,
        response = false,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct RdsReadyNotify {
        fields {
            /// RDS/RBDS Data Type
            // Virtual schema indices: 1
            required pub rds_data_type: RdsReadyNotifyRdsDataType => { bit: 0, key: "rdsDataType", when: (truthy(&true)), predicate_value: true },
            /// Character Set
            // Virtual schema indices: 2, 4
            optional pub char_set: RdsReadyNotifyCharSet => { bit: 1, key: "charSet", when: (eq(&rds_data_type, &4u64)) || (eq(&rds_data_type, &30u64)), predicate_value: false },
            /// Radio Text
            // Virtual schema indices: 3
            optional pub radio_text: Vec<u8> => { bit: 2, key: "radioText", when: (eq(&rds_data_type, &4u64)), predicate_value: false },
            /// Program Service Name
            // Virtual schema indices: 5
            optional pub program_service_name: Vec<u8> => { bit: 3, key: "programServiceName", when: (eq(&rds_data_type, &30u64)), predicate_value: false },
            /// Block A
            // Virtual schema indices: 6
            optional pub block_a: u16 => { bit: 4, key: "blockA", when: (eq(&rds_data_type, &5u64)), predicate_value: false },
            /// Block B
            // Virtual schema indices: 7
            optional pub block_b: u16 => { bit: 5, key: "blockB", when: (eq(&rds_data_type, &5u64)), predicate_value: false },
            /// Block C
            // Virtual schema indices: 8
            optional pub block_c: u16 => { bit: 6, key: "blockC", when: (eq(&rds_data_type, &5u64)), predicate_value: false },
            /// Block D
            // Virtual schema indices: 9
            optional pub block_d: u16 => { bit: 7, key: "blockD", when: (eq(&rds_data_type, &5u64)), predicate_value: false },
            /// Block Errors
            // Virtual schema indices: 10
            optional pub block_errors: RdsReadyNotifyBlockErrors => { bit: 8, key: "blockErrors", when: (eq(&rds_data_type, &5u64)), predicate_value: false },
        }
        steps {
            1 => required rds_data_type: RdsReadyNotifyRdsDataType => { bit: 0, key: "rdsDataType", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: true, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            2 => optional char_set: RdsReadyNotifyCharSet => { bit: 1, key: "charSet", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&rds_data_type, &4u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(4u8)] } },
            3 => optional radio_text: Vec<u8> => { bit: 2, key: "radioText", wire: [raw_bytes], project_wire: Iap1WireSpec::Raw, predicate_value: false, predicate: { expr: eq(&rds_data_type, &4u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(4u8)] } },
            4 => optional char_set: RdsReadyNotifyCharSet => { bit: 1, key: "charSet", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&rds_data_type, &30u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(30u8)] } },
            5 => optional program_service_name: Vec<u8> => { bit: 3, key: "programServiceName", wire: [raw_bytes], project_wire: Iap1WireSpec::Raw, predicate_value: false, predicate: { expr: eq(&rds_data_type, &30u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(30u8)] } },
            6 => optional block_a: u16 => { bit: 4, key: "blockA", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&rds_data_type, &5u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(5u8)] } },
            7 => optional block_b: u16 => { bit: 5, key: "blockB", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&rds_data_type, &5u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(5u8)] } },
            8 => optional block_c: u16 => { bit: 6, key: "blockC", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&rds_data_type, &5u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(5u8)] } },
            9 => optional block_d: u16 => { bit: 7, key: "blockD", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&rds_data_type, &5u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(5u8)] } },
            10 => optional block_errors: RdsReadyNotifyBlockErrors => { bit: 8, key: "blockErrors", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&rds_data_type, &5u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(5u8)] } },
        }
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x07, command = 0x0025,
        source = device,
        response = false,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct GetHDProgramServiceCount {
        fields {
        }
        steps {
        }
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x07, command = 0x0026,
        source = accessory,
        response = true,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct RetHDProgramServiceCount {
        fields {
            /// HD Program Service Count
            // Virtual schema indices: 1
            required pub hd_program_service_count: u8 => { bit: 0, key: "hdProgramServiceCount", when: (truthy(&true)), predicate_value: false },
            /// Analog Program Exists
            // Virtual schema indices: 2
            required pub analog_program_exists: bool => { bit: 1, key: "analogProgramExists", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required hd_program_service_count: u8 => { bit: 0, key: "hdProgramServiceCount", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            2 => required analog_program_exists: bool => { bit: 1, key: "analogProgramExists", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x07, command = 0x0027,
        source = device,
        response = false,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct GetHDProgramService {
        fields {
        }
        steps {
        }
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum RetHDProgramServiceHdProgramServiceIndex: u8 {
        /// analog program
        AnalogProgram = 0,
        /// 1
        Value1 = 1,
        /// 2
        Value2 = 2,
        /// 3
        Value3 = 3,
        /// 4
        Value4 = 4,
        /// 5
        Value5 = 5,
        /// 6
        Value6 = 6,
        /// 7
        Value7 = 7,
        /// 8
        Value8 = 8,
        /// audio decoding and output disabled
        AudioDecodingAndOutputDisabled = 255,
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x07, command = 0x0028,
        source = accessory,
        response = true,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct RetHDProgramService {
        fields {
            /// HD Program Service Index
            // Virtual schema indices: 1
            required pub hd_program_service_index: RetHDProgramServiceHdProgramServiceIndex => { bit: 0, key: "hdProgramServiceIndex", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required hd_program_service_index: RetHDProgramServiceHdProgramServiceIndex => { bit: 0, key: "hdProgramServiceIndex", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum SetHDProgramServiceHdProgramServiceIndex: u8 {
        /// analog program
        AnalogProgram = 0,
        /// 1
        Value1 = 1,
        /// 2
        Value2 = 2,
        /// 3
        Value3 = 3,
        /// 4
        Value4 = 4,
        /// 5
        Value5 = 5,
        /// 6
        Value6 = 6,
        /// 7
        Value7 = 7,
        /// 8
        Value8 = 8,
        /// audio decoding and output disabled
        AudioDecodingAndOutputDisabled = 255,
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x07, command = 0x0029,
        source = device,
        response = false,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct SetHDProgramService {
        fields {
            /// HD Program Service Index
            // Virtual schema indices: 1
            required pub hd_program_service_index: SetHDProgramServiceHdProgramServiceIndex => { bit: 0, key: "hdProgramServiceIndex", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required hd_program_service_index: SetHDProgramServiceHdProgramServiceIndex => { bit: 0, key: "hdProgramServiceIndex", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x07, command = 0x002a,
        source = device,
        response = false,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct GetHDDataReadyStatus {
        fields {
        }
        steps {
        }
    }
}

iap1_bitflags! {
    strings = LingoString;
    pub struct RetHDDataReadyStatusHdDataReadyStatus: u32 {
        /// PSD data ready
        const PSD_DATA_READY = 1 << 0;
        /// SIS Station ID number data ready
        const SIS_STATION_ID_NUMBER_DATA_READY = 1 << 2;
        /// SIS Station Name (short) data ready
        const SIS_STATION_NAME_SHORT_DATA_READY = 1 << 3;
        /// SIS Station Name (long) data ready
        const SIS_STATION_NAME_LONG_DATA_READY = 1 << 4;
        /// SIS ALFN data ready
        const SIS_ALFN_DATA_READY = 1 << 5;
        /// SIS Station Location data ready
        const SIS_STATION_LOCATION_DATA_READY = 1 << 6;
        /// SIS Station Message data ready
        const SIS_STATION_MESSAGE_DATA_READY = 1 << 7;
        /// SIS Slogan data ready
        const SIS_SLOGAN_DATA_READY = 1 << 8;
        /// SIS Parameter Message data ready
        const SIS_PARAMETER_MESSAGE_DATA_READY = 1 << 9;
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x07, command = 0x002b,
        source = accessory,
        response = true,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct RetHDDataReadyStatus {
        fields {
            /// HD Data Ready Status Bits
            // Virtual schema indices: 1
            required pub hd_data_ready_status: RetHDDataReadyStatusHdDataReadyStatus => { bit: 0, key: "hdDataReadyStatus", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required hd_data_ready_status: RetHDDataReadyStatusHdDataReadyStatus => { bit: 0, key: "hdDataReadyStatus", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum GetHDDataHdDataType: u8 {
        /// PSD data
        PSDData = 0,
        /// SIS Station ID number data
        SISStationIDNumberData = 2,
        /// SIS Station Name (short) data
        SISStationNameShortData = 3,
        /// SIS Station Name (long) data
        SISStationNameLongData = 4,
        /// SIS ALFN data
        SISALFNData = 5,
        /// SIS Station Location data
        SISStationLocationData = 6,
        /// SIS Station Message data
        SISStationMessageData = 7,
        /// SIS Slogan data
        SISSloganData = 8,
        /// SIS Parameter Message data
        SISParameterMessageData = 9,
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x07, command = 0x002c,
        source = device,
        response = false,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct GetHDData {
        fields {
            /// HD Data Type
            // Virtual schema indices: 1
            required pub hd_data_type: GetHDDataHdDataType => { bit: 0, key: "hdDataType", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required hd_data_type: GetHDDataHdDataType => { bit: 0, key: "hdDataType", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum RetHDDataHdDataType: u8 {
        /// PSD data
        PSDData = 0,
        /// SIS Station ID number data
        SISStationIDNumberData = 2,
        /// SIS Station Name (short) data
        SISStationNameShortData = 3,
        /// SIS Station Name (long) data
        SISStationNameLongData = 4,
        /// SIS ALFN data
        SISALFNData = 5,
        /// SIS Station Location data
        SISStationLocationData = 6,
        /// SIS Station Message data
        SISStationMessageData = 7,
        /// SIS Slogan data
        SISSloganData = 8,
        /// SIS Parameter Message data
        SISParameterMessageData = 9,
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x07, command = 0x002d,
        source = accessory,
        response = true,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct RetHDData {
        fields {
            /// HD Data Type
            // Virtual schema indices: 1
            required pub hd_data_type: RetHDDataHdDataType => { bit: 0, key: "hdDataType", when: (truthy(&true)), predicate_value: false },
            /// HD Data
            // Virtual schema indices: 2
            required pub hd_data: Vec<u8> => { bit: 1, key: "hdData", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required hd_data_type: RetHDDataHdDataType => { bit: 0, key: "hdDataType", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            2 => required hd_data: Vec<u8> => { bit: 1, key: "hdData", wire: [raw_bytes], project_wire: Iap1WireSpec::Raw, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x07, command = 0x002e,
        source = device,
        response = false,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct GetHDDataNotifyMask {
        fields {
        }
        steps {
        }
    }
}

iap1_bitflags! {
    strings = LingoString;
    pub struct RetHDDataNotifyMaskHdDataChangeNotificationMask: u32 {
        /// PSD data ready
        const PSD_DATA_READY = 1 << 0;
        /// SIS Station ID number data ready
        const SIS_STATION_ID_NUMBER_DATA_READY = 1 << 2;
        /// SIS Station Name (short) data ready
        const SIS_STATION_NAME_SHORT_DATA_READY = 1 << 3;
        /// SIS Station Name (long) data ready
        const SIS_STATION_NAME_LONG_DATA_READY = 1 << 4;
        /// SIS ALFN data ready
        const SIS_ALFN_DATA_READY = 1 << 5;
        /// SIS Station Location data ready
        const SIS_STATION_LOCATION_DATA_READY = 1 << 6;
        /// SIS Station Message data ready
        const SIS_STATION_MESSAGE_DATA_READY = 1 << 7;
        /// SIS Slogan data ready
        const SIS_SLOGAN_DATA_READY = 1 << 8;
        /// SIS Parameter Message data ready
        const SIS_PARAMETER_MESSAGE_DATA_READY = 1 << 9;
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x07, command = 0x002f,
        source = accessory,
        response = true,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct RetHDDataNotifyMask {
        fields {
            /// HD Data Change Notification Mask
            // Virtual schema indices: 1
            required pub hd_data_change_notification_mask: RetHDDataNotifyMaskHdDataChangeNotificationMask => { bit: 0, key: "hdDataChangeNotificationMask", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required hd_data_change_notification_mask: RetHDDataNotifyMaskHdDataChangeNotificationMask => { bit: 0, key: "hdDataChangeNotificationMask", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1_bitflags! {
    strings = LingoString;
    pub struct SetHDDataNotifyMaskHdDataChangeNotificationMask: u32 {
        /// PSD data ready
        const PSD_DATA_READY = 1 << 0;
        /// SIS Station ID number data ready
        const SIS_STATION_ID_NUMBER_DATA_READY = 1 << 2;
        /// SIS Station Name (short) data ready
        const SIS_STATION_NAME_SHORT_DATA_READY = 1 << 3;
        /// SIS Station Name (long) data ready
        const SIS_STATION_NAME_LONG_DATA_READY = 1 << 4;
        /// SIS ALFN data ready
        const SIS_ALFN_DATA_READY = 1 << 5;
        /// SIS Station Location data ready
        const SIS_STATION_LOCATION_DATA_READY = 1 << 6;
        /// SIS Station Message data ready
        const SIS_STATION_MESSAGE_DATA_READY = 1 << 7;
        /// SIS Slogan data ready
        const SIS_SLOGAN_DATA_READY = 1 << 8;
        /// SIS Parameter Message data ready
        const SIS_PARAMETER_MESSAGE_DATA_READY = 1 << 9;
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x07, command = 0x0030,
        source = device,
        response = false,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct SetHDDataNotifyMask {
        fields {
            /// HD Data Change Notification Mask
            // Virtual schema indices: 1
            required pub hd_data_change_notification_mask: SetHDDataNotifyMaskHdDataChangeNotificationMask => { bit: 0, key: "hdDataChangeNotificationMask", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required hd_data_change_notification_mask: SetHDDataNotifyMaskHdDataChangeNotificationMask => { bit: 0, key: "hdDataChangeNotificationMask", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum HDDataReadyNotifyHdDataType: u8 {
        /// PSD data
        PSDData = 0,
        /// SIS Station ID number data
        SISStationIDNumberData = 2,
        /// SIS Station Name (short) data
        SISStationNameShortData = 3,
        /// SIS Station Name (long) data
        SISStationNameLongData = 4,
        /// SIS ALFN data
        SISALFNData = 5,
        /// SIS Station Location data
        SISStationLocationData = 6,
        /// SIS Station Message data
        SISStationMessageData = 7,
        /// SIS Slogan data
        SISSloganData = 8,
        /// SIS Parameter Message data
        SISParameterMessageData = 9,
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x07, command = 0x0031,
        source = accessory,
        response = false,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct HDDataReadyNotify {
        fields {
            /// HD Data Type
            // Virtual schema indices: 1
            required pub hd_data_type: HDDataReadyNotifyHdDataType => { bit: 0, key: "hdDataType", when: (truthy(&true)), predicate_value: false },
            /// HD Data
            // Virtual schema indices: 2
            required pub hd_data: Vec<u8> => { bit: 1, key: "hdData", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required hd_data_type: HDDataReadyNotifyHdDataType => { bit: 0, key: "hdDataType", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            2 => required hd_data: Vec<u8> => { bit: 1, key: "hdData", wire: [raw_bytes], project_wire: Iap1WireSpec::Raw, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1_registry! {
    0x07;
    AccessoryAck => 0x0000,
    GetTunerCaps => 0x0001,
    RetTunerCaps => 0x0002,
    GetTunerCtrl => 0x0003,
    RetTunerCtrl => 0x0004,
    SetTunerCtrl => 0x0005,
    GetTunerBand => 0x0006,
    RetTunerBand => 0x0007,
    SetTunerBand => 0x0008,
    GetTunerFreq => 0x0009,
    RetTunerFreq => 0x000a,
    SetTunerFreq => 0x000b,
    GetTunerMode => 0x000c,
    RetTunerMode => 0x000d,
    SetTunerMode => 0x000e,
    GetTunerSeekRssi => 0x000f,
    RetTunerSeekRssi => 0x0010,
    SetTunerSeekRssi => 0x0011,
    TunerSeekStart => 0x0012,
    TunerSeekDone => 0x0013,
    GetTunerStatus => 0x0014,
    RetTunerStatus => 0x0015,
    GetStatusNotifyMask => 0x0016,
    RetStatusNotifyMask => 0x0017,
    SetStatusNotifyMask => 0x0018,
    StatusChangeNotify => 0x0019,
    GetRdsReadyStatus => 0x001a,
    RetRdsReadyStatus => 0x001b,
    GetRdsData => 0x001c,
    RetRdsData => 0x001d,
    GetRdsNotifyMask => 0x001e,
    RetRdsNotifyMask => 0x001f,
    SetRdsNotifyMask => 0x0020,
    RdsReadyNotify => 0x0021,
    GetHDProgramServiceCount => 0x0025,
    RetHDProgramServiceCount => 0x0026,
    GetHDProgramService => 0x0027,
    RetHDProgramService => 0x0028,
    SetHDProgramService => 0x0029,
    GetHDDataReadyStatus => 0x002a,
    RetHDDataReadyStatus => 0x002b,
    GetHDData => 0x002c,
    RetHDData => 0x002d,
    GetHDDataNotifyMask => 0x002e,
    RetHDDataNotifyMask => 0x002f,
    SetHDDataNotifyMask => 0x0030,
    HDDataReadyNotify => 0x0031,
}
