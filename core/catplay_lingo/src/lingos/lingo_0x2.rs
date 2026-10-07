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

// Lingo 0x02: Simple Remote
iap1_bitflags! {
    strings = LingoString;
    pub struct ContextButtonStatusButtonStates0: u8 {
        /// play/pause
        const PLAY_PAUSE = 1 << 0;
        /// volume up
        const VOLUME_UP = 1 << 1;
        /// volume down
        const VOLUME_DOWN = 1 << 2;
        /// next track
        const NEXT_TRACK = 1 << 3;
        /// previous track
        const PREVIOUS_TRACK = 1 << 4;
        /// next album
        const NEXT_ALBUM = 1 << 5;
        /// previous album
        const PREVIOUS_ALBUM = 1 << 6;
        /// stop
        const STOP = 1 << 7;
    }
}

iap1_bitflags! {
    strings = LingoString;
    pub struct ContextButtonStatusButtonStates1: u8 {
        /// play/resume
        const PLAY_RESUME = 1 << 0;
        /// pause
        const PAUSE = 1 << 1;
        /// mute toggle
        const MUTE_TOGGLE = 1 << 2;
        /// next chapter
        const NEXT_CHAPTER = 1 << 3;
        /// previous chapter
        const PREVIOUS_CHAPTER = 1 << 4;
        /// next playlist
        const NEXT_PLAYLIST = 1 << 5;
        /// previous playlist
        const PREVIOUS_PLAYLIST = 1 << 6;
        /// shuffle setting advance
        const SHUFFLE_SETTING_ADVANCE = 1 << 7;
    }
}

iap1_bitflags! {
    strings = LingoString;
    pub struct ContextButtonStatusButtonStates2: u8 {
        /// repeat setting advance
        const REPEAT_SETTING_ADVANCE = 1 << 0;
        /// power on
        const POWER_ON = 1 << 1;
        /// power off
        const POWER_OFF = 1 << 2;
        /// backlight for 30 seconds
        const BACKLIGHT_FOR_30_SECONDS = 1 << 3;
        /// begin fast forward
        const BEGIN_FAST_FORWARD = 1 << 4;
        /// begin rewind
        const BEGIN_REWIND = 1 << 5;
        /// menu
        const MENU = 1 << 6;
        /// select
        const SELECT = 1 << 7;
    }
}

iap1_bitflags! {
    strings = LingoString;
    pub struct ContextButtonStatusButtonStates3: u8 {
        /// up arrow
        const UP_ARROW = 1 << 0;
        /// down arrow
        const DOWN_ARROW = 1 << 1;
        /// backlight off
        const BACKLIGHT_OFF = 1 << 2;
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x02, command = 0x0000,
        source = accessory,
        response = false,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct ContextButtonStatus {
        fields {
            /// Button States 0
            // Virtual schema indices: 1
            required pub button_states0: ContextButtonStatusButtonStates0 => { bit: 0, key: "buttonStates0", when: (truthy(&true)), predicate_value: false },
            /// Button States 1
            // Virtual schema indices: 2
            length_optional pub button_states1: ContextButtonStatusButtonStates1 => { bit: 1, key: "buttonStates1", when: (ctx.encoding() || (gt(&ctx.adjusted_payload_length, &1u64))), predicate_value: false },
            /// Button States 2
            // Virtual schema indices: 3
            length_optional pub button_states2: ContextButtonStatusButtonStates2 => { bit: 2, key: "buttonStates2", when: (ctx.encoding() || (gt(&ctx.adjusted_payload_length, &2u64))), predicate_value: false },
            /// Button States 3
            // Virtual schema indices: 4
            length_optional pub button_states3: ContextButtonStatusButtonStates3 => { bit: 3, key: "buttonStates3", when: (ctx.encoding() || (gt(&ctx.adjusted_payload_length, &3u64))), predicate_value: false },
        }
        steps {
            1 => required button_states0: ContextButtonStatusButtonStates0 => { bit: 0, key: "buttonStates0", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            2 => length_optional button_states1: ContextButtonStatusButtonStates1 => { bit: 1, key: "buttonStates1", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: ctx.encoding() || (gt(&ctx.adjusted_payload_length, &1u64)), program: [FlatPredicateToken::Or, FlatPredicateToken::Encoding, FlatPredicateToken::Gt, FlatPredicateToken::AdjustedPayloadLength, FlatPredicateToken::Integer(1u8)] } },
            3 => length_optional button_states2: ContextButtonStatusButtonStates2 => { bit: 2, key: "buttonStates2", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: ctx.encoding() || (gt(&ctx.adjusted_payload_length, &2u64)), program: [FlatPredicateToken::Or, FlatPredicateToken::Encoding, FlatPredicateToken::Gt, FlatPredicateToken::AdjustedPayloadLength, FlatPredicateToken::Integer(2u8)] } },
            4 => length_optional button_states3: ContextButtonStatusButtonStates3 => { bit: 3, key: "buttonStates3", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: ctx.encoding() || (gt(&ctx.adjusted_payload_length, &3u64)), program: [FlatPredicateToken::Or, FlatPredicateToken::Encoding, FlatPredicateToken::Gt, FlatPredicateToken::AdjustedPayloadLength, FlatPredicateToken::Integer(3u8)] } },
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
        context = ctx, strings = LingoString, lingo = 0x02, command = 0x0001,
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

iap1_bitflags! {
    strings = LingoString;
    pub struct ImageButtonStatusButtonStates0: u8 {
        /// play/pause
        const PLAY_PAUSE = 1 << 0;
        /// next image
        const NEXT_IMAGE = 1 << 1;
        /// previous image
        const PREVIOUS_IMAGE = 1 << 2;
        /// stop
        const STOP = 1 << 3;
        /// play/resume
        const PLAY_RESUME = 1 << 4;
        /// pause
        const PAUSE = 1 << 5;
        /// shuffle advance
        const SHUFFLE_ADVANCE = 1 << 6;
        /// repeat advance
        const REPEAT_ADVANCE = 1 << 7;
    }
}

iap1_bitflags! {
    strings = LingoString;
    pub struct ImageButtonStatusButtonStates1: u8 {
    }
}

iap1_bitflags! {
    strings = LingoString;
    pub struct ImageButtonStatusButtonStates2: u8 {
    }
}

iap1_bitflags! {
    strings = LingoString;
    pub struct ImageButtonStatusButtonStates3: u8 {
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x02, command = 0x0002,
        source = accessory,
        response = false,
        ack = false,
        deprecated = true,
        transaction_id = permitted
    )]
    pub struct ImageButtonStatus {
        fields {
            /// Button States 0
            // Virtual schema indices: 1
            required pub button_states0: ImageButtonStatusButtonStates0 => { bit: 0, key: "buttonStates0", when: (truthy(&true)), predicate_value: false },
            /// Button States 1
            // Virtual schema indices: 2
            length_optional pub button_states1: ImageButtonStatusButtonStates1 => { bit: 1, key: "buttonStates1", when: (ctx.encoding() || (gt(&ctx.adjusted_payload_length, &1u64))), predicate_value: false },
            /// Button States 2
            // Virtual schema indices: 3
            length_optional pub button_states2: ImageButtonStatusButtonStates2 => { bit: 2, key: "buttonStates2", when: (ctx.encoding() || (gt(&ctx.adjusted_payload_length, &2u64))), predicate_value: false },
            /// Button States 3
            // Virtual schema indices: 4
            length_optional pub button_states3: ImageButtonStatusButtonStates3 => { bit: 3, key: "buttonStates3", when: (ctx.encoding() || (gt(&ctx.adjusted_payload_length, &3u64))), predicate_value: false },
        }
        steps {
            1 => required button_states0: ImageButtonStatusButtonStates0 => { bit: 0, key: "buttonStates0", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            2 => length_optional button_states1: ImageButtonStatusButtonStates1 => { bit: 1, key: "buttonStates1", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: ctx.encoding() || (gt(&ctx.adjusted_payload_length, &1u64)), program: [FlatPredicateToken::Or, FlatPredicateToken::Encoding, FlatPredicateToken::Gt, FlatPredicateToken::AdjustedPayloadLength, FlatPredicateToken::Integer(1u8)] } },
            3 => length_optional button_states2: ImageButtonStatusButtonStates2 => { bit: 2, key: "buttonStates2", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: ctx.encoding() || (gt(&ctx.adjusted_payload_length, &2u64)), program: [FlatPredicateToken::Or, FlatPredicateToken::Encoding, FlatPredicateToken::Gt, FlatPredicateToken::AdjustedPayloadLength, FlatPredicateToken::Integer(2u8)] } },
            4 => length_optional button_states3: ImageButtonStatusButtonStates3 => { bit: 3, key: "buttonStates3", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: ctx.encoding() || (gt(&ctx.adjusted_payload_length, &3u64)), program: [FlatPredicateToken::Or, FlatPredicateToken::Encoding, FlatPredicateToken::Gt, FlatPredicateToken::AdjustedPayloadLength, FlatPredicateToken::Integer(3u8)] } },
        }
    }
}

