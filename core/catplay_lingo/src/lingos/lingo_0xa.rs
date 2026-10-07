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

// Lingo 0x0a: Digital Audio
iap1_enum! {
    strings = LingoString;
    pub enum AccessoryAckCommandResult: u8 {
        /// OK
        OK = 0,
        /// failed
        Failed = 2,
        /// out of resources
        OutOfResources = 3,
        /// bad parameter
        BadParameter = 4,
        /// unknown ID
        UnknownID = 5,
        /// not authenticated
        NotAuthenticated = 7,
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x0a, command = 0x0000,
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
            required pub command_result: AccessoryAckCommandResult => { bit: 0, key: "commandResult", when: (truthy(&true)), predicate_value: false },
            /// Command ID
            // Virtual schema indices: 2
            required pub acked_command_id: u8 => { bit: 1, key: "ackedCommandID", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required command_result: AccessoryAckCommandResult => { bit: 0, key: "commandResult", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            2 => required acked_command_id: u8 => { bit: 1, key: "ackedCommandID", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

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
        context = ctx, strings = LingoString, lingo = 0x0a, command = 0x0001,
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
        context = ctx, strings = LingoString, lingo = 0x0a, command = 0x0002,
        source = device,
        response = false,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct GetAccessorySampleRateCaps {
        fields {
        }
        steps {
        }
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum RetAccessorySampleRateCapsSampleRatesSampleRate: u32 {
        /// 8000 Hz
        Value8000Hz = 8000,
        /// 11025 Hz
        Value11025Hz = 11025,
        /// 12000 Hz
        Value12000Hz = 12000,
        /// 16000 Hz
        Value16000Hz = 16000,
        /// 22050 Hz
        Value22050Hz = 22050,
        /// 24000 Hz
        Value24000Hz = 24000,
        /// 32000 Hz
        Value32000Hz = 32000,
        /// 44100 Hz
        Value44100Hz = 44100,
        /// 48000 Hz
        Value48000Hz = 48000,
    }
}

iap1_record! {
    strings = LingoString;
    pub struct RetAccessorySampleRateCapsSampleRates {
        fields {
            /// sample rate
            // Virtual schema indices: 1
            required pub sample_rate: RetAccessorySampleRateCapsSampleRatesSampleRate => { bit: 0, key: "sampleRate", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required sample_rate: RetAccessorySampleRateCapsSampleRatesSampleRate => { bit: 0, key: "sampleRate", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x0a, command = 0x0003,
        source = accessory,
        response = true,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct RetAccessorySampleRateCaps {
        fields {
            /// Sample Rates
            // Virtual schema indices: 1
            required pub sample_rates: Vec<RetAccessorySampleRateCapsSampleRates> => { bit: 0, key: "sampleRates", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required sample_rates: Vec<RetAccessorySampleRateCapsSampleRates> => { bit: 0, key: "sampleRates", wire: [counted_remaining], project_wire: Iap1WireSpec::CountedRemaining, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum TrackNewAudioAttributesNewSampleRate: u32 {
        /// 8000 Hz
        Value8000Hz = 8000,
        /// 11025 Hz
        Value11025Hz = 11025,
        /// 12000 Hz
        Value12000Hz = 12000,
        /// 16000 Hz
        Value16000Hz = 16000,
        /// 22050 Hz
        Value22050Hz = 22050,
        /// 24000 Hz
        Value24000Hz = 24000,
        /// 32000 Hz
        Value32000Hz = 32000,
        /// 44100 Hz
        Value44100Hz = 44100,
        /// 48000 Hz
        Value48000Hz = 48000,
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x0a, command = 0x0004,
        source = device,
        response = false,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct TrackNewAudioAttributes {
        fields {
            /// New Sample Rate
            // Virtual schema indices: 1
            required pub new_sample_rate: TrackNewAudioAttributesNewSampleRate => { bit: 0, key: "newSampleRate", when: (truthy(&true)), predicate_value: false },
            /// New Sound Check Value
            // Virtual schema indices: 2
            required pub new_sound_check_value: i32 => { bit: 1, key: "newSoundCheckValue", when: (truthy(&true)), predicate_value: false },
            /// New Track Volume Adjustment
            // Virtual schema indices: 3
            required pub new_track_volume_adjustment: i32 => { bit: 2, key: "newTrackVolumeAdjustment", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required new_sample_rate: TrackNewAudioAttributesNewSampleRate => { bit: 0, key: "newSampleRate", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            2 => required new_sound_check_value: i32 => { bit: 1, key: "newSoundCheckValue", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            3 => required new_track_volume_adjustment: i32 => { bit: 2, key: "newTrackVolumeAdjustment", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x0a, command = 0x0005,
        source = accessory,
        response = false,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct SetVideoDelay {
        fields {
            /// Video Delay
            // Virtual schema indices: 1
            required pub video_delay: u32 => { bit: 0, key: "videoDelay", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required video_delay: u32 => { bit: 0, key: "videoDelay", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1_registry! {
    0x0a;
    AccessoryAck => 0x0000,
    IPodAck => 0x0001,
    GetAccessorySampleRateCaps => 0x0002,
    RetAccessorySampleRateCaps => 0x0003,
    TrackNewAudioAttributes => 0x0004,
    SetVideoDelay => 0x0005,
}
