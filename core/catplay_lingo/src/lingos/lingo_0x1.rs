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

// Lingo 0x01: Microphone [deprecated]
iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x01, command = 0x0000,
        source = device,
        response = false,
        ack = false,
        deprecated = true,
        transaction_id = permitted
    )]
    pub struct BeginRecord {
        fields {
        }
        steps {
        }
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x01, command = 0x0001,
        source = device,
        response = false,
        ack = false,
        deprecated = true,
        transaction_id = permitted
    )]
    pub struct EndRecord {
        fields {
        }
        steps {
        }
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x01, command = 0x0002,
        source = device,
        response = false,
        ack = false,
        deprecated = true,
        transaction_id = permitted
    )]
    pub struct BeginPlayback {
        fields {
        }
        steps {
        }
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x01, command = 0x0003,
        source = device,
        response = false,
        ack = false,
        deprecated = true,
        transaction_id = permitted
    )]
    pub struct EndPlayback {
        fields {
        }
        steps {
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
        /// out of resources
        OutOfResources = 3,
        /// bad parameter
        BadParameter = 4,
        /// not authenticated
        NotAuthenticated = 7,
        /// bad authentication version
        BadAuthenticationVersion = 8,
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x01, command = 0x0004,
        source = accessory,
        response = true,
        ack = true,
        deprecated = true,
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

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x01, command = 0x0005,
        source = device,
        response = false,
        ack = false,
        deprecated = true,
        transaction_id = permitted
    )]
    pub struct GetAccessoryAck {
        fields {
        }
        steps {
        }
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum IPodModeChangeMode: u8 {
        /// begin audio recording
        BeginAudioRecording = 0,
        /// end audio recording
        EndAudioRecording = 1,
        /// begin audio playback
        BeginAudioPlayback = 2,
        /// end audio playback
        EndAudioPlayback = 3,
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x01, command = 0x0006,
        source = device,
        response = false,
        ack = false,
        deprecated = true,
        transaction_id = permitted
    )]
    pub struct IPodModeChange {
        fields {
            /// Mode
            // Virtual schema indices: 1
            required pub mode: IPodModeChangeMode => { bit: 0, key: "mode", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required mode: IPodModeChangeMode => { bit: 0, key: "mode", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x01, command = 0x0007,
        source = device,
        response = false,
        ack = false,
        deprecated = true,
        transaction_id = permitted
    )]
    pub struct GetAccessoryCaps {
        fields {
        }
        steps {
        }
    }
}

iap1_bitflags! {
    strings = LingoString;
    pub struct RetAccessoryCapsDeviceCaps: u32 {
        /// stereo line input
        const STEREO_LINE_INPUT = 1 << 0;
        /// stereo or mono line input
        const STEREO_OR_MONO_LINE_INPUT = 1 << 1;
        /// recording level is present and variable
        const RECORDING_LEVEL_IS_PRESENT_AND_VARIABLE = 1 << 2;
        /// recording level limit is present
        const RECORDING_LEVEL_LIMIT_IS_PRESENT = 1 << 3;
        /// duplex audio
        const DUPLEX_AUDIO = 1 << 4;
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x01, command = 0x0008,
        source = accessory,
        response = true,
        ack = false,
        deprecated = true,
        transaction_id = permitted
    )]
    pub struct RetAccessoryCaps {
        fields {
            /// Device Capabilities
            // Virtual schema indices: 1
            required pub device_caps: RetAccessoryCapsDeviceCaps => { bit: 0, key: "deviceCaps", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required device_caps: RetAccessoryCapsDeviceCaps => { bit: 0, key: "deviceCaps", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum GetDevCtrlControlType: u8 {
        /// stereo/mono line input
        StereoMonoLineInput = 1,
        /// recording level control
        RecordingLevelControl = 2,
        /// recording level limiter control
        RecordingLevelLimiterControl = 3,
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x01, command = 0x0009,
        source = device,
        response = false,
        ack = false,
        deprecated = true,
        transaction_id = permitted
    )]
    pub struct GetDevCtrl {
        fields {
            /// Control Type
            // Virtual schema indices: 1
            required pub control_type: GetDevCtrlControlType => { bit: 0, key: "controlType", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required control_type: GetDevCtrlControlType => { bit: 0, key: "controlType", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum RetDevCtrlControlType: u8 {
        /// stereo/mono line input
        StereoMonoLineInput = 1,
        /// recording level control
        RecordingLevelControl = 2,
        /// recording level limiter control
        RecordingLevelLimiterControl = 3,
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum RetDevCtrlLineInputType: u8 {
        /// mono
        Mono = 0,
        /// stereo
        Stereo = 1,
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x01, command = 0x000a,
        source = accessory,
        response = true,
        ack = false,
        deprecated = true,
        transaction_id = permitted
    )]
    pub struct RetDevCtrl {
        fields {
            /// Control Type
            // Virtual schema indices: 1
            required pub control_type: RetDevCtrlControlType => { bit: 0, key: "controlType", when: (truthy(&true)), predicate_value: true },
            /// Line Input Type
            // Virtual schema indices: 2
            optional pub line_input_type: RetDevCtrlLineInputType => { bit: 1, key: "lineInputType", when: (eq(&control_type, &1u64)), predicate_value: false },
            /// Recording Level Gain
            // Virtual schema indices: 3
            optional pub recording_level_gain: u8 => { bit: 2, key: "recordingLevelGain", when: (eq(&control_type, &2u64)), predicate_value: false },
            /// Recording Level Limiter
            // Virtual schema indices: 4
            optional pub recording_level_limiter: bool => { bit: 3, key: "recordingLevelLimiter", when: (eq(&control_type, &3u64)), predicate_value: false },
        }
        steps {
            1 => required control_type: RetDevCtrlControlType => { bit: 0, key: "controlType", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: true, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            2 => optional line_input_type: RetDevCtrlLineInputType => { bit: 1, key: "lineInputType", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&control_type, &1u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(1u8)] } },
            3 => optional recording_level_gain: u8 => { bit: 2, key: "recordingLevelGain", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&control_type, &2u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(2u8)] } },
            4 => optional recording_level_limiter: bool => { bit: 3, key: "recordingLevelLimiter", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&control_type, &3u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(3u8)] } },
        }
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum SetDevCtrlControlType: u8 {
        /// stereo/mono line input
        StereoMonoLineInput = 1,
        /// recording level control
        RecordingLevelControl = 2,
        /// recording level limiter control
        RecordingLevelLimiterControl = 3,
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum SetDevCtrlLineInputType: u8 {
        /// mono
        Mono = 0,
        /// stereo
        Stereo = 1,
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x01, command = 0x000b,
        source = device,
        response = false,
        ack = false,
        deprecated = true,
        transaction_id = permitted
    )]
    pub struct SetDevCtrl {
        fields {
            /// Control Type
            // Virtual schema indices: 1
            required pub control_type: SetDevCtrlControlType => { bit: 0, key: "controlType", when: (truthy(&true)), predicate_value: true },
            /// Line Input Type
            // Virtual schema indices: 2
            optional pub line_input_type: SetDevCtrlLineInputType => { bit: 1, key: "lineInputType", when: (eq(&control_type, &1u64)), predicate_value: false },
            /// Recording Level Gain
            // Virtual schema indices: 3
            optional pub recording_level_gain: u8 => { bit: 2, key: "recordingLevelGain", when: (eq(&control_type, &2u64)), predicate_value: false },
            /// Recording Level Limiter
            // Virtual schema indices: 4
            optional pub recording_level_limiter: bool => { bit: 3, key: "recordingLevelLimiter", when: (eq(&control_type, &3u64)), predicate_value: false },
        }
        steps {
            1 => required control_type: SetDevCtrlControlType => { bit: 0, key: "controlType", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: true, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            2 => optional line_input_type: SetDevCtrlLineInputType => { bit: 1, key: "lineInputType", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&control_type, &1u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(1u8)] } },
            3 => optional recording_level_gain: u8 => { bit: 2, key: "recordingLevelGain", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&control_type, &2u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(2u8)] } },
            4 => optional recording_level_limiter: bool => { bit: 3, key: "recordingLevelLimiter", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&control_type, &3u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(3u8)] } },
        }
    }
}

iap1_registry! {
    0x01;
    BeginRecord => 0x0000,
    EndRecord => 0x0001,
    BeginPlayback => 0x0002,
    EndPlayback => 0x0003,
    AccessoryAck => 0x0004,
    GetAccessoryAck => 0x0005,
    IPodModeChange => 0x0006,
    GetAccessoryCaps => 0x0007,
    RetAccessoryCaps => 0x0008,
    GetDevCtrl => 0x0009,
    RetDevCtrl => 0x000a,
    SetDevCtrl => 0x000b,
}