iap1_bitflags! {
    strings = LingoString;
    pub struct VideoButtonStatusButtonStates0: u8 {
        /// play/pause
        const PLAY_PAUSE = 1 << 0;
        /// next video
        const NEXT_VIDEO = 1 << 1;
        /// previous video
        const PREVIOUS_VIDEO = 1 << 2;
        /// stop
        const STOP = 1 << 3;
        /// play/resume
        const PLAY_RESUME = 1 << 4;
        /// pause
        const PAUSE = 1 << 5;
        /// begin fast forward
        const BEGIN_FAST_FORWARD = 1 << 6;
        /// begin rewind
        const BEGIN_REWIND = 1 << 7;
    }
}

iap1_bitflags! {
    strings = LingoString;
    pub struct VideoButtonStatusButtonStates1: u8 {
        /// next chapter
        const NEXT_CHAPTER = 1 << 0;
        /// previous chapter
        const PREVIOUS_CHAPTER = 1 << 1;
        /// next frame
        const NEXT_FRAME = 1 << 2;
        /// previous frame
        const PREVIOUS_FRAME = 1 << 3;
    }
}

iap1_bitflags! {
    strings = LingoString;
    pub struct VideoButtonStatusButtonStates2: u8 {
    }
}

iap1_bitflags! {
    strings = LingoString;
    pub struct VideoButtonStatusButtonStates3: u8 {
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x02, command = 0x0003,
        source = accessory,
        response = false,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct VideoButtonStatus {
        fields {
            /// Button States 0
            // Virtual schema indices: 1
            required pub button_states0: VideoButtonStatusButtonStates0 => { bit: 0, key: "buttonStates0", when: (truthy(&true)), predicate_value: false },
            /// Button States 1
            // Virtual schema indices: 2
            length_optional pub button_states1: VideoButtonStatusButtonStates1 => { bit: 1, key: "buttonStates1", when: (ctx.encoding() || (gt(&ctx.adjusted_payload_length, &1u64))), predicate_value: false },
            /// Button States 2
            // Virtual schema indices: 3
            length_optional pub button_states2: VideoButtonStatusButtonStates2 => { bit: 2, key: "buttonStates2", when: (ctx.encoding() || (gt(&ctx.adjusted_payload_length, &2u64))), predicate_value: false },
            /// Button States 3
            // Virtual schema indices: 4
            length_optional pub button_states3: VideoButtonStatusButtonStates3 => { bit: 3, key: "buttonStates3", when: (ctx.encoding() || (gt(&ctx.adjusted_payload_length, &3u64))), predicate_value: false },
        }
        steps {
            1 => required button_states0: VideoButtonStatusButtonStates0 => { bit: 0, key: "buttonStates0", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            2 => length_optional button_states1: VideoButtonStatusButtonStates1 => { bit: 1, key: "buttonStates1", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: ctx.encoding() || (gt(&ctx.adjusted_payload_length, &1u64)), program: [FlatPredicateToken::Or, FlatPredicateToken::Encoding, FlatPredicateToken::Gt, FlatPredicateToken::AdjustedPayloadLength, FlatPredicateToken::Integer(1u8)] } },
            3 => length_optional button_states2: VideoButtonStatusButtonStates2 => { bit: 2, key: "buttonStates2", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: ctx.encoding() || (gt(&ctx.adjusted_payload_length, &2u64)), program: [FlatPredicateToken::Or, FlatPredicateToken::Encoding, FlatPredicateToken::Gt, FlatPredicateToken::AdjustedPayloadLength, FlatPredicateToken::Integer(2u8)] } },
            4 => length_optional button_states3: VideoButtonStatusButtonStates3 => { bit: 3, key: "buttonStates3", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: ctx.encoding() || (gt(&ctx.adjusted_payload_length, &3u64)), program: [FlatPredicateToken::Or, FlatPredicateToken::Encoding, FlatPredicateToken::Gt, FlatPredicateToken::AdjustedPayloadLength, FlatPredicateToken::Integer(3u8)] } },
        }
    }
}

iap1_bitflags! {
    strings = LingoString;
    pub struct AudioButtonStatusButtonStates0: u8 {
        /// play/pause
        const PLAY_PAUSE = 1 << 0;
        /// volume up
        const VOLUME_UP = 1 << 1;
        /// volume down
        const VOLUME_DOWN = 1 << 2;
        /// next track
        const NEXT_TRACK = 1 << 3;
        /// previous track
        const PREVIOUS_TRACK = 1 << 4;
        /// next album
        const NEXT_ALBUM = 1 << 5;
        /// previous album
        const PREVIOUS_ALBUM = 1 << 6;
        /// stop
        const STOP = 1 << 7;
    }
}

iap1_bitflags! {
    strings = LingoString;
    pub struct AudioButtonStatusButtonStates1: u8 {
        /// play/resume
        const PLAY_RESUME = 1 << 0;
        /// pause
        const PAUSE = 1 << 1;
        /// mute toggle
        const MUTE_TOGGLE = 1 << 2;
        /// next chapter
        const NEXT_CHAPTER = 1 << 3;
        /// previous chapter
        const PREVIOUS_CHAPTER = 1 << 4;
        /// next playlist
        const NEXT_PLAYLIST = 1 << 5;
        /// previous playlist
        const PREVIOUS_PLAYLIST = 1 << 6;
        /// shuffle setting advance
        const SHUFFLE_SETTING_ADVANCE = 1 << 7;
    }
}

iap1_bitflags! {
    strings = LingoString;
    pub struct AudioButtonStatusButtonStates2: u8 {
        /// repeat setting advance
        const REPEAT_SETTING_ADVANCE = 1 << 0;
        /// begin fast forward
        const BEGIN_FAST_FORWARD = 1 << 1;
        /// begin rewind
        const BEGIN_REWIND = 1 << 2;
        /// record
        const RECORD = 1 << 3;
    }
}

iap1_bitflags! {
    strings = LingoString;
    pub struct AudioButtonStatusButtonStates3: u8 {
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x02, command = 0x0004,
        source = accessory,
        response = false,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct AudioButtonStatus {
        fields {
            /// Button States 0
            // Virtual schema indices: 1
            required pub button_states0: AudioButtonStatusButtonStates0 => { bit: 0, key: "buttonStates0", when: (truthy(&true)), predicate_value: false },
            /// Button States 1
            // Virtual schema indices: 2
            length_optional pub button_states1: AudioButtonStatusButtonStates1 => { bit: 1, key: "buttonStates1", when: (ctx.encoding() || (gt(&ctx.adjusted_payload_length, &1u64))), predicate_value: false },
            /// Button States 2
            // Virtual schema indices: 3
            length_optional pub button_states2: AudioButtonStatusButtonStates2 => { bit: 2, key: "buttonStates2", when: (ctx.encoding() || (gt(&ctx.adjusted_payload_length, &2u64))), predicate_value: false },
            /// Button States 3
            // Virtual schema indices: 4
            length_optional pub button_states3: AudioButtonStatusButtonStates3 => { bit: 3, key: "buttonStates3", when: (ctx.encoding() || (gt(&ctx.adjusted_payload_length, &3u64))), predicate_value: false },
        }
        steps {
            1 => required button_states0: AudioButtonStatusButtonStates0 => { bit: 0, key: "buttonStates0", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            2 => length_optional button_states1: AudioButtonStatusButtonStates1 => { bit: 1, key: "buttonStates1", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: ctx.encoding() || (gt(&ctx.adjusted_payload_length, &1u64)), program: [FlatPredicateToken::Or, FlatPredicateToken::Encoding, FlatPredicateToken::Gt, FlatPredicateToken::AdjustedPayloadLength, FlatPredicateToken::Integer(1u8)] } },
            3 => length_optional button_states2: AudioButtonStatusButtonStates2 => { bit: 2, key: "buttonStates2", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: ctx.encoding() || (gt(&ctx.adjusted_payload_length, &2u64)), program: [FlatPredicateToken::Or, FlatPredicateToken::Encoding, FlatPredicateToken::Gt, FlatPredicateToken::AdjustedPayloadLength, FlatPredicateToken::Integer(2u8)] } },
            4 => length_optional button_states3: AudioButtonStatusButtonStates3 => { bit: 3, key: "buttonStates3", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: ctx.encoding() || (gt(&ctx.adjusted_payload_length, &3u64)), program: [FlatPredicateToken::Or, FlatPredicateToken::Encoding, FlatPredicateToken::Gt, FlatPredicateToken::AdjustedPayloadLength, FlatPredicateToken::Integer(3u8)] } },
        }
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum IPodOutButtonStatusButtonSource: u8 {
        /// car center console
        CarCenterConsole = 0,
        /// steering wheel
        SteeringWheel = 1,
        /// car dashboard
        CarDashboard = 2,
    }
}

iap1_bitflags! {
    strings = LingoString;
    pub struct IPodOutButtonStatusButtonStates0: u8 {
        /// select
        const SELECT = 1 << 0;
        /// left
        const LEFT = 1 << 1;
        /// right
        const RIGHT = 1 << 2;
        /// up
        const UP = 1 << 3;
        /// down
        const DOWN = 1 << 4;
        /// menu
        const MENU = 1 << 5;
    }
}

iap1_bitflags! {
    strings = LingoString;
    pub struct IPodOutButtonStatusButtonStates1: u8 {
    }
}

iap1_bitflags! {
    strings = LingoString;
    pub struct IPodOutButtonStatusButtonStates2: u8 {
    }
}

iap1_bitflags! {
    strings = LingoString;
    pub struct IPodOutButtonStatusButtonStates3: u8 {
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x02, command = 0x000b,
        source = accessory,
        response = false,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct IPodOutButtonStatus {
        fields {
            /// Button Source
            // Virtual schema indices: 1
            required pub button_source: IPodOutButtonStatusButtonSource => { bit: 0, key: "buttonSource", when: (truthy(&true)), predicate_value: false },
            /// Button States 0
            // Virtual schema indices: 2
            required pub button_states0: IPodOutButtonStatusButtonStates0 => { bit: 1, key: "buttonStates0", when: (truthy(&true)), predicate_value: false },
            /// Button States 1
            // Virtual schema indices: 3
            length_optional pub button_states1: IPodOutButtonStatusButtonStates1 => { bit: 2, key: "buttonStates1", when: (ctx.encoding() || (gt(&ctx.adjusted_payload_length, &2u64))), predicate_value: false },
            /// Button States 2
            // Virtual schema indices: 4
            length_optional pub button_states2: IPodOutButtonStatusButtonStates2 => { bit: 3, key: "buttonStates2", when: (ctx.encoding() || (gt(&ctx.adjusted_payload_length, &3u64))), predicate_value: false },
            /// Button States 3
            // Virtual schema indices: 5
            length_optional pub button_states3: IPodOutButtonStatusButtonStates3 => { bit: 4, key: "buttonStates3", when: (ctx.encoding() || (gt(&ctx.adjusted_payload_length, &4u64))), predicate_value: false },
        }
        steps {
            1 => required button_source: IPodOutButtonStatusButtonSource => { bit: 0, key: "buttonSource", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            2 => required button_states0: IPodOutButtonStatusButtonStates0 => { bit: 1, key: "buttonStates0", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            3 => length_optional button_states1: IPodOutButtonStatusButtonStates1 => { bit: 2, key: "buttonStates1", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: ctx.encoding() || (gt(&ctx.adjusted_payload_length, &2u64)), program: [FlatPredicateToken::Or, FlatPredicateToken::Encoding, FlatPredicateToken::Gt, FlatPredicateToken::AdjustedPayloadLength, FlatPredicateToken::Integer(2u8)] } },
            4 => length_optional button_states2: IPodOutButtonStatusButtonStates2 => { bit: 3, key: "buttonStates2", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: ctx.encoding() || (gt(&ctx.adjusted_payload_length, &3u64)), program: [FlatPredicateToken::Or, FlatPredicateToken::Encoding, FlatPredicateToken::Gt, FlatPredicateToken::AdjustedPayloadLength, FlatPredicateToken::Integer(3u8)] } },
            5 => length_optional button_states3: IPodOutButtonStatusButtonStates3 => { bit: 4, key: "buttonStates3", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: ctx.encoding() || (gt(&ctx.adjusted_payload_length, &4u64)), program: [FlatPredicateToken::Or, FlatPredicateToken::Encoding, FlatPredicateToken::Gt, FlatPredicateToken::AdjustedPayloadLength, FlatPredicateToken::Integer(4u8)] } },
        }
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum RotationInputStatusRotationSource: u8 {
        /// car center console
        CarCenterConsole = 0,
        /// steering wheel
        SteeringWheel = 1,
        /// car dashboard
        CarDashboard = 2,
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum RotationInputStatusControllerType: u8 {
        /// free wheel
        FreeWheel = 0,
        /// jog wheel
        JogWheel = 1,
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum RotationInputStatusRotationDirection: u8 {
        /// counterclockwise
        Counterclockwise = 0,
        /// clockwise
        Clockwise = 1,
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum RotationInputStatusRotationAction: u8 {
        /// completed
        Completed = 0,
        /// in progress
        InProgress = 1,
        /// repeating
        Repeating = 2,
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum RotationInputStatusRotationType: u8 {
        /// detents
        Detents = 0,
        /// degrees
        Degrees = 1,
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x02, command = 0x000c,
        source = accessory,
        response = false,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct RotationInputStatus {
        fields {
            /// User Action Duration
            // Virtual schema indices: 1
            required pub user_action_duration_ms: u32 => { bit: 0, key: "userActionDurationMs", when: (truthy(&true)), predicate_value: false },
            /// Rotation Source
            // Virtual schema indices: 2
            required pub rotation_source: RotationInputStatusRotationSource => { bit: 1, key: "rotationSource", when: (truthy(&true)), predicate_value: false },
            /// Controller Type
            // Virtual schema indices: 3
            required pub controller_type: RotationInputStatusControllerType => { bit: 2, key: "controllerType", when: (truthy(&true)), predicate_value: false },
            /// Rotation Direction
            // Virtual schema indices: 4
            required pub rotation_direction: RotationInputStatusRotationDirection => { bit: 3, key: "rotationDirection", when: (truthy(&true)), predicate_value: false },
            /// Rotation Action
            // Virtual schema indices: 5
            required pub rotation_action: RotationInputStatusRotationAction => { bit: 4, key: "rotationAction", when: (truthy(&true)), predicate_value: false },
            /// Rotation Type
            // Virtual schema indices: 6
            required pub rotation_type: RotationInputStatusRotationType => { bit: 5, key: "rotationType", when: (truthy(&true)), predicate_value: false },
            /// Detents or Degrees Moved
            // Virtual schema indices: 7
            required pub detents_or_degrees_moved: u16 => { bit: 6, key: "detentsOrDegreesMoved", when: (truthy(&true)), predicate_value: false },
            /// Total Detents or Degrees
            // Virtual schema indices: 8
            required pub detents_or_degrees_total: u16 => { bit: 7, key: "detentsOrDegreesTotal", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required user_action_duration_ms: u32 => { bit: 0, key: "userActionDurationMs", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            2 => required rotation_source: RotationInputStatusRotationSource => { bit: 1, key: "rotationSource", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            3 => required controller_type: RotationInputStatusControllerType => { bit: 2, key: "controllerType", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            4 => required rotation_direction: RotationInputStatusRotationDirection => { bit: 3, key: "rotationDirection", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            5 => required rotation_action: RotationInputStatusRotationAction => { bit: 4, key: "rotationAction", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            6 => required rotation_type: RotationInputStatusRotationType => { bit: 5, key: "rotationType", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            7 => required detents_or_degrees_moved: u16 => { bit: 6, key: "detentsOrDegreesMoved", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            8 => required detents_or_degrees_total: u16 => { bit: 7, key: "detentsOrDegreesTotal", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum RadioButtonStatusButtonStatus: u8 {
        /// released
        Released = 0,
        /// tag current song
        TagCurrentSong = 2,
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x02, command = 0x000d,
        source = accessory,
        response = false,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct RadioButtonStatus {
        fields {
            /// Button Status
            // Virtual schema indices: 1
            required pub button_status: RadioButtonStatusButtonStatus => { bit: 0, key: "buttonStatus", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required button_status: RadioButtonStatusButtonStatus => { bit: 0, key: "buttonStatus", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum CameraButtonStatusButtonStatus: u8 {
        /// up
        Up = 0,
        /// down
        Down = 1,
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x02, command = 0x000e,
        source = accessory,
        response = false,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct CameraButtonStatus {
        fields {
            /// Button Status
            // Virtual schema indices: 1
            required pub button_status: CameraButtonStatusButtonStatus => { bit: 0, key: "buttonStatus", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required button_status: CameraButtonStatusButtonStatus => { bit: 0, key: "buttonStatus", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum RegisterDescriptorCountryCode: u8 {
        /// not supported/undefined
        NotSupportedUndefined = 0,
        /// Arabic
        Arabic = 1,
        /// Belgian
        Belgian = 2,
        /// Canadian-Bilingual
        CanadianBilingual = 3,
        /// Canadian-French
        CanadianFrench = 4,
        /// Czech Republic
        CzechRepublic = 5,
        /// Danish
        Danish = 6,
        /// Finnish
        Finnish = 7,
        /// French
        French = 8,
        /// German
        German = 9,
        /// Greek
        Greek = 10,
        /// Hebrew
        Hebrew = 11,
        /// Hungarian
        Hungarian = 12,
        /// International (ISO)
        InternationalISO = 13,
        /// Italian
        Italian = 14,
        /// Japan (Katakana)
        JapanKatakana = 15,
        /// Korean
        Korean = 16,
        /// Latin American
        LatinAmerican = 17,
        /// Netherlands/Dutch
        NetherlandsDutch = 18,
        /// Norwegian
        Norwegian = 19,
        /// Persian (Farsi)
        PersianFarsi = 20,
        /// Poland
        Poland = 21,
        /// Portuguese
        Portuguese = 22,
        /// Russian
        Russian = 23,
        /// Slovakia
        Slovakia = 24,
        /// Spanish
        Spanish = 25,
        /// Swedish
        Swedish = 26,
        /// Swiss/French
        SwissFrench = 27,
        /// Swiss/German
        SwissGerman = 28,
        /// Switzerland
        Switzerland = 29,
        /// Taiwan
        Taiwan = 30,
        /// Turkish-Q
        TurkishQ = 31,
        /// UK
        UK = 32,
        /// US
        US = 33,
        /// Croatian
        Croatian = 34,
        /// Turkish-F
        TurkishF = 35,
        /// Thai
        Thai = 250,
        /// Flemish
        Flemish = 251,
        /// Romanian
        Romanian = 252,
        /// Bulgarian
        Bulgarian = 253,
        /// Chinese (Simplified)
        ChineseSimplified = 254,
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x02, command = 0x000f,
        source = accessory,
        response = false,
        ack = false,
        deprecated = false,
        transaction_id = required
    )]
    pub struct RegisterDescriptor {
        fields {
            /// Index
            // Virtual schema indices: 1
            required pub index: u8 => { bit: 0, key: "index", when: (truthy(&true)), predicate_value: false },
            /// Vendor ID
            // Virtual schema indices: 2
            required pub vendor_id: u16 => { bit: 1, key: "vendorID", when: (truthy(&true)), predicate_value: false },
            /// Product ID
            // Virtual schema indices: 3
            required pub product_id: u16 => { bit: 2, key: "productID", when: (truthy(&true)), predicate_value: false },
            /// Country Code
            // Virtual schema indices: 4
            required pub country_code: RegisterDescriptorCountryCode => { bit: 3, key: "countryCode", when: (truthy(&true)), predicate_value: false },
            /// HID Report Descriptor
            // Virtual schema indices: 5
            required pub descriptor: Vec<u8> => { bit: 4, key: "descriptor", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required index: u8 => { bit: 0, key: "index", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            2 => required vendor_id: u16 => { bit: 1, key: "vendorID", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            3 => required product_id: u16 => { bit: 2, key: "productID", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            4 => required country_code: RegisterDescriptorCountryCode => { bit: 3, key: "countryCode", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            5 => required descriptor: Vec<u8> => { bit: 4, key: "descriptor", wire: [raw_bytes], project_wire: Iap1WireSpec::Raw, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum IPodHIDReportReportType: u8 {
        /// input
        Input = 0,
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x02, command = 0x0010,
        source = accessory,
        response = false,
        ack = false,
        deprecated = false,
        transaction_id = required
    )]
    pub struct IPodHIDReport {
        fields {
            /// Index
            // Virtual schema indices: 1
            required pub index: u8 => { bit: 0, key: "index", when: (truthy(&true)), predicate_value: false },
            /// Report Type
            // Virtual schema indices: 2
            required pub report_type: IPodHIDReportReportType => { bit: 1, key: "reportType", when: (truthy(&true)), predicate_value: false },
            /// Report
            // Virtual schema indices: 3
            required pub report: Vec<u8> => { bit: 2, key: "report", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required index: u8 => { bit: 0, key: "index", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            2 => required report_type: IPodHIDReportReportType => { bit: 1, key: "reportType", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            3 => required report: Vec<u8> => { bit: 2, key: "report", wire: [raw_bytes], project_wire: Iap1WireSpec::Raw, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum AccessoryHIDReportReportType: u8 {
        /// output
        Output = 1,
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x02, command = 0x0011,
        source = device,
        response = false,
        ack = false,
        deprecated = false,
        transaction_id = required
    )]
    pub struct AccessoryHIDReport {
        fields {
            /// Index
            // Virtual schema indices: 1
            required pub index: u8 => { bit: 0, key: "index", when: (truthy(&true)), predicate_value: false },
            /// Report Type
            // Virtual schema indices: 2
            required pub report_type: AccessoryHIDReportReportType => { bit: 1, key: "reportType", when: (truthy(&true)), predicate_value: false },
            /// Report
            // Virtual schema indices: 3
            required pub report: Vec<u8> => { bit: 2, key: "report", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required index: u8 => { bit: 0, key: "index", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            2 => required report_type: AccessoryHIDReportReportType => { bit: 1, key: "reportType", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            3 => required report: Vec<u8> => { bit: 2, key: "report", wire: [raw_bytes], project_wire: Iap1WireSpec::Raw, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x02, command = 0x0012,
        source = accessory,
        response = false,
        ack = false,
        deprecated = false,
        transaction_id = required
    )]
    pub struct UnregisterDescriptor {
        fields {
            /// Index
            // Virtual schema indices: 1
            required pub index: u8 => { bit: 0, key: "index", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required index: u8 => { bit: 0, key: "index", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum VoiceOverEventEventType: u8 {
        /// move to point
        MoveToPoint = 0,
        /// move to first
        MoveToFirst = 1,
        /// move to last
        MoveToLast = 2,
        /// move to next
        MoveToNext = 3,
        /// move to previous
        MoveToPrevious = 4,
        /// scroll left page
        ScrollLeftPage = 5,
        /// scroll right page
        ScrollRightPage = 6,
        /// scroll up page
        ScrollUpPage = 7,
        /// scroll down page
        ScrollDownPage = 8,
        /// scroll to point
        ScrollToPoint = 9,
        /// text input
        TextInput = 10,
        /// cut
        Cut = 11,
        /// copy
        Copy = 12,
        /// paste
        Paste = 13,
        /// home
        Home = 14,
        /// touch
        Touch = 15,
        /// set display scale factor
        SetDisplayScaleFactor = 16,
        /// center display at point
        CenterDisplayAtPoint = 17,
        /// pause speaking
        PauseSpeaking = 18,
        /// resume speaking
        ResumeSpeaking = 19,
        /// read all text from current point
        ReadAllTextFromCurrentPoint = 20,
        /// read all text from top
        ReadAllTextFromTop = 21,
        /// speak text
        SpeakText = 22,
        /// escape
        Escape = 23,
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum VoiceOverEventTouchEventType: u8 {
        /// began
        Began = 0,
        /// moved
        Moved = 1,
        /// stationary
        Stationary = 2,
        /// ended
        Ended = 3,
        /// cancelled
        Cancelled = 4,
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x02, command = 0x0013,
        source = accessory,
        response = false,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct VoiceOverEvent {
        fields {
            /// Event Type
            // Virtual schema indices: 1
            required pub event_type: VoiceOverEventEventType => { bit: 0, key: "eventType", when: (truthy(&true)), predicate_value: true },
            /// X Coordinate
            // Virtual schema indices: 2, 7
            optional pub x_coordinate: u16 => { bit: 1, key: "xCoordinate", when: (((eq(&event_type, &0u64)) || (eq(&event_type, &9u64))) || (eq(&event_type, &17u64))) || (eq(&event_type, &15u64)), predicate_value: false },
            /// Y Coordinate
            // Virtual schema indices: 3, 8
            optional pub y_coordinate: u16 => { bit: 2, key: "yCoordinate", when: (((eq(&event_type, &0u64)) || (eq(&event_type, &9u64))) || (eq(&event_type, &17u64))) || (eq(&event_type, &15u64)), predicate_value: false },
            /// Current Section Index
            // Virtual schema indices: 4
            optional pub current_section_index: u16 => { bit: 3, key: "currentSectionIndex", when: ((eq(&event_type, &10u64)) || (eq(&event_type, &22u64))), predicate_value: false },
            /// Last Section Index
            // Virtual schema indices: 5
            optional pub last_section_index: u16 => { bit: 4, key: "lastSectionIndex", when: ((eq(&event_type, &10u64)) || (eq(&event_type, &22u64))), predicate_value: false },
            /// Text Data
            // Virtual schema indices: 6
            optional pub text_data: Vec<u8> => { bit: 5, key: "textData", when: ((eq(&event_type, &10u64)) || (eq(&event_type, &22u64))), predicate_value: false },
            /// Touch Event Type
            // Virtual schema indices: 9
            optional pub touch_event_type: VoiceOverEventTouchEventType => { bit: 6, key: "touchEventType", when: (eq(&event_type, &15u64)), predicate_value: false },
            /// Scale Factor
            // Virtual schema indices: 10
            optional pub scale_factor: u16 => { bit: 7, key: "scaleFactor", when: (eq(&event_type, &16u64)), predicate_value: false },
        }
        steps {
            1 => required event_type: VoiceOverEventEventType => { bit: 0, key: "eventType", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: true, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            2 => optional x_coordinate: u16 => { bit: 1, key: "xCoordinate", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: ((eq(&event_type, &0u64)) || (eq(&event_type, &9u64))) || (eq(&event_type, &17u64)), program: [FlatPredicateToken::Or, FlatPredicateToken::Or, FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(0u8), FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(9u8), FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(17u8)] } },
            3 => optional y_coordinate: u16 => { bit: 2, key: "yCoordinate", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: ((eq(&event_type, &0u64)) || (eq(&event_type, &9u64))) || (eq(&event_type, &17u64)), program: [FlatPredicateToken::Or, FlatPredicateToken::Or, FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(0u8), FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(9u8), FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(17u8)] } },
            4 => optional current_section_index: u16 => { bit: 3, key: "currentSectionIndex", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: (eq(&event_type, &10u64)) || (eq(&event_type, &22u64)), program: [FlatPredicateToken::Or, FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(10u8), FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(22u8)] } },
            5 => optional last_section_index: u16 => { bit: 4, key: "lastSectionIndex", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: (eq(&event_type, &10u64)) || (eq(&event_type, &22u64)), program: [FlatPredicateToken::Or, FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(10u8), FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(22u8)] } },
            6 => optional text_data: Vec<u8> => { bit: 5, key: "textData", wire: [raw_bytes], project_wire: Iap1WireSpec::Raw, predicate_value: false, predicate: { expr: (eq(&event_type, &10u64)) || (eq(&event_type, &22u64)), program: [FlatPredicateToken::Or, FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(10u8), FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(22u8)] } },
            7 => optional x_coordinate: u16 => { bit: 1, key: "xCoordinate", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&event_type, &15u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(15u8)] } },
            8 => optional y_coordinate: u16 => { bit: 2, key: "yCoordinate", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&event_type, &15u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(15u8)] } },
            9 => optional touch_event_type: VoiceOverEventTouchEventType => { bit: 6, key: "touchEventType", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&event_type, &15u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(15u8)] } },
            10 => optional scale_factor: u16 => { bit: 7, key: "scaleFactor", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&event_type, &16u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(16u8)] } },
        }
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum GetVoiceOverParameterParamType: u8 {
        /// VoiceOver volume
        VoiceOverVolume = 0,
        /// speaking rate
        SpeakingRate = 1,
        /// VoiceOver enabled
        VoiceOverEnabled = 2,
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x02, command = 0x0014,
        source = accessory,
        response = false,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct GetVoiceOverParameter {
        fields {
            /// Parameter Type
            // Virtual schema indices: 1
            required pub param_type: GetVoiceOverParameterParamType => { bit: 0, key: "paramType", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required param_type: GetVoiceOverParameterParamType => { bit: 0, key: "paramType", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum RetVoiceOverParameterParamType: u8 {
        /// VoiceOver volume
        VoiceOverVolume = 0,
        /// speaking rate
        SpeakingRate = 1,
        /// VoiceOver enabled
        VoiceOverEnabled = 2,
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x02, command = 0x0015,
        source = device,
        response = true,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct RetVoiceOverParameter {
        fields {
            /// Parameter Type
            // Virtual schema indices: 1
            required pub param_type: RetVoiceOverParameterParamType => { bit: 0, key: "paramType", when: (truthy(&true)), predicate_value: true },
            /// VoiceOver volume
            // Virtual schema indices: 2
            optional pub voice_over_volume: u8 => { bit: 1, key: "voiceOverVolume", when: (eq(&param_type, &0u64)), predicate_value: false },
            /// Speaking Rate
            // Virtual schema indices: 3
            optional pub speaking_rate: u8 => { bit: 2, key: "speakingRate", when: (eq(&param_type, &1u64)), predicate_value: false },
            /// VoiceOver enabled
            // Virtual schema indices: 4
            optional pub voice_over_enabled: bool => { bit: 3, key: "voiceOverEnabled", when: (eq(&param_type, &2u64)), predicate_value: false },
        }
        steps {
            1 => required param_type: RetVoiceOverParameterParamType => { bit: 0, key: "paramType", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: true, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            2 => optional voice_over_volume: u8 => { bit: 1, key: "voiceOverVolume", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&param_type, &0u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(0u8)] } },
            3 => optional speaking_rate: u8 => { bit: 2, key: "speakingRate", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&param_type, &1u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(1u8)] } },
            4 => optional voice_over_enabled: bool => { bit: 3, key: "voiceOverEnabled", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&param_type, &2u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(2u8)] } },
        }
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum SetVoiceOverParameterParamType: u8 {
        /// VoiceOver volume
        VoiceOverVolume = 0,
        /// speaking rate
        SpeakingRate = 1,
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x02, command = 0x0016,
        source = accessory,
        response = false,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct SetVoiceOverParameter {
        fields {
            /// Parameter Type
            // Virtual schema indices: 1
            required pub param_type: SetVoiceOverParameterParamType => { bit: 0, key: "paramType", when: (truthy(&true)), predicate_value: true },
            /// VoiceOver volume
            // Virtual schema indices: 2
            optional pub voice_over_volume: u8 => { bit: 1, key: "voiceOverVolume", when: (eq(&param_type, &0u64)), predicate_value: false },
            /// Speaking Rate
            // Virtual schema indices: 3
            optional pub speaking_rate: u8 => { bit: 2, key: "speakingRate", when: (eq(&param_type, &1u64)), predicate_value: false },
        }
        steps {
            1 => required param_type: SetVoiceOverParameterParamType => { bit: 0, key: "paramType", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: true, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            2 => optional voice_over_volume: u8 => { bit: 1, key: "voiceOverVolume", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&param_type, &0u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(0u8)] } },
            3 => optional speaking_rate: u8 => { bit: 2, key: "speakingRate", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&param_type, &1u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(1u8)] } },
        }
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum GetCurrentVoiceOverItemPropertyPropertyType: u8 {
        /// label
        Label = 0,
        /// value
        Value = 1,
        /// hint
        Hint = 2,
        /// frame
        Frame = 3,
        /// traits
        Traits = 4,
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x02, command = 0x0017,
        source = accessory,
        response = false,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct GetCurrentVoiceOverItemProperty {
        fields {
            /// Property Type
            // Virtual schema indices: 1
            required pub property_type: GetCurrentVoiceOverItemPropertyPropertyType => { bit: 0, key: "propertyType", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required property_type: GetCurrentVoiceOverItemPropertyPropertyType => { bit: 0, key: "propertyType", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum RetCurrentVoiceOverItemPropertyPropertyType: u8 {
        /// label
        Label = 0,
        /// value
        Value = 1,
        /// hint
        Hint = 2,
        /// frame
        Frame = 3,
        /// traits
        Traits = 4,
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum RetCurrentVoiceOverItemPropertyTraitsRecordsTraitsData: u16 {
        /// Button
        Button = 0,
        /// Link
        Link = 1,
        /// Search Field
        SearchField = 2,
        /// Image
        Image = 3,
        /// Selected
        Selected = 4,
        /// Sound
        Sound = 5,
        /// Keyboard Key
        KeyboardKey = 6,
        /// Static Text
        StaticText = 7,
        /// Summary Element
        SummaryElement = 8,
        /// Not Enabled
        NotEnabled = 9,
        /// Updates Frequently
        UpdatesFrequently = 10,
        /// Starts Media Session
        StartsMediaSession = 11,
        /// Adjustable
        Adjustable = 12,
        /// Back Button
        BackButton = 13,
        /// Map
        Map = 14,
        /// Delete Key
        DeleteKey = 15,
    }
}

iap1_record! {
    strings = LingoString;
    pub struct RetCurrentVoiceOverItemPropertyTraitsRecords {
        fields {
            /// Trait
            // Virtual schema indices: 1
            required pub traits_data: RetCurrentVoiceOverItemPropertyTraitsRecordsTraitsData => { bit: 0, key: "traitsData", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required traits_data: RetCurrentVoiceOverItemPropertyTraitsRecordsTraitsData => { bit: 0, key: "traitsData", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x02, command = 0x0018,
        source = device,
        response = true,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct RetCurrentVoiceOverItemProperty {
        fields {
            /// Property Type
            // Virtual schema indices: 1
            required pub property_type: RetCurrentVoiceOverItemPropertyPropertyType => { bit: 0, key: "propertyType", when: (truthy(&true)), predicate_value: true },
            /// Current Section Index
            // Virtual schema indices: 2
            optional pub current_section_index: u16 => { bit: 1, key: "currentSectionIndex", when: (((eq(&property_type, &0u64)) || (eq(&property_type, &1u64))) || (eq(&property_type, &2u64))), predicate_value: false },
            /// Last Section Index
            // Virtual schema indices: 3
            optional pub last_section_index: u16 => { bit: 2, key: "lastSectionIndex", when: (((eq(&property_type, &0u64)) || (eq(&property_type, &1u64))) || (eq(&property_type, &2u64))), predicate_value: false },
            /// Text Data
            // Virtual schema indices: 4
            optional pub text_data: Vec<u8> => { bit: 3, key: "textData", when: (((eq(&property_type, &0u64)) || (eq(&property_type, &1u64))) || (eq(&property_type, &2u64))), predicate_value: false },
            /// Top Left X Coordinate
            // Virtual schema indices: 5
            optional pub top_left_x_coordinate: u16 => { bit: 4, key: "topLeftXCoordinate", when: (eq(&property_type, &3u64)), predicate_value: false },
            /// Top Left Y Coordinate
            // Virtual schema indices: 6
            optional pub top_left_y_coordinate: u16 => { bit: 5, key: "topLeftYCoordinate", when: (eq(&property_type, &3u64)), predicate_value: false },
            /// Bottom Right X Coordinate
            // Virtual schema indices: 7
            optional pub bottom_right_x_coordinate: u16 => { bit: 6, key: "bottomRightXCoordinate", when: (eq(&property_type, &3u64)), predicate_value: false },
            /// Bottom Right Y Coordinate
            // Virtual schema indices: 8
            optional pub bottom_right_y_coordinate: u16 => { bit: 7, key: "bottomRightYCoordinate", when: (eq(&property_type, &3u64)), predicate_value: false },
            /// Traits
            // Virtual schema indices: 9
            optional pub traits_records: Vec<RetCurrentVoiceOverItemPropertyTraitsRecords> => { bit: 8, key: "traitsRecords", when: (eq(&property_type, &4u64)), predicate_value: false },
        }
        steps {
            1 => required property_type: RetCurrentVoiceOverItemPropertyPropertyType => { bit: 0, key: "propertyType", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: true, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            2 => optional current_section_index: u16 => { bit: 1, key: "currentSectionIndex", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: ((eq(&property_type, &0u64)) || (eq(&property_type, &1u64))) || (eq(&property_type, &2u64)), program: [FlatPredicateToken::Or, FlatPredicateToken::Or, FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(0u8), FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(1u8), FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(2u8)] } },
            3 => optional last_section_index: u16 => { bit: 2, key: "lastSectionIndex", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: ((eq(&property_type, &0u64)) || (eq(&property_type, &1u64))) || (eq(&property_type, &2u64)), program: [FlatPredicateToken::Or, FlatPredicateToken::Or, FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(0u8), FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(1u8), FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(2u8)] } },
            4 => optional text_data: Vec<u8> => { bit: 3, key: "textData", wire: [raw_bytes], project_wire: Iap1WireSpec::Raw, predicate_value: false, predicate: { expr: ((eq(&property_type, &0u64)) || (eq(&property_type, &1u64))) || (eq(&property_type, &2u64)), program: [FlatPredicateToken::Or, FlatPredicateToken::Or, FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(0u8), FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(1u8), FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(2u8)] } },
            5 => optional top_left_x_coordinate: u16 => { bit: 4, key: "topLeftXCoordinate", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&property_type, &3u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(3u8)] } },
            6 => optional top_left_y_coordinate: u16 => { bit: 5, key: "topLeftYCoordinate", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&property_type, &3u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(3u8)] } },
            7 => optional bottom_right_x_coordinate: u16 => { bit: 6, key: "bottomRightXCoordinate", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&property_type, &3u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(3u8)] } },
            8 => optional bottom_right_y_coordinate: u16 => { bit: 7, key: "bottomRightYCoordinate", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&property_type, &3u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(3u8)] } },
            9 => optional traits_records: Vec<RetCurrentVoiceOverItemPropertyTraitsRecords> => { bit: 8, key: "traitsRecords", wire: [counted_remaining], project_wire: Iap1WireSpec::CountedRemaining, predicate_value: false, predicate: { expr: eq(&property_type, &4u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(4u8)] } },
        }
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum SetVoiceOverContextUserInterfaceContext: u8 {
        /// none
        None = 0,
        /// header
        Header = 1,
        /// link
        Link = 2,
        /// form
        Form = 3,
        /// cursor
        Cursor = 4,
        /// vertical navigation
        VerticalNavigation = 5,
        /// value adjustment
        ValueAdjustment = 6,
        /// zoom adjustment
        ZoomAdjustment = 7,
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x02, command = 0x0019,
        source = accessory,
        response = false,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct SetVoiceOverContext {
        fields {
            /// User Interface Context
            // Virtual schema indices: 1
            required pub user_interface_context: SetVoiceOverContextUserInterfaceContext => { bit: 0, key: "userInterfaceContext", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required user_interface_context: SetVoiceOverContextUserInterfaceContext => { bit: 0, key: "userInterfaceContext", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum VoiceOverParameterChangedParamType: u8 {
        /// VoiceOver volume
        VoiceOverVolume = 0,
        /// speaking rate
        SpeakingRate = 1,
        /// VoiceOver enabled
        VoiceOverEnabled = 2,
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x02, command = 0x001a,
        source = device,
        response = false,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct VoiceOverParameterChanged {
        fields {
            /// Parameter Type
            // Virtual schema indices: 1
            required pub param_type: VoiceOverParameterChangedParamType => { bit: 0, key: "paramType", when: (truthy(&true)), predicate_value: true },
            /// VoiceOver volume
            // Virtual schema indices: 2
            optional pub voice_over_volume: u8 => { bit: 1, key: "voiceOverVolume", when: (eq(&param_type, &0u64)), predicate_value: false },
            /// Speaking Rate
            // Virtual schema indices: 3
            optional pub speaking_rate: u8 => { bit: 2, key: "speakingRate", when: (eq(&param_type, &1u64)), predicate_value: false },
            /// VoiceOver enabled
            // Virtual schema indices: 4
            optional pub voice_over_enabled: bool => { bit: 3, key: "voiceOverEnabled", when: (eq(&param_type, &2u64)), predicate_value: false },
        }
        steps {
            1 => required param_type: VoiceOverParameterChangedParamType => { bit: 0, key: "paramType", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: true, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            2 => optional voice_over_volume: u8 => { bit: 1, key: "voiceOverVolume", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&param_type, &0u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(0u8)] } },
            3 => optional speaking_rate: u8 => { bit: 2, key: "speakingRate", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&param_type, &1u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(1u8)] } },
            4 => optional voice_over_enabled: bool => { bit: 3, key: "voiceOverEnabled", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&param_type, &2u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(2u8)] } },
        }
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum AccessoryAckCommandResult: u8 {
        /// OK
        OK = 0,
        /// failed
        Failed = 2,
        /// bad parameter
        BadParameter = 4,
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x02, command = 0x0081,
        source = accessory,
        response = true,
        ack = false,
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

iap1_registry! {
    0x02;
    ContextButtonStatus => 0x0000,
    IPodAck => 0x0001,
    ImageButtonStatus => 0x0002,
    VideoButtonStatus => 0x0003,
    AudioButtonStatus => 0x0004,
    IPodOutButtonStatus => 0x000b,
    RotationInputStatus => 0x000c,
    RadioButtonStatus => 0x000d,
    CameraButtonStatus => 0x000e,
    RegisterDescriptor => 0x000f,
    IPodHIDReport => 0x0010,
    AccessoryHIDReport => 0x0011,
    UnregisterDescriptor => 0x0012,
    VoiceOverEvent => 0x0013,
    GetVoiceOverParameter => 0x0014,
    RetVoiceOverParameter => 0x0015,
    SetVoiceOverParameter => 0x0016,
    GetCurrentVoiceOverItemProperty => 0x0017,
    RetCurrentVoiceOverItemProperty => 0x0018,
    SetVoiceOverContext => 0x0019,
    VoiceOverParameterChanged => 0x001a,
    AccessoryAck => 0x0081,
}
