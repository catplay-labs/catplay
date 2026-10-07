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

// Lingo 0x00: General
iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x00, command = 0x0000,
        source = device,
        response = false,
        ack = false,
        deprecated = false,
        transaction_id = prohibited
    )]
    pub struct RequestIdentify {
        fields {
        }
        steps {
        }
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x00, command = 0x0001,
        source = accessory,
        response = false,
        ack = false,
        deprecated = true,
        transaction_id = prohibited
    )]
    pub struct Identify {
        fields {
            /// Supported Lingo
            // Virtual schema indices: 1
            required pub supported_lingo: u8 => { bit: 0, key: "supportedLingo", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required supported_lingo: u8 => { bit: 0, key: "supportedLingo", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
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
        context = ctx, strings = LingoString, lingo = 0x00, command = 0x0002,
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
        context = ctx, strings = LingoString, lingo = 0x00, command = 0x0003,
        source = accessory,
        response = false,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct RequestExtendedInterfaceMode {
        fields {
        }
        steps {
        }
    }
}

iap1_enum_open! {
    strings = LingoString;
    #[iap1_enum_open(storage = u8, ranges = [1..=255])]
    /// Permitted range 1..=255: Extended Interface mode
    pub enum ReturnExtendedInterfaceModeMode {
        /// Standard UI mode
        StandardUIMode = 0,
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x00, command = 0x0004,
        source = device,
        response = true,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct ReturnExtendedInterfaceMode {
        fields {
            /// Mode
            // Virtual schema indices: 1
            required pub mode: ReturnExtendedInterfaceModeMode => { bit: 0, key: "mode", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required mode: ReturnExtendedInterfaceModeMode => { bit: 0, key: "mode", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x00, command = 0x0005,
        source = accessory,
        response = false,
        ack = false,
        deprecated = true,
        transaction_id = permitted
    )]
    pub struct EnterExtendedInterfaceMode {
        fields {
        }
        steps {
        }
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x00, command = 0x0006,
        source = accessory,
        response = false,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct ExitExtendedInterfaceMode {
        fields {
        }
        steps {
        }
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x00, command = 0x0007,
        source = accessory,
        response = false,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct RequestiPodName {
        fields {
        }
        steps {
        }
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x00, command = 0x0008,
        source = device,
        response = true,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct ReturniPodName {
        fields {
            /// iPod Name
            // Virtual schema indices: 1
            required pub i_pod_name: String => { bit: 0, key: "iPodName", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required i_pod_name: String => { bit: 0, key: "iPodName", wire: [raw_string], project_wire: Iap1WireSpec::Raw, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x00, command = 0x0009,
        source = accessory,
        response = false,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct RequestiPodSoftwareVersion {
        fields {
        }
        steps {
        }
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x00, command = 0x000a,
        source = device,
        response = true,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct ReturniPodSoftwareVersion {
        fields {
            /// iPod Major Version Number
            // Virtual schema indices: 1
            required pub i_pod_major_version_number: u8 => { bit: 0, key: "iPodMajorVersionNumber", when: (truthy(&true)), predicate_value: false },
            /// iPod Minor Version Number
            // Virtual schema indices: 2
            required pub i_pod_minor_version_number: u8 => { bit: 1, key: "iPodMinorVersionNumber", when: (truthy(&true)), predicate_value: false },
            /// iPod Revision Version Number
            // Virtual schema indices: 3
            required pub i_pod_revision_version_number: u8 => { bit: 2, key: "iPodRevisionVersionNumber", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required i_pod_major_version_number: u8 => { bit: 0, key: "iPodMajorVersionNumber", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            2 => required i_pod_minor_version_number: u8 => { bit: 1, key: "iPodMinorVersionNumber", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            3 => required i_pod_revision_version_number: u8 => { bit: 2, key: "iPodRevisionVersionNumber", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x00, command = 0x000b,
        source = accessory,
        response = false,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct RequestiPodSerialNum {
        fields {
        }
        steps {
        }
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x00, command = 0x000c,
        source = device,
        response = true,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct ReturniPodSerialNum {
        fields {
            /// iPod Serial Number
            // Virtual schema indices: 1
            required pub i_pod_serial_number: String => { bit: 0, key: "iPodSerialNumber", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required i_pod_serial_number: String => { bit: 0, key: "iPodSerialNumber", wire: [raw_string], project_wire: Iap1WireSpec::Raw, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x00, command = 0x000d,
        source = accessory,
        response = false,
        ack = false,
        deprecated = true,
        transaction_id = permitted
    )]
    pub struct RequestiPodModelNum {
        fields {
        }
        steps {
        }
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x00, command = 0x000e,
        source = device,
        response = true,
        ack = false,
        deprecated = true,
        transaction_id = permitted
    )]
    pub struct ReturniPodModelNum {
        fields {
            /// iPod Model ID
            // Virtual schema indices: 1
            required pub i_pod_model_id: u32 => { bit: 0, key: "iPodModelID", when: (truthy(&true)), predicate_value: false },
            /// iPod Model Number
            // Virtual schema indices: 2
            required pub i_pod_model_number: String => { bit: 1, key: "iPodModelNumber", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required i_pod_model_id: u32 => { bit: 0, key: "iPodModelID", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            2 => required i_pod_model_number: String => { bit: 1, key: "iPodModelNumber", wire: [raw_string], project_wire: Iap1WireSpec::Raw, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x00, command = 0x000f,
        source = accessory,
        response = false,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct RequestLingoProtocolVersion {
        fields {
            /// Lingo Requested
            // Virtual schema indices: 1
            required pub lingo_requested: u8 => { bit: 0, key: "lingoRequested", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required lingo_requested: u8 => { bit: 0, key: "lingoRequested", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x00, command = 0x0010,
        source = device,
        response = true,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct ReturnLingoProtocolVersion {
        fields {
            /// Lingo Returned
            // Virtual schema indices: 1
            required pub lingo_returned: u8 => { bit: 0, key: "lingoReturned", when: (truthy(&true)), predicate_value: false },
            /// Major Protocol Version
            // Virtual schema indices: 2
            required pub lingo_major_protocol_version: u8 => { bit: 1, key: "lingoMajorProtocolVersion", when: (truthy(&true)), predicate_value: false },
            /// Minor Protocol Version
            // Virtual schema indices: 3
            required pub lingo_minor_protocol_version: u8 => { bit: 2, key: "lingoMinorProtocolVersion", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required lingo_returned: u8 => { bit: 0, key: "lingoReturned", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            2 => required lingo_major_protocol_version: u8 => { bit: 1, key: "lingoMajorProtocolVersion", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            3 => required lingo_minor_protocol_version: u8 => { bit: 2, key: "lingoMinorProtocolVersion", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x00, command = 0x0011,
        source = accessory,
        response = false,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct RequestTransportMaxPayloadSize {
        fields {
        }
        steps {
        }
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x00, command = 0x0012,
        source = device,
        response = true,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct ReturnTransportMaxPayloadSize {
        fields {
            /// Maximum Payload Size
            // Virtual schema indices: 1
            required pub max_payload: u16 => { bit: 0, key: "maxPayload", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required max_payload: u16 => { bit: 0, key: "maxPayload", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1_bitflags! {
    strings = LingoString;
    pub struct IdentifyDeviceLingoesDeviceLingoesSpoken: u32 {
        /// General
        /// Required by the specification.
        const GENERAL = 1 << 0;
        /// Microphone
        const MICROPHONE = 1 << 1;
        /// Simple Remote
        const SIMPLE_REMOTE = 1 << 2;
        /// Display Remote
        const DISPLAY_REMOTE = 1 << 3;
        /// Extended Interface
        const EXTENDED_INTERFACE = 1 << 4;
        /// Accessory Power
        const ACCESSORY_POWER = 1 << 5;
        /// USB Host Control
        const USB_HOST_CONTROL = 1 << 6;
        /// RF Tuner
        const RF_TUNER = 1 << 7;
        /// Accessory Equalizer
        const ACCESSORY_EQUALIZER = 1 << 8;
        /// Sports
        const SPORTS = 1 << 9;
        /// Digital Audio
        const DIGITAL_AUDIO = 1 << 10;
        /// Storage
        const STORAGE = 1 << 12;
    }
}

iap1_bitfield! {
    pub struct IdentifyDeviceLingoesOptions: u32 {
        #[group(mask = 0x3, shift = 0)]
        /// authentication control bits
        pub authentication_control_bits: IdentifyDeviceLingoesOptionsAuthenticationControlBits {
            /// no authentication
            NoAuthentication = 0,
            /// defer authentication
            DeferAuthentication = 1,
            /// immediate authentication
            ImmediateAuthentication = 2,
        },
        #[group(mask = 0xc, shift = 2)]
        /// power control bits
        pub power_control_bits: IdentifyDeviceLingoesOptionsPowerControlBits {
            /// low power
            LowPower = 0,
            /// intermittent high power
            #[deprecated]
            IntermittentHighPower = 1,
            /// constant high power
            #[deprecated]
            ConstantHighPower = 3,
        },
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x00, command = 0x0013,
        source = accessory,
        response = false,
        ack = false,
        deprecated = false,
        transaction_id = prohibited
    )]
    pub struct IdentifyDeviceLingoes {
        fields {
            /// Device Lingoes Spoken
            // Virtual schema indices: 1
            required pub device_lingoes_spoken: IdentifyDeviceLingoesDeviceLingoesSpoken => { bit: 0, key: "deviceLingoesSpoken", when: (truthy(&true)), predicate_value: false },
            /// Options
            // Virtual schema indices: 2
            required pub options: IdentifyDeviceLingoesOptions => { bit: 1, key: "options", when: (truthy(&true)), predicate_value: false },
            /// Device ID
            // Virtual schema indices: 3
            required pub device_id: u32 => { bit: 2, key: "deviceID", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required device_lingoes_spoken: IdentifyDeviceLingoesDeviceLingoesSpoken => { bit: 0, key: "deviceLingoesSpoken", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            2 => required options: IdentifyDeviceLingoesOptions => { bit: 1, key: "options", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            3 => required device_id: u32 => { bit: 2, key: "deviceID", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x00, command = 0x0014,
        source = device,
        response = false,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct GetAccessoryAuthenticationInfo {
        fields {
        }
        steps {
        }
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x00, command = 0x0015,
        source = accessory,
        response = true,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct RetAccessoryAuthenticationInfo {
        fields {
            /// Authentication Protocol Major Version Number
            // Virtual schema indices: 1
            required pub authentication_protocol_major_version_number: u8 => { bit: 0, key: "authenticationProtocolMajorVersionNumber", when: (truthy(&true)), predicate_value: true },
            /// Authentication Protocol Minor Version Number
            // Virtual schema indices: 2
            required pub authentication_protocol_minor_version_number: u8 => { bit: 1, key: "authenticationProtocolMinorVersionNumber", when: (truthy(&true)), predicate_value: true },
            /// X.509 Certificate Current Section Index
            // Virtual schema indices: 3
            optional pub certificate_current_section_index: u8 => { bit: 2, key: "certificateCurrentSectionIndex", when: ((eq(&authentication_protocol_major_version_number, &2u64)) && (eq(&authentication_protocol_minor_version_number, &0u64))), predicate_value: false },
            /// X.509 Certificate Maximum Section Index
            // Virtual schema indices: 4
            optional pub certificate_maximum_section_index: u8 => { bit: 3, key: "certificateMaximumSectionIndex", when: ((eq(&authentication_protocol_major_version_number, &2u64)) && (eq(&authentication_protocol_minor_version_number, &0u64))), predicate_value: false },
            /// X.509 Certificate Data
            // Virtual schema indices: 5
            optional pub certificate_data: Vec<u8> => { bit: 4, key: "certificateData", when: ((eq(&authentication_protocol_major_version_number, &2u64)) && (eq(&authentication_protocol_minor_version_number, &0u64))), predicate_value: false },
        }
        steps {
            1 => required authentication_protocol_major_version_number: u8 => { bit: 0, key: "authenticationProtocolMajorVersionNumber", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: true, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            2 => required authentication_protocol_minor_version_number: u8 => { bit: 1, key: "authenticationProtocolMinorVersionNumber", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: true, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            3 => optional certificate_current_section_index: u8 => { bit: 2, key: "certificateCurrentSectionIndex", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: (eq(&authentication_protocol_major_version_number, &2u64)) && (eq(&authentication_protocol_minor_version_number, &0u64)), program: [FlatPredicateToken::And, FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(2u8), FlatPredicateToken::Eq, FlatPredicateToken::Field(1), FlatPredicateToken::Integer(0u8)] } },
            4 => optional certificate_maximum_section_index: u8 => { bit: 3, key: "certificateMaximumSectionIndex", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: (eq(&authentication_protocol_major_version_number, &2u64)) && (eq(&authentication_protocol_minor_version_number, &0u64)), program: [FlatPredicateToken::And, FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(2u8), FlatPredicateToken::Eq, FlatPredicateToken::Field(1), FlatPredicateToken::Integer(0u8)] } },
            5 => optional certificate_data: Vec<u8> => { bit: 4, key: "certificateData", wire: [raw_bytes], project_wire: Iap1WireSpec::Raw, predicate_value: false, predicate: { expr: (eq(&authentication_protocol_major_version_number, &2u64)) && (eq(&authentication_protocol_minor_version_number, &0u64)), program: [FlatPredicateToken::And, FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(2u8), FlatPredicateToken::Eq, FlatPredicateToken::Field(1), FlatPredicateToken::Integer(0u8)] } },
        }
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum AckAccessoryAuthenticationInfoAuthenticationStatus: u8 {
        /// authentication info supported
        AuthenticationInfoSupported = 0,
        /// certificate too long or sections out of order
        CertificateTooLongOrSectionsOutOfOrder = 4,
        /// authentication info unsupported
        AuthenticationInfoUnsupported = 8,
        /// certificate is invalid
        CertificateIsInvalid = 10,
        /// certificate permissions are invalid
        CertificatePermissionsAreInvalid = 11,
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x00, command = 0x0016,
        source = device,
        response = false,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct AckAccessoryAuthenticationInfo {
        fields {
            /// Authentication Info Status
            // Virtual schema indices: 1
            required pub authentication_status: AckAccessoryAuthenticationInfoAuthenticationStatus => { bit: 0, key: "authenticationStatus", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required authentication_status: AckAccessoryAuthenticationInfoAuthenticationStatus => { bit: 0, key: "authenticationStatus", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1_disjoint! {
    strings = LingoString;
    #[iap1_disjoint(predicate = opaque)]
    pub enum GetAccessoryAuthenticationSignatureChallengeDisjoint {
        #[iap1_disjoint(decode = decode_value16_byte_challenge, encode = encode_value16_byte_challenge)]
        /// 16-Byte Challenge
        Value16ByteChallenge([u8; 16]),
        #[iap1_disjoint(decode = decode_value20_byte_challenge, encode = encode_value20_byte_challenge)]
        /// 20-Byte Challenge
        Value20ByteChallenge([u8; 20]),
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x00, command = 0x0017,
        source = device,
        response = false,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct GetAccessoryAuthenticationSignature {
        fields {
            /// 16-Byte Challenge
            // Virtual schema indices: 1, 3
            length_optional pub challenge: GetAccessoryAuthenticationSignatureChallengeDisjoint => { bit: 0, key: "challenge", when: (ctx.encoding() || (eq(&ctx.adjusted_payload_length, &17u64))) || (ctx.encoding() || (eq(&ctx.adjusted_payload_length, &21u64))), predicate_value: false },
            /// Authentication Retry Counter
            // Virtual schema indices: 2, 4
            length_optional pub authentication_retry_counter: u8 => { bit: 1, key: "authenticationRetryCounter", when: (ctx.encoding() || (eq(&ctx.adjusted_payload_length, &17u64))) || (ctx.encoding() || (eq(&ctx.adjusted_payload_length, &21u64))), predicate_value: false },
        }
        steps {
            1 => length_optional challenge: GetAccessoryAuthenticationSignatureChallengeDisjoint => { bit: 0, key: "challenge", wire: [disjoint(decode_value16_byte_challenge, encode_value16_byte_challenge, [u8; 16], [scalar])], project_wire: Iap1WireSpec::Disjoint { variant: const { LingoString::require_raw(stringify!(decode_value16_byte_challenge)) }, child: &Iap1WireSpec::Scalar }, predicate_value: false, predicate: { expr: ctx.encoding() || (eq(&ctx.adjusted_payload_length, &17u64)), program: [FlatPredicateToken::Or, FlatPredicateToken::Encoding, FlatPredicateToken::Eq, FlatPredicateToken::AdjustedPayloadLength, FlatPredicateToken::Integer(17u8)] } },
            2 => length_optional authentication_retry_counter: u8 => { bit: 1, key: "authenticationRetryCounter", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: ctx.encoding() || (eq(&ctx.adjusted_payload_length, &17u64)), program: [FlatPredicateToken::Or, FlatPredicateToken::Encoding, FlatPredicateToken::Eq, FlatPredicateToken::AdjustedPayloadLength, FlatPredicateToken::Integer(17u8)] } },
            3 => length_optional challenge: GetAccessoryAuthenticationSignatureChallengeDisjoint => { bit: 0, key: "challenge", wire: [disjoint(decode_value20_byte_challenge, encode_value20_byte_challenge, [u8; 20], [scalar])], project_wire: Iap1WireSpec::Disjoint { variant: const { LingoString::require_raw(stringify!(decode_value20_byte_challenge)) }, child: &Iap1WireSpec::Scalar }, predicate_value: false, predicate: { expr: ctx.encoding() || (eq(&ctx.adjusted_payload_length, &21u64)), program: [FlatPredicateToken::Or, FlatPredicateToken::Encoding, FlatPredicateToken::Eq, FlatPredicateToken::AdjustedPayloadLength, FlatPredicateToken::Integer(21u8)] } },
            4 => length_optional authentication_retry_counter: u8 => { bit: 1, key: "authenticationRetryCounter", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: ctx.encoding() || (eq(&ctx.adjusted_payload_length, &21u64)), program: [FlatPredicateToken::Or, FlatPredicateToken::Encoding, FlatPredicateToken::Eq, FlatPredicateToken::AdjustedPayloadLength, FlatPredicateToken::Integer(21u8)] } },
        }
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x00, command = 0x0018,
        source = accessory,
        response = true,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct RetAccessoryAuthenticationSignature {
        fields {
            /// Calculated Signature
            // Virtual schema indices: 1
            required pub calculated_signature: Vec<u8> => { bit: 0, key: "calculatedSignature", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required calculated_signature: Vec<u8> => { bit: 0, key: "calculatedSignature", wire: [raw_bytes], project_wire: Iap1WireSpec::Raw, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1_enum_open! {
    strings = LingoString;
    #[iap1_enum_open(storage = u8, ranges = [1..=255])]
    /// Permitted range 1..=255: failed
    pub enum AckAccessoryAuthenticationStatusStatus {
        /// passed
        Passed = 0,
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x00, command = 0x0019,
        source = device,
        response = false,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct AckAccessoryAuthenticationStatus {
        fields {
            /// Authentication Operation Status
            // Virtual schema indices: 1
            required pub status: AckAccessoryAuthenticationStatusStatus => { bit: 0, key: "status", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required status: AckAccessoryAuthenticationStatusStatus => { bit: 0, key: "status", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x00, command = 0x001a,
        source = accessory,
        response = false,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct GetiPodAuthenticationInfo {
        fields {
        }
        steps {
        }
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x00, command = 0x001b,
        source = device,
        response = true,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct RetiPodAuthenticationInfo {
        fields {
            /// Authentication Protocol Major Version Number
            // Virtual schema indices: 1
            required pub authentication_protocol_major_version_number: u8 => { bit: 0, key: "authenticationProtocolMajorVersionNumber", when: (truthy(&true)), predicate_value: false },
            /// Authentication Protocol Minor Version Number
            // Virtual schema indices: 2
            required pub authentication_protocol_minor_version_number: u8 => { bit: 1, key: "authenticationProtocolMinorVersionNumber", when: (truthy(&true)), predicate_value: false },
            /// X.509 Certificate Current Section Index
            // Virtual schema indices: 3
            required pub certificate_current_section_index: u8 => { bit: 2, key: "certificateCurrentSectionIndex", when: (truthy(&true)), predicate_value: false },
            /// X.509 Certificate Maximum Section Index
            // Virtual schema indices: 4
            required pub certificate_maximum_section_index: u8 => { bit: 3, key: "certificateMaximumSectionIndex", when: (truthy(&true)), predicate_value: false },
            /// X.509 Certificate Data
            // Virtual schema indices: 5
            required pub certificate_data: Vec<u8> => { bit: 4, key: "certificateData", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required authentication_protocol_major_version_number: u8 => { bit: 0, key: "authenticationProtocolMajorVersionNumber", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            2 => required authentication_protocol_minor_version_number: u8 => { bit: 1, key: "authenticationProtocolMinorVersionNumber", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            3 => required certificate_current_section_index: u8 => { bit: 2, key: "certificateCurrentSectionIndex", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            4 => required certificate_maximum_section_index: u8 => { bit: 3, key: "certificateMaximumSectionIndex", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            5 => required certificate_data: Vec<u8> => { bit: 4, key: "certificateData", wire: [raw_bytes], project_wire: Iap1WireSpec::Raw, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1_enum_open! {
    strings = LingoString;
    #[iap1_enum_open(storage = u8, ranges = [1..=255])]
    /// Permitted range 1..=255: authentication information not valid
    pub enum AckiPodAuthenticationInfoStatus {
        /// authentication information valid
        AuthenticationInformationValid = 0,
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x00, command = 0x001c,
        source = accessory,
        response = false,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct AckiPodAuthenticationInfo {
        fields {
            /// Authentication Information Status
            // Virtual schema indices: 1
            required pub status: AckiPodAuthenticationInfoStatus => { bit: 0, key: "status", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required status: AckiPodAuthenticationInfoStatus => { bit: 0, key: "status", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x00, command = 0x001d,
        source = accessory,
        response = false,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct GetiPodAuthenticationSignature {
        fields {
            /// 20-Byte Challenge
            // Virtual schema indices: 1
            required pub challenge: [u8; 20] => { bit: 0, key: "challenge", when: (truthy(&true)), predicate_value: false },
            /// Authentication Retry Counter
            // Virtual schema indices: 2
            required pub authentication_retry_counter: u8 => { bit: 1, key: "authenticationRetryCounter", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required challenge: [u8; 20] => { bit: 0, key: "challenge", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            2 => required authentication_retry_counter: u8 => { bit: 1, key: "authenticationRetryCounter", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x00, command = 0x001e,
        source = device,
        response = true,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct RetiPodAuthenticationSignature {
        fields {
            /// Calculated Signature
            // Virtual schema indices: 1
            required pub calculated_signature: Vec<u8> => { bit: 0, key: "calculatedSignature", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required calculated_signature: Vec<u8> => { bit: 0, key: "calculatedSignature", wire: [raw_bytes], project_wire: Iap1WireSpec::Raw, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1_enum_open! {
    strings = LingoString;
    #[iap1_enum_open(storage = u8, ranges = [1..=255])]
    /// Permitted range 1..=255: failed
    pub enum AckiPodAuthenticationStatusStatus {
        /// passed
        Passed = 0,
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x00, command = 0x001f,
        source = accessory,
        response = false,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct AckiPodAuthenticationStatus {
        fields {
            /// Authentication Operation Status
            // Virtual schema indices: 1
            required pub status: AckiPodAuthenticationStatusStatus => { bit: 0, key: "status", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required status: AckiPodAuthenticationStatusStatus => { bit: 0, key: "status", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum NotifyiPodStateChangeStateChange: u8 {
        /// hibernate without preserving menu selections or playback context
        HibernateWithoutPreservingMenuSelectionsOrPlaybackContext = 1,
        /// hibernate but preserving menu selections and playback context
        HibernateButPreservingMenuSelectionsAndPlaybackContext = 2,
        /// sleep
        Sleep = 3,
        /// power on
        PowerOn = 4,
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x00, command = 0x0023,
        source = device,
        response = false,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct NotifyiPodStateChange {
        fields {
            /// iPod State
            // Virtual schema indices: 1
            required pub state_change: NotifyiPodStateChangeStateChange => { bit: 0, key: "stateChange", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required state_change: NotifyiPodStateChangeStateChange => { bit: 0, key: "stateChange", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x00, command = 0x0024,
        source = accessory,
        response = false,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct GetiPodOptions {
        fields {
        }
        steps {
        }
    }
}

iap1_bitflags! {
    strings = LingoString;
    pub struct RetiPodOptionsOptionBits: u64 {
        /// supports video output
        const SUPPORTS_VIDEO_OUTPUT = 1 << 0;
        /// supports line out control
        const SUPPORTS_LINE_OUT_CONTROL = 1 << 1;
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x00, command = 0x0025,
        source = device,
        response = true,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct RetiPodOptions {
        fields {
            /// Option Bits
            // Virtual schema indices: 1
            required pub option_bits: RetiPodOptionsOptionBits => { bit: 0, key: "optionBits", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required option_bits: RetiPodOptionsOptionBits => { bit: 0, key: "optionBits", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum GetAccessoryInfoAccessoryInfoType: u8 {
        /// info capabilities
        InfoCapabilities = 0,
        /// name
        Name = 1,
        /// minimum supported iPod firmware version
        MinimumSupportedIPodFirmwareVersion = 2,
        /// minimum supported lingo version
        MinimumSupportedLingoVersion = 3,
        /// firmware version
        FirmwareVersion = 4,
        /// hardware version
        HardwareVersion = 5,
        /// manufacturer
        Manufacturer = 6,
        /// model number
        ModelNumber = 7,
        /// serial number
        SerialNumber = 8,
        /// incoming maximum payload size
        IncomingMaximumPayloadSize = 9,
        /// status types
        StatusTypes = 11,
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x00, command = 0x0027,
        source = device,
        response = false,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct GetAccessoryInfo {
        fields {
            /// Accessory Info Type
            // Virtual schema indices: 1
            required pub accessory_info_type: GetAccessoryInfoAccessoryInfoType => { bit: 0, key: "accessoryInfoType", when: (truthy(&true)), predicate_value: true },
            /// iPod Model ID
            // Virtual schema indices: 2
            optional pub i_pod_model_id: u32 => { bit: 1, key: "iPodModelID", when: (eq(&accessory_info_type, &2u64)), predicate_value: false },
            /// iPod Firmware Major Version Number
            // Virtual schema indices: 3
            optional pub i_pod_firmware_major_version: u8 => { bit: 2, key: "iPodFirmwareMajorVersion", when: (eq(&accessory_info_type, &2u64)), predicate_value: false },
            /// iPod Firmware Minor Version Number
            // Virtual schema indices: 4
            optional pub i_pod_firmware_minor_version: u8 => { bit: 3, key: "iPodFirmwareMinorVersion", when: (eq(&accessory_info_type, &2u64)), predicate_value: false },
            /// iPod Firmware Revision Version Number
            // Virtual schema indices: 5
            optional pub i_pod_firmware_revision_version: u8 => { bit: 4, key: "iPodFirmwareRevisionVersion", when: (eq(&accessory_info_type, &2u64)), predicate_value: false },
            /// Lingo ID
            // Virtual schema indices: 6
            optional pub lingo_id: u8 => { bit: 5, key: "lingoID", when: (eq(&accessory_info_type, &3u64)), predicate_value: false },
        }
        steps {
            1 => required accessory_info_type: GetAccessoryInfoAccessoryInfoType => { bit: 0, key: "accessoryInfoType", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: true, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            2 => optional i_pod_model_id: u32 => { bit: 1, key: "iPodModelID", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&accessory_info_type, &2u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(2u8)] } },
            3 => optional i_pod_firmware_major_version: u8 => { bit: 2, key: "iPodFirmwareMajorVersion", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&accessory_info_type, &2u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(2u8)] } },
            4 => optional i_pod_firmware_minor_version: u8 => { bit: 3, key: "iPodFirmwareMinorVersion", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&accessory_info_type, &2u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(2u8)] } },
            5 => optional i_pod_firmware_revision_version: u8 => { bit: 4, key: "iPodFirmwareRevisionVersion", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&accessory_info_type, &2u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(2u8)] } },
            6 => optional lingo_id: u8 => { bit: 5, key: "lingoID", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&accessory_info_type, &3u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(3u8)] } },
        }
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum RetAccessoryInfoAccessoryInfoType: u8 {
        /// info capabilities
        InfoCapabilities = 0,
        /// name
        Name = 1,
        /// minimum supported iPod firmware version
        MinimumSupportedIPodFirmwareVersion = 2,
        /// minimum supported lingo version
        MinimumSupportedLingoVersion = 3,
        /// firmware version
        FirmwareVersion = 4,
        /// hardware version
        HardwareVersion = 5,
        /// manufacturer
        Manufacturer = 6,
        /// model number
        ModelNumber = 7,
        /// serial number
        SerialNumber = 8,
        /// incoming maximum payload size
        IncomingMaximumPayloadSize = 9,
        /// status types
        StatusTypes = 11,
    }
}

iap1_bitflags! {
    strings = LingoString;
    pub struct RetAccessoryInfoAccessoryInfoCapabilities: u32 {
        /// info capabilities
        /// Required by the specification.
        const INFO_CAPABILITIES = 1 << 0;
        /// name
        /// Required by the specification.
        const NAME = 1 << 1;
        /// minimum supported iPod firmware version
        const MINIMUM_SUPPORTED_I_POD_FIRMWARE_VERSION = 1 << 2;
        /// minimum supported lingo version
        const MINIMUM_SUPPORTED_LINGO_VERSION = 1 << 3;
        /// firmware version
        /// Required by the specification.
        const FIRMWARE_VERSION = 1 << 4;
        /// hardware version
        /// Required by the specification.
        const HARDWARE_VERSION = 1 << 5;
        /// manufacturer
        /// Required by the specification.
        const MANUFACTURER = 1 << 6;
        /// model number
        /// Required by the specification.
        const MODEL_NUMBER = 1 << 7;
        /// serial number
        const SERIAL_NUMBER = 1 << 8;
        /// incoming maximum payload size
        const INCOMING_MAXIMUM_PAYLOAD_SIZE = 1 << 9;
        /// status notifications
        const STATUS_NOTIFICATIONS = 1 << 11;
        /// asynchronous playback state changes
        const ASYNCHRONOUS_PLAYBACK_STATE_CHANGES = 1 << 18;
    }
}

iap1_bitflags! {
    strings = LingoString;
    pub struct RetAccessoryInfoStatusTypesSupported: u32 {
        /// Bluetooth device
        const BLUETOOTH_DEVICE = 1 << 1;
        /// fault condition
        const FAULT_CONDITION = 1 << 2;
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x00, command = 0x0028,
        source = accessory,
        response = true,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct RetAccessoryInfo {
        fields {
            /// Accessory Info Type
            // Virtual schema indices: 1
            required pub accessory_info_type: RetAccessoryInfoAccessoryInfoType => { bit: 0, key: "accessoryInfoType", when: (truthy(&true)), predicate_value: true },
            /// Accessory Info Capabilities
            // Virtual schema indices: 2
            optional pub accessory_info_capabilities: RetAccessoryInfoAccessoryInfoCapabilities => { bit: 1, key: "accessoryInfoCapabilities", when: (eq(&accessory_info_type, &0u64)), predicate_value: false },
            /// Name
            // Virtual schema indices: 3
            optional pub accessory_name: String => { bit: 2, key: "accessoryName", when: (eq(&accessory_info_type, &1u64)), predicate_value: false },
            /// iPod Model ID
            // Virtual schema indices: 4
            optional pub i_pod_model_id: u32 => { bit: 3, key: "iPodModelID", when: (eq(&accessory_info_type, &2u64)), predicate_value: false },
            /// Minimum Supported iPod Firmware Major Version
            // Virtual schema indices: 5
            optional pub i_pod_firmware_major_version: u8 => { bit: 4, key: "iPodFirmwareMajorVersion", when: (eq(&accessory_info_type, &2u64)), predicate_value: false },
            /// Minimum Supported iPod Firmware Minor Version
            // Virtual schema indices: 6
            optional pub i_pod_firmware_minor_version: u8 => { bit: 5, key: "iPodFirmwareMinorVersion", when: (eq(&accessory_info_type, &2u64)), predicate_value: false },
            /// Minimum Supported iPod Firmware Revision Version
            // Virtual schema indices: 7
            optional pub i_pod_firmware_revision_version: u8 => { bit: 6, key: "iPodFirmwareRevisionVersion", when: (eq(&accessory_info_type, &2u64)), predicate_value: false },
            /// Lingo ID
            // Virtual schema indices: 8
            optional pub lingo_id: u8 => { bit: 7, key: "lingoID", when: (eq(&accessory_info_type, &3u64)), predicate_value: false },
            /// Major Protocol Version
            // Virtual schema indices: 9
            optional pub major_protocol_version: u8 => { bit: 8, key: "majorProtocolVersion", when: (eq(&accessory_info_type, &3u64)), predicate_value: false },
            /// Minor Protocol Version
            // Virtual schema indices: 10
            optional pub minor_protocol_version: u8 => { bit: 9, key: "minorProtocolVersion", when: (eq(&accessory_info_type, &3u64)), predicate_value: false },
            /// Accessory Major Version Number
            // Virtual schema indices: 11
            optional pub accessory_major_version: u8 => { bit: 10, key: "accessoryMajorVersion", when: ((eq(&accessory_info_type, &4u64)) || (eq(&accessory_info_type, &5u64))), predicate_value: false },
            /// Accessory Minor Version Number
            // Virtual schema indices: 12
            optional pub accessory_minor_version: u8 => { bit: 11, key: "accessoryMinorVersion", when: ((eq(&accessory_info_type, &4u64)) || (eq(&accessory_info_type, &5u64))), predicate_value: false },
            /// Accessory Revision Version Number
            // Virtual schema indices: 13
            optional pub accessory_revision_version: u8 => { bit: 12, key: "accessoryRevisionVersion", when: ((eq(&accessory_info_type, &4u64)) || (eq(&accessory_info_type, &5u64))), predicate_value: false },
            /// Manufacturer
            // Virtual schema indices: 14
            optional pub accessory_manufacturer: String => { bit: 13, key: "accessoryManufacturer", when: (eq(&accessory_info_type, &6u64)), predicate_value: false },
            /// Model Number
            // Virtual schema indices: 15
            optional pub accessory_model_number: String => { bit: 14, key: "accessoryModelNumber", when: (eq(&accessory_info_type, &7u64)), predicate_value: false },
            /// Serial Number
            // Virtual schema indices: 16
            optional pub accessory_serial_number: String => { bit: 15, key: "accessorySerialNumber", when: (eq(&accessory_info_type, &8u64)), predicate_value: false },
            /// Maximum Incoming Payload Size
            // Virtual schema indices: 17
            optional pub max_payload_size: u16 => { bit: 16, key: "maxPayloadSize", when: (eq(&accessory_info_type, &9u64)), predicate_value: false },
            /// Status Types Supported
            // Virtual schema indices: 18
            optional pub status_types_supported: RetAccessoryInfoStatusTypesSupported => { bit: 17, key: "statusTypesSupported", when: (eq(&accessory_info_type, &11u64)), predicate_value: false },
        }
        steps {
            1 => required accessory_info_type: RetAccessoryInfoAccessoryInfoType => { bit: 0, key: "accessoryInfoType", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: true, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            2 => optional accessory_info_capabilities: RetAccessoryInfoAccessoryInfoCapabilities => { bit: 1, key: "accessoryInfoCapabilities", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&accessory_info_type, &0u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(0u8)] } },
            3 => optional accessory_name: String => { bit: 2, key: "accessoryName", wire: [raw_string], project_wire: Iap1WireSpec::Raw, predicate_value: false, predicate: { expr: eq(&accessory_info_type, &1u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(1u8)] } },
            4 => optional i_pod_model_id: u32 => { bit: 3, key: "iPodModelID", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&accessory_info_type, &2u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(2u8)] } },
            5 => optional i_pod_firmware_major_version: u8 => { bit: 4, key: "iPodFirmwareMajorVersion", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&accessory_info_type, &2u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(2u8)] } },
            6 => optional i_pod_firmware_minor_version: u8 => { bit: 5, key: "iPodFirmwareMinorVersion", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&accessory_info_type, &2u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(2u8)] } },
            7 => optional i_pod_firmware_revision_version: u8 => { bit: 6, key: "iPodFirmwareRevisionVersion", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&accessory_info_type, &2u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(2u8)] } },
            8 => optional lingo_id: u8 => { bit: 7, key: "lingoID", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&accessory_info_type, &3u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(3u8)] } },
            9 => optional major_protocol_version: u8 => { bit: 8, key: "majorProtocolVersion", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&accessory_info_type, &3u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(3u8)] } },
            10 => optional minor_protocol_version: u8 => { bit: 9, key: "minorProtocolVersion", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&accessory_info_type, &3u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(3u8)] } },
            11 => optional accessory_major_version: u8 => { bit: 10, key: "accessoryMajorVersion", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: (eq(&accessory_info_type, &4u64)) || (eq(&accessory_info_type, &5u64)), program: [FlatPredicateToken::Or, FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(4u8), FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(5u8)] } },
            12 => optional accessory_minor_version: u8 => { bit: 11, key: "accessoryMinorVersion", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: (eq(&accessory_info_type, &4u64)) || (eq(&accessory_info_type, &5u64)), program: [FlatPredicateToken::Or, FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(4u8), FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(5u8)] } },
            13 => optional accessory_revision_version: u8 => { bit: 12, key: "accessoryRevisionVersion", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: (eq(&accessory_info_type, &4u64)) || (eq(&accessory_info_type, &5u64)), program: [FlatPredicateToken::Or, FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(4u8), FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(5u8)] } },
            14 => optional accessory_manufacturer: String => { bit: 13, key: "accessoryManufacturer", wire: [raw_string], project_wire: Iap1WireSpec::Raw, predicate_value: false, predicate: { expr: eq(&accessory_info_type, &6u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(6u8)] } },
            15 => optional accessory_model_number: String => { bit: 14, key: "accessoryModelNumber", wire: [raw_string], project_wire: Iap1WireSpec::Raw, predicate_value: false, predicate: { expr: eq(&accessory_info_type, &7u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(7u8)] } },
            16 => optional accessory_serial_number: String => { bit: 15, key: "accessorySerialNumber", wire: [raw_string], project_wire: Iap1WireSpec::Raw, predicate_value: false, predicate: { expr: eq(&accessory_info_type, &8u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(8u8)] } },
            17 => optional max_payload_size: u16 => { bit: 16, key: "maxPayloadSize", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&accessory_info_type, &9u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(9u8)] } },
            18 => optional status_types_supported: RetAccessoryInfoStatusTypesSupported => { bit: 17, key: "statusTypesSupported", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&accessory_info_type, &11u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(11u8)] } },
        }
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum GetiPodPreferencesPreferenceClassID: u8 {
        /// video out setting
        VideoOutSetting = 0,
        /// screen configuration
        ScreenConfiguration = 1,
        /// video signal format
        VideoSignalFormat = 2,
        /// line-out usage
        LineOutUsage = 3,
        /// video-out connection
        VideoOutConnection = 8,
        /// closed captioning
        ClosedCaptioning = 9,
        /// video monitor aspect ratio
        VideoMonitorAspectRatio = 10,
        /// subtitles
        Subtitles = 12,
        /// video alternate audio channel
        VideoAlternateAudioChannel = 13,
        /// pause on power removal
        PauseOnPowerRemoval = 15,
        /// VoiceOver
        VoiceOver = 20,
        /// AssistiveTouch
        AssistiveTouch = 22,
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x00, command = 0x0029,
        source = accessory,
        response = false,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct GetiPodPreferences {
        fields {
            /// Preference Class
            // Virtual schema indices: 1
            required pub preference_class_id: GetiPodPreferencesPreferenceClassID => { bit: 0, key: "preferenceClassID", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required preference_class_id: GetiPodPreferencesPreferenceClassID => { bit: 0, key: "preferenceClassID", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum RetiPodPreferencesPreferenceClassID: u8 {
        /// video out setting
        VideoOutSetting = 0,
        /// screen configuration
        ScreenConfiguration = 1,
        /// video signal format
        VideoSignalFormat = 2,
        /// line-out usage
        LineOutUsage = 3,
        /// video-out connection
        VideoOutConnection = 8,
        /// closed captioning
        ClosedCaptioning = 9,
        /// video monitor aspect ratio
        VideoMonitorAspectRatio = 10,
        /// subtitles
        Subtitles = 12,
        /// video alternate audio channel
        VideoAlternateAudioChannel = 13,
        /// pause on power removal
        PauseOnPowerRemoval = 15,
        /// VoiceOver
        VoiceOver = 20,
        /// AssistiveTouch
        AssistiveTouch = 22,
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum RetiPodPreferencesVideoOutSetting: u8 {
        /// off
        Off = 0,
        /// on
        On = 1,
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum RetiPodPreferencesScreenConfiguration: u8 {
        /// fill entire screen
        FillEntireScreen = 0,
        /// fit to screen edge
        FitToScreenEdge = 1,
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum RetiPodPreferencesVideoSignalFormat: u8 {
        /// NTSC
        NTSC = 0,
        /// PAL
        PAL = 1,
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum RetiPodPreferencesLineOutUsage: u8 {
        /// disabled
        Disabled = 0,
        /// enabled
        Enabled = 1,
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum RetiPodPreferencesVideoOutConnection: u8 {
        /// no change
        NoChange = 0,
        /// composite
        Composite = 1,
        /// S-Video
        SVideo = 2,
        /// component
        Component = 3,
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum RetiPodPreferencesClosedCaptioning: u8 {
        /// disabled
        Disabled = 0,
        /// enabled
        Enabled = 1,
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum RetiPodPreferencesVideoMonitorAspectRatio: u8 {
        /// 4:3 aspect ratio (fullscreen)
        Value43AspectRatioFullscreen = 0,
        /// 16:9 aspect ratio (widescreen)
        Value169AspectRatioWidescreen = 1,
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum RetiPodPreferencesVoiceOver: u8 {
        /// VoiceOver off
        VoiceOverOff = 0,
        /// VoiceOver on
        VoiceOverOn = 1,
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum RetiPodPreferencesAssistiveTouch: u8 {
        /// disabled
        Disabled = 0,
        /// enabled
        Enabled = 1,
    }
}

iap1_disjoint! {
    strings = LingoString;
    #[iap1_disjoint(predicate = integer)]
    pub enum RetiPodPreferencesPreferenceSettingIDDisjoint {
        #[iap1_disjoint(decode = decode_video_out_setting, encode = encode_video_out_setting)]
        /// Video Out Setting
        VideoOutSetting(RetiPodPreferencesVideoOutSetting),
        #[iap1_disjoint(decode = decode_screen_configuration, encode = encode_screen_configuration)]
        /// Screen Configuration
        ScreenConfiguration(RetiPodPreferencesScreenConfiguration),
        #[iap1_disjoint(decode = decode_video_signal_format, encode = encode_video_signal_format)]
        /// Video Signal Format
        VideoSignalFormat(RetiPodPreferencesVideoSignalFormat),
        #[iap1_disjoint(decode = decode_line_out_usage, encode = encode_line_out_usage)]
        /// Line Out Usage
        LineOutUsage(RetiPodPreferencesLineOutUsage),
        #[iap1_disjoint(decode = decode_video_out_connection, encode = encode_video_out_connection)]
        /// Video Out Connection
        VideoOutConnection(RetiPodPreferencesVideoOutConnection),
        #[iap1_disjoint(decode = decode_closed_captioning, encode = encode_closed_captioning)]
        /// Closed Captioning
        ClosedCaptioning(RetiPodPreferencesClosedCaptioning),
        #[iap1_disjoint(decode = decode_video_monitor_aspect_ratio, encode = encode_video_monitor_aspect_ratio)]
        /// Video Monitor Aspect Ratio
        VideoMonitorAspectRatio(RetiPodPreferencesVideoMonitorAspectRatio),
        #[iap1_disjoint(decode = decode_subtitles, encode = encode_subtitles)]
        /// Subtitles
        Subtitles(bool),
        #[iap1_disjoint(decode = decode_video_alternate_audio_channel, encode = encode_video_alternate_audio_channel)]
        /// Video Alternate Audio Channel
        VideoAlternateAudioChannel(bool),
        #[iap1_disjoint(decode = decode_pause_on_power_removal, encode = encode_pause_on_power_removal)]
        /// Pause On Power Removal
        PauseOnPowerRemoval(bool),
        #[iap1_disjoint(decode = decode_voice_over, encode = encode_voice_over)]
        /// VoiceOver
        VoiceOver(RetiPodPreferencesVoiceOver),
        #[iap1_disjoint(decode = decode_assistive_touch, encode = encode_assistive_touch)]
        /// AssistiveTouch
        AssistiveTouch(RetiPodPreferencesAssistiveTouch),
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x00, command = 0x002a,
        source = device,
        response = true,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct RetiPodPreferences {
        fields {
            /// Preference Class
            // Virtual schema indices: 1
            required pub preference_class_id: RetiPodPreferencesPreferenceClassID => { bit: 0, key: "preferenceClassID", when: (truthy(&true)), predicate_value: true },
            /// Video Out Setting
            // Virtual schema indices: 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13
            optional pub preference_setting_id: RetiPodPreferencesPreferenceSettingIDDisjoint => { bit: 1, key: "preferenceSettingID", when: (eq(&preference_class_id, &0u64)) || (eq(&preference_class_id, &1u64)) || (eq(&preference_class_id, &2u64)) || (eq(&preference_class_id, &3u64)) || (eq(&preference_class_id, &8u64)) || (eq(&preference_class_id, &9u64)) || (eq(&preference_class_id, &10u64)) || (eq(&preference_class_id, &12u64)) || (eq(&preference_class_id, &13u64)) || (eq(&preference_class_id, &15u64)) || (eq(&preference_class_id, &20u64)) || (eq(&preference_class_id, &22u64)), predicate_value: false },
        }
        steps {
            1 => required preference_class_id: RetiPodPreferencesPreferenceClassID => { bit: 0, key: "preferenceClassID", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: true, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            2 => optional preference_setting_id: RetiPodPreferencesPreferenceSettingIDDisjoint => { bit: 1, key: "preferenceSettingID", wire: [disjoint(decode_video_out_setting, encode_video_out_setting, RetiPodPreferencesVideoOutSetting, [scalar])], project_wire: Iap1WireSpec::Disjoint { variant: const { LingoString::require_raw(stringify!(decode_video_out_setting)) }, child: &Iap1WireSpec::Scalar }, predicate_value: false, predicate: { expr: eq(&preference_class_id, &0u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(0u8)] } },
            3 => optional preference_setting_id: RetiPodPreferencesPreferenceSettingIDDisjoint => { bit: 1, key: "preferenceSettingID", wire: [disjoint(decode_screen_configuration, encode_screen_configuration, RetiPodPreferencesScreenConfiguration, [scalar])], project_wire: Iap1WireSpec::Disjoint { variant: const { LingoString::require_raw(stringify!(decode_screen_configuration)) }, child: &Iap1WireSpec::Scalar }, predicate_value: false, predicate: { expr: eq(&preference_class_id, &1u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(1u8)] } },
            4 => optional preference_setting_id: RetiPodPreferencesPreferenceSettingIDDisjoint => { bit: 1, key: "preferenceSettingID", wire: [disjoint(decode_video_signal_format, encode_video_signal_format, RetiPodPreferencesVideoSignalFormat, [scalar])], project_wire: Iap1WireSpec::Disjoint { variant: const { LingoString::require_raw(stringify!(decode_video_signal_format)) }, child: &Iap1WireSpec::Scalar }, predicate_value: false, predicate: { expr: eq(&preference_class_id, &2u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(2u8)] } },
            5 => optional preference_setting_id: RetiPodPreferencesPreferenceSettingIDDisjoint => { bit: 1, key: "preferenceSettingID", wire: [disjoint(decode_line_out_usage, encode_line_out_usage, RetiPodPreferencesLineOutUsage, [scalar])], project_wire: Iap1WireSpec::Disjoint { variant: const { LingoString::require_raw(stringify!(decode_line_out_usage)) }, child: &Iap1WireSpec::Scalar }, predicate_value: false, predicate: { expr: eq(&preference_class_id, &3u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(3u8)] } },
            6 => optional preference_setting_id: RetiPodPreferencesPreferenceSettingIDDisjoint => { bit: 1, key: "preferenceSettingID", wire: [disjoint(decode_video_out_connection, encode_video_out_connection, RetiPodPreferencesVideoOutConnection, [scalar])], project_wire: Iap1WireSpec::Disjoint { variant: const { LingoString::require_raw(stringify!(decode_video_out_connection)) }, child: &Iap1WireSpec::Scalar }, predicate_value: false, predicate: { expr: eq(&preference_class_id, &8u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(8u8)] } },
            7 => optional preference_setting_id: RetiPodPreferencesPreferenceSettingIDDisjoint => { bit: 1, key: "preferenceSettingID", wire: [disjoint(decode_closed_captioning, encode_closed_captioning, RetiPodPreferencesClosedCaptioning, [scalar])], project_wire: Iap1WireSpec::Disjoint { variant: const { LingoString::require_raw(stringify!(decode_closed_captioning)) }, child: &Iap1WireSpec::Scalar }, predicate_value: false, predicate: { expr: eq(&preference_class_id, &9u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(9u8)] } },
            8 => optional preference_setting_id: RetiPodPreferencesPreferenceSettingIDDisjoint => { bit: 1, key: "preferenceSettingID", wire: [disjoint(decode_video_monitor_aspect_ratio, encode_video_monitor_aspect_ratio, RetiPodPreferencesVideoMonitorAspectRatio, [scalar])], project_wire: Iap1WireSpec::Disjoint { variant: const { LingoString::require_raw(stringify!(decode_video_monitor_aspect_ratio)) }, child: &Iap1WireSpec::Scalar }, predicate_value: false, predicate: { expr: eq(&preference_class_id, &10u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(10u8)] } },
            9 => optional preference_setting_id: RetiPodPreferencesPreferenceSettingIDDisjoint => { bit: 1, key: "preferenceSettingID", wire: [disjoint(decode_subtitles, encode_subtitles, bool, [scalar])], project_wire: Iap1WireSpec::Disjoint { variant: const { LingoString::require_raw(stringify!(decode_subtitles)) }, child: &Iap1WireSpec::Scalar }, predicate_value: false, predicate: { expr: eq(&preference_class_id, &12u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(12u8)] } },
            10 => optional preference_setting_id: RetiPodPreferencesPreferenceSettingIDDisjoint => { bit: 1, key: "preferenceSettingID", wire: [disjoint(decode_video_alternate_audio_channel, encode_video_alternate_audio_channel, bool, [scalar])], project_wire: Iap1WireSpec::Disjoint { variant: const { LingoString::require_raw(stringify!(decode_video_alternate_audio_channel)) }, child: &Iap1WireSpec::Scalar }, predicate_value: false, predicate: { expr: eq(&preference_class_id, &13u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(13u8)] } },
            11 => optional preference_setting_id: RetiPodPreferencesPreferenceSettingIDDisjoint => { bit: 1, key: "preferenceSettingID", wire: [disjoint(decode_pause_on_power_removal, encode_pause_on_power_removal, bool, [scalar])], project_wire: Iap1WireSpec::Disjoint { variant: const { LingoString::require_raw(stringify!(decode_pause_on_power_removal)) }, child: &Iap1WireSpec::Scalar }, predicate_value: false, predicate: { expr: eq(&preference_class_id, &15u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(15u8)] } },
            12 => optional preference_setting_id: RetiPodPreferencesPreferenceSettingIDDisjoint => { bit: 1, key: "preferenceSettingID", wire: [disjoint(decode_voice_over, encode_voice_over, RetiPodPreferencesVoiceOver, [scalar])], project_wire: Iap1WireSpec::Disjoint { variant: const { LingoString::require_raw(stringify!(decode_voice_over)) }, child: &Iap1WireSpec::Scalar }, predicate_value: false, predicate: { expr: eq(&preference_class_id, &20u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(20u8)] } },
            13 => optional preference_setting_id: RetiPodPreferencesPreferenceSettingIDDisjoint => { bit: 1, key: "preferenceSettingID", wire: [disjoint(decode_assistive_touch, encode_assistive_touch, RetiPodPreferencesAssistiveTouch, [scalar])], project_wire: Iap1WireSpec::Disjoint { variant: const { LingoString::require_raw(stringify!(decode_assistive_touch)) }, child: &Iap1WireSpec::Scalar }, predicate_value: false, predicate: { expr: eq(&preference_class_id, &22u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(22u8)] } },
        }
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum SetiPodPreferencesPreferenceClassID: u8 {
        /// video out setting
        VideoOutSetting = 0,
        /// screen configuration
        ScreenConfiguration = 1,
        /// video signal format
        VideoSignalFormat = 2,
        /// line-out usage
        LineOutUsage = 3,
        /// video-out connection
        VideoOutConnection = 8,
        /// closed captioning
        ClosedCaptioning = 9,
        /// video monitor aspect ratio
        VideoMonitorAspectRatio = 10,
        /// subtitles
        Subtitles = 12,
        /// video alternate audio channel
        VideoAlternateAudioChannel = 13,
        /// pause on power removal
        PauseOnPowerRemoval = 15,
        /// VoiceOver
        VoiceOver = 20,
        /// AssistiveTouch
        AssistiveTouch = 22,
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum SetiPodPreferencesVideoOutSetting: u8 {
        /// off
        Off = 0,
        /// on
        On = 1,
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum SetiPodPreferencesScreenConfiguration: u8 {
        /// fill entire screen
        FillEntireScreen = 0,
        /// fit to screen edge
        FitToScreenEdge = 1,
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum SetiPodPreferencesVideoSignalFormat: u8 {
        /// NTSC
        NTSC = 0,
        /// PAL
        PAL = 1,
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum SetiPodPreferencesLineOutUsage: u8 {
        /// disabled
        Disabled = 0,
        /// enabled
        Enabled = 1,
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum SetiPodPreferencesVideoOutConnection: u8 {
        /// composite
        Composite = 1,
        /// S-Video
        SVideo = 2,
        /// component
        Component = 3,
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum SetiPodPreferencesClosedCaptioning: u8 {
        /// disabled
        Disabled = 0,
        /// enabled
        Enabled = 1,
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum SetiPodPreferencesVideoMonitorAspectRatio: u8 {
        /// 4:3 aspect ratio (fullscreen)
        Value43AspectRatioFullscreen = 0,
        /// 16:9 aspect ratio (widescreen)
        Value169AspectRatioWidescreen = 1,
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum SetiPodPreferencesVoiceOver: u8 {
        /// VoiceOver off
        VoiceOverOff = 0,
        /// VoiceOver on
        VoiceOverOn = 1,
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum SetiPodPreferencesAssistiveTouch: u8 {
        /// disabled
        Disabled = 0,
        /// enabled
        Enabled = 1,
    }
}

iap1_disjoint! {
    strings = LingoString;
    #[iap1_disjoint(predicate = integer)]
    pub enum SetiPodPreferencesPreferenceSettingIDDisjoint {
        #[iap1_disjoint(decode = decode_video_out_setting, encode = encode_video_out_setting)]
        /// Video Out Setting
        VideoOutSetting(SetiPodPreferencesVideoOutSetting),
        #[iap1_disjoint(decode = decode_screen_configuration, encode = encode_screen_configuration)]
        /// Screen Configuration
        ScreenConfiguration(SetiPodPreferencesScreenConfiguration),
        #[iap1_disjoint(decode = decode_video_signal_format, encode = encode_video_signal_format)]
        /// Video Signal Format
        VideoSignalFormat(SetiPodPreferencesVideoSignalFormat),
        #[iap1_disjoint(decode = decode_line_out_usage, encode = encode_line_out_usage)]
        /// Line Out Usage
        LineOutUsage(SetiPodPreferencesLineOutUsage),
        #[iap1_disjoint(decode = decode_video_out_connection, encode = encode_video_out_connection)]
        /// Video Out Connection
        VideoOutConnection(SetiPodPreferencesVideoOutConnection),
        #[iap1_disjoint(decode = decode_closed_captioning, encode = encode_closed_captioning)]
        /// Closed Captioning
        ClosedCaptioning(SetiPodPreferencesClosedCaptioning),
        #[iap1_disjoint(decode = decode_video_monitor_aspect_ratio, encode = encode_video_monitor_aspect_ratio)]
        /// Video Monitor Aspect Ratio
        VideoMonitorAspectRatio(SetiPodPreferencesVideoMonitorAspectRatio),
        #[iap1_disjoint(decode = decode_subtitles, encode = encode_subtitles)]
        /// Subtitles
        Subtitles(bool),
        #[iap1_disjoint(decode = decode_video_alternate_audio_channel, encode = encode_video_alternate_audio_channel)]
        /// Video Alternate Audio Channel
        VideoAlternateAudioChannel(bool),
        #[iap1_disjoint(decode = decode_pause_on_power_removal, encode = encode_pause_on_power_removal)]
        /// Pause On Power Removal
        PauseOnPowerRemoval(bool),
        #[iap1_disjoint(decode = decode_voice_over, encode = encode_voice_over)]
        /// VoiceOver
        VoiceOver(SetiPodPreferencesVoiceOver),
        #[iap1_disjoint(decode = decode_assistive_touch, encode = encode_assistive_touch)]
        /// AssistiveTouch
        AssistiveTouch(SetiPodPreferencesAssistiveTouch),
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x00, command = 0x002b,
        source = accessory,
        response = false,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct SetiPodPreferences {
        fields {
            /// Preference Class
            // Virtual schema indices: 1
            required pub preference_class_id: SetiPodPreferencesPreferenceClassID => { bit: 0, key: "preferenceClassID", when: (truthy(&true)), predicate_value: true },
            /// Video Out Setting
            // Virtual schema indices: 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13
            optional pub preference_setting_id: SetiPodPreferencesPreferenceSettingIDDisjoint => { bit: 1, key: "preferenceSettingID", when: (eq(&preference_class_id, &0u64)) || (eq(&preference_class_id, &1u64)) || (eq(&preference_class_id, &2u64)) || (eq(&preference_class_id, &3u64)) || (eq(&preference_class_id, &8u64)) || (eq(&preference_class_id, &9u64)) || (eq(&preference_class_id, &10u64)) || (eq(&preference_class_id, &12u64)) || (eq(&preference_class_id, &13u64)) || (eq(&preference_class_id, &15u64)) || (eq(&preference_class_id, &20u64)) || (eq(&preference_class_id, &22u64)), predicate_value: false },
            /// Restore on Exit
            // Virtual schema indices: 14
            required pub restore_on_exit: bool => { bit: 2, key: "restoreOnExit", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required preference_class_id: SetiPodPreferencesPreferenceClassID => { bit: 0, key: "preferenceClassID", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: true, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            2 => optional preference_setting_id: SetiPodPreferencesPreferenceSettingIDDisjoint => { bit: 1, key: "preferenceSettingID", wire: [disjoint(decode_video_out_setting, encode_video_out_setting, SetiPodPreferencesVideoOutSetting, [scalar])], project_wire: Iap1WireSpec::Disjoint { variant: const { LingoString::require_raw(stringify!(decode_video_out_setting)) }, child: &Iap1WireSpec::Scalar }, predicate_value: false, predicate: { expr: eq(&preference_class_id, &0u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(0u8)] } },
            3 => optional preference_setting_id: SetiPodPreferencesPreferenceSettingIDDisjoint => { bit: 1, key: "preferenceSettingID", wire: [disjoint(decode_screen_configuration, encode_screen_configuration, SetiPodPreferencesScreenConfiguration, [scalar])], project_wire: Iap1WireSpec::Disjoint { variant: const { LingoString::require_raw(stringify!(decode_screen_configuration)) }, child: &Iap1WireSpec::Scalar }, predicate_value: false, predicate: { expr: eq(&preference_class_id, &1u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(1u8)] } },
            4 => optional preference_setting_id: SetiPodPreferencesPreferenceSettingIDDisjoint => { bit: 1, key: "preferenceSettingID", wire: [disjoint(decode_video_signal_format, encode_video_signal_format, SetiPodPreferencesVideoSignalFormat, [scalar])], project_wire: Iap1WireSpec::Disjoint { variant: const { LingoString::require_raw(stringify!(decode_video_signal_format)) }, child: &Iap1WireSpec::Scalar }, predicate_value: false, predicate: { expr: eq(&preference_class_id, &2u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(2u8)] } },
            5 => optional preference_setting_id: SetiPodPreferencesPreferenceSettingIDDisjoint => { bit: 1, key: "preferenceSettingID", wire: [disjoint(decode_line_out_usage, encode_line_out_usage, SetiPodPreferencesLineOutUsage, [scalar])], project_wire: Iap1WireSpec::Disjoint { variant: const { LingoString::require_raw(stringify!(decode_line_out_usage)) }, child: &Iap1WireSpec::Scalar }, predicate_value: false, predicate: { expr: eq(&preference_class_id, &3u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(3u8)] } },
            6 => optional preference_setting_id: SetiPodPreferencesPreferenceSettingIDDisjoint => { bit: 1, key: "preferenceSettingID", wire: [disjoint(decode_video_out_connection, encode_video_out_connection, SetiPodPreferencesVideoOutConnection, [scalar])], project_wire: Iap1WireSpec::Disjoint { variant: const { LingoString::require_raw(stringify!(decode_video_out_connection)) }, child: &Iap1WireSpec::Scalar }, predicate_value: false, predicate: { expr: eq(&preference_class_id, &8u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(8u8)] } },
            7 => optional preference_setting_id: SetiPodPreferencesPreferenceSettingIDDisjoint => { bit: 1, key: "preferenceSettingID", wire: [disjoint(decode_closed_captioning, encode_closed_captioning, SetiPodPreferencesClosedCaptioning, [scalar])], project_wire: Iap1WireSpec::Disjoint { variant: const { LingoString::require_raw(stringify!(decode_closed_captioning)) }, child: &Iap1WireSpec::Scalar }, predicate_value: false, predicate: { expr: eq(&preference_class_id, &9u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(9u8)] } },
            8 => optional preference_setting_id: SetiPodPreferencesPreferenceSettingIDDisjoint => { bit: 1, key: "preferenceSettingID", wire: [disjoint(decode_video_monitor_aspect_ratio, encode_video_monitor_aspect_ratio, SetiPodPreferencesVideoMonitorAspectRatio, [scalar])], project_wire: Iap1WireSpec::Disjoint { variant: const { LingoString::require_raw(stringify!(decode_video_monitor_aspect_ratio)) }, child: &Iap1WireSpec::Scalar }, predicate_value: false, predicate: { expr: eq(&preference_class_id, &10u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(10u8)] } },
            9 => optional preference_setting_id: SetiPodPreferencesPreferenceSettingIDDisjoint => { bit: 1, key: "preferenceSettingID", wire: [disjoint(decode_subtitles, encode_subtitles, bool, [scalar])], project_wire: Iap1WireSpec::Disjoint { variant: const { LingoString::require_raw(stringify!(decode_subtitles)) }, child: &Iap1WireSpec::Scalar }, predicate_value: false, predicate: { expr: eq(&preference_class_id, &12u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(12u8)] } },
            10 => optional preference_setting_id: SetiPodPreferencesPreferenceSettingIDDisjoint => { bit: 1, key: "preferenceSettingID", wire: [disjoint(decode_video_alternate_audio_channel, encode_video_alternate_audio_channel, bool, [scalar])], project_wire: Iap1WireSpec::Disjoint { variant: const { LingoString::require_raw(stringify!(decode_video_alternate_audio_channel)) }, child: &Iap1WireSpec::Scalar }, predicate_value: false, predicate: { expr: eq(&preference_class_id, &13u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(13u8)] } },
            11 => optional preference_setting_id: SetiPodPreferencesPreferenceSettingIDDisjoint => { bit: 1, key: "preferenceSettingID", wire: [disjoint(decode_pause_on_power_removal, encode_pause_on_power_removal, bool, [scalar])], project_wire: Iap1WireSpec::Disjoint { variant: const { LingoString::require_raw(stringify!(decode_pause_on_power_removal)) }, child: &Iap1WireSpec::Scalar }, predicate_value: false, predicate: { expr: eq(&preference_class_id, &15u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(15u8)] } },
            12 => optional preference_setting_id: SetiPodPreferencesPreferenceSettingIDDisjoint => { bit: 1, key: "preferenceSettingID", wire: [disjoint(decode_voice_over, encode_voice_over, SetiPodPreferencesVoiceOver, [scalar])], project_wire: Iap1WireSpec::Disjoint { variant: const { LingoString::require_raw(stringify!(decode_voice_over)) }, child: &Iap1WireSpec::Scalar }, predicate_value: false, predicate: { expr: eq(&preference_class_id, &20u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(20u8)] } },
            13 => optional preference_setting_id: SetiPodPreferencesPreferenceSettingIDDisjoint => { bit: 1, key: "preferenceSettingID", wire: [disjoint(decode_assistive_touch, encode_assistive_touch, SetiPodPreferencesAssistiveTouch, [scalar])], project_wire: Iap1WireSpec::Disjoint { variant: const { LingoString::require_raw(stringify!(decode_assistive_touch)) }, child: &Iap1WireSpec::Scalar }, predicate_value: false, predicate: { expr: eq(&preference_class_id, &22u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(22u8)] } },
            14 => required restore_on_exit: bool => { bit: 2, key: "restoreOnExit", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x00, command = 0x0035,
        source = accessory,
        response = false,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct GetUIMode {
        fields {
        }
        steps {
        }
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum RetUIModeUiMode: u8 {
        /// Standard
        Standard = 0,
        /// Extended Interface
        ExtendedInterface = 1,
        /// iPod Out
        IPodOut = 2,
        /// iPod Out (action safe)
        IPodOutActionSafe = 3,
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x00, command = 0x0036,
        source = device,
        response = true,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct RetUIMode {
        fields {
            /// UI Mode
            // Virtual schema indices: 1
            required pub ui_mode: RetUIModeUiMode => { bit: 0, key: "uiMode", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required ui_mode: RetUIModeUiMode => { bit: 0, key: "uiMode", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum SetUIModeUiMode: u8 {
        /// Standard
        Standard = 0,
        /// Extended Interface
        ExtendedInterface = 1,
        /// iPod Out
        IPodOut = 2,
        /// iPod Out (action safe)
        IPodOutActionSafe = 3,
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x00, command = 0x0037,
        source = accessory,
        response = false,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct SetUIMode {
        fields {
            /// UI Mode
            // Virtual schema indices: 1
            required pub ui_mode: SetUIModeUiMode => { bit: 0, key: "uiMode", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required ui_mode: SetUIModeUiMode => { bit: 0, key: "uiMode", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x00, command = 0x0038,
        source = accessory,
        response = false,
        ack = false,
        deprecated = false,
        transaction_id = required
    )]
    pub struct StartIDPS {
        fields {
        }
        steps {
        }
    }
}

iap1_bitfield! {
    pub struct SetFIDTokenValuesTokensIdentifyTokenOptions: u32 {
        #[group(mask = 0x3, shift = 0)]
        /// authentication control bits
        pub authentication_control_bits: SetFIDTokenValuesTokensIdentifyTokenOptionsAuthenticationControlBits {
            /// immediate authentication
            ImmediateAuthentication = 2,
        },
        #[group(mask = 0xc, shift = 2)]
        /// power control bits
        pub power_control_bits: SetFIDTokenValuesTokensIdentifyTokenOptionsPowerControlBits {
            /// low power
            LowPower = 0,
            /// intermittent high power
            #[deprecated]
            IntermittentHighPower = 1,
            /// constant high power
            #[deprecated]
            ConstantHighPower = 3,
        },
    }
}

iap1_record! {
    strings = LingoString;
    pub struct SetFIDTokenValuesTokensIdentifyToken {
        fields {
            /// Number of Lingoes
            // Virtual schema indices: 1
            required pub num_lingoes: u8 => { bit: 0, key: "numLingoes", when: (truthy(&true)), predicate_value: true },
            /// Accessory Lingoes
            // Virtual schema indices: 2
            required pub accessory_lingoes: Vec<u8> => { bit: 1, key: "accessoryLingoes", when: (truthy(&true)), predicate_value: false },
            /// Options
            // Virtual schema indices: 3
            required pub options: SetFIDTokenValuesTokensIdentifyTokenOptions => { bit: 2, key: "options", when: (truthy(&true)), predicate_value: false },
            /// Device ID
            // Virtual schema indices: 4
            required pub device_id: u32 => { bit: 3, key: "deviceID", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required num_lingoes: u8 => { bit: 0, key: "numLingoes", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: true, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            2 => required accessory_lingoes: Vec<u8> => { bit: 1, key: "accessoryLingoes", wire: [bytes(num_lingoes)], project_wire: Iap1WireSpec::CountedField(0), predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            3 => required options: SetFIDTokenValuesTokensIdentifyTokenOptions => { bit: 2, key: "options", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            4 => required device_id: u32 => { bit: 3, key: "deviceID", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1_bitflags! {
    strings = LingoString;
    pub struct SetFIDTokenValuesTokensAccessoryCapsTokenAccCaps: u64 {
        /// analog line out
        const ANALOG_LINE_OUT = 1 << 0;
        /// analog line in
        const ANALOG_LINE_IN = 1 << 1;
        /// analog video out
        const ANALOG_VIDEO_OUT = 1 << 2;
        /// USB audio
        const USB_AUDIO = 1 << 4;
        /// accessory supports communication with iOS Applications
        const ACCESSORY_SUPPORTS_COMMUNICATION_WITH_I_OS_APPLICATIONS = 1 << 9;
        /// accessory checks iPod volume
        const ACCESSORY_CHECKS_I_POD_VOLUME = 1 << 11;
        /// VoiceOver
        const VOICE_OVER = 1 << 17;
        /// async playback state changes
        const ASYNC_PLAYBACK_STATE_CHANGES = 1 << 18;
        /// multi-packet responses
        const MULTI_PACKET_RESPONSES = 1 << 19;
        /// analog USB audio routing
        const ANALOG_USB_AUDIO_ROUTING = 1 << 21;
        /// AssistiveTouch
        const ASSISTIVE_TOUCH = 1 << 23;
    }
}

iap1_record! {
    strings = LingoString;
    pub struct SetFIDTokenValuesTokensAccessoryCapsToken {
        fields {
            /// Accessory Capabilities Bits
            // Virtual schema indices: 1
            required pub acc_caps: SetFIDTokenValuesTokensAccessoryCapsTokenAccCaps => { bit: 0, key: "accCaps", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required acc_caps: SetFIDTokenValuesTokensAccessoryCapsTokenAccCaps => { bit: 0, key: "accCaps", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum SetFIDTokenValuesTokensAccessoryInfoTokenAccInfoType: u8 {
        /// name
        Name = 1,
        /// firmware version
        FirmwareVersion = 4,
        /// hardware version
        HardwareVersion = 5,
        /// manufacturer
        Manufacturer = 6,
        /// model number
        ModelNumber = 7,
        /// serial number
        SerialNumber = 8,
        /// incoming maximum payload size
        IncomingMaximumPayloadSize = 9,
        /// status types
        StatusTypes = 11,
        /// RF certifications
        RFCertifications = 12,
    }
}

iap1_bitflags! {
    strings = LingoString;
    pub struct SetFIDTokenValuesTokensAccessoryInfoTokenStatusTypesSupported: u32 {
        /// Bluetooth device
        const BLUETOOTH_DEVICE = 1 << 1;
        /// fault condition
        const FAULT_CONDITION = 1 << 2;
    }
}

iap1_bitflags! {
    strings = LingoString;
    pub struct SetFIDTokenValuesTokensAccessoryInfoTokenRfCertificationDeclaration: u32 {
        /// Class 1: iPhone, iPhone 3G, iPhone 3GS
        const CLASS1 = 1 << 0;
        /// Class 2: iPhone 4 (GSM model)
        const CLASS2 = 1 << 1;
        /// Class 4: iPhone 4 (CDMA model)
        const CLASS4 = 1 << 3;
        /// Class 5: iPhone 4S
        const CLASS5 = 1 << 4;
        /// Class 6: iPhone 5 (A1428 model)
        const CLASS6 = 1 << 5;
        /// Class 7: iPhone 5 (A1429 model)
        const CLASS7 = 1 << 6;
    }
}

iap1_record! {
    strings = LingoString;
    pub struct SetFIDTokenValuesTokensAccessoryInfoToken {
        fields {
            /// Accessory Info Type
            // Virtual schema indices: 1
            required pub acc_info_type: SetFIDTokenValuesTokensAccessoryInfoTokenAccInfoType => { bit: 0, key: "accInfoType", when: (truthy(&true)), predicate_value: true },
            /// Name
            // Virtual schema indices: 2
            optional pub acc_name: String => { bit: 1, key: "accName", when: (eq(&acc_info_type, &1u64)), predicate_value: false },
            /// Firmware Version
            // Virtual schema indices: 3
            optional pub acc_firmware_version: [u8; 3] => { bit: 2, key: "accFirmwareVersion", when: (eq(&acc_info_type, &4u64)), predicate_value: false },
            /// Hardware Version
            // Virtual schema indices: 4
            optional pub acc_hardware_version: [u8; 3] => { bit: 3, key: "accHardwareVersion", when: (eq(&acc_info_type, &5u64)), predicate_value: false },
            /// Manufacturer
            // Virtual schema indices: 5
            optional pub acc_manufacturer: String => { bit: 4, key: "accManufacturer", when: (eq(&acc_info_type, &6u64)), predicate_value: false },
            /// Model Number
            // Virtual schema indices: 6
            optional pub acc_model_number: String => { bit: 5, key: "accModelNumber", when: (eq(&acc_info_type, &7u64)), predicate_value: false },
            /// Serial Number
            // Virtual schema indices: 7
            optional pub acc_serial_number: String => { bit: 6, key: "accSerialNumber", when: (eq(&acc_info_type, &8u64)), predicate_value: false },
            /// Incoming Maximum Payload Size
            // Virtual schema indices: 8
            optional pub acc_max_payload: u16 => { bit: 7, key: "accMaxPayload", when: (eq(&acc_info_type, &9u64)), predicate_value: false },
            /// Status Types Supported
            // Virtual schema indices: 9
            optional pub status_types_supported: SetFIDTokenValuesTokensAccessoryInfoTokenStatusTypesSupported => { bit: 8, key: "statusTypesSupported", when: (eq(&acc_info_type, &11u64)), predicate_value: false },
            /// Accessory RF certification declaration (R46 Table 3-74)
            // Virtual schema indices: 10
            optional pub rf_certification_declaration: SetFIDTokenValuesTokensAccessoryInfoTokenRfCertificationDeclaration => { bit: 9, key: "rfCertificationDeclaration", when: (eq(&acc_info_type, &12u64)), predicate_value: false },
        }
        steps {
            1 => required acc_info_type: SetFIDTokenValuesTokensAccessoryInfoTokenAccInfoType => { bit: 0, key: "accInfoType", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: true, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            2 => optional acc_name: String => { bit: 1, key: "accName", wire: [raw_string], project_wire: Iap1WireSpec::Raw, predicate_value: false, predicate: { expr: eq(&acc_info_type, &1u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(1u8)] } },
            3 => optional acc_firmware_version: [u8; 3] => { bit: 2, key: "accFirmwareVersion", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&acc_info_type, &4u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(4u8)] } },
            4 => optional acc_hardware_version: [u8; 3] => { bit: 3, key: "accHardwareVersion", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&acc_info_type, &5u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(5u8)] } },
            5 => optional acc_manufacturer: String => { bit: 4, key: "accManufacturer", wire: [raw_string], project_wire: Iap1WireSpec::Raw, predicate_value: false, predicate: { expr: eq(&acc_info_type, &6u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(6u8)] } },
            6 => optional acc_model_number: String => { bit: 5, key: "accModelNumber", wire: [raw_string], project_wire: Iap1WireSpec::Raw, predicate_value: false, predicate: { expr: eq(&acc_info_type, &7u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(7u8)] } },
            7 => optional acc_serial_number: String => { bit: 6, key: "accSerialNumber", wire: [raw_string], project_wire: Iap1WireSpec::Raw, predicate_value: false, predicate: { expr: eq(&acc_info_type, &8u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(8u8)] } },
            8 => optional acc_max_payload: u16 => { bit: 7, key: "accMaxPayload", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&acc_info_type, &9u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(9u8)] } },
            9 => optional status_types_supported: SetFIDTokenValuesTokensAccessoryInfoTokenStatusTypesSupported => { bit: 8, key: "statusTypesSupported", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&acc_info_type, &11u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(11u8)] } },
            10 => optional rf_certification_declaration: SetFIDTokenValuesTokensAccessoryInfoTokenRfCertificationDeclaration => { bit: 9, key: "rfCertificationDeclaration", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&acc_info_type, &12u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(12u8)] } },
        }
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum SetFIDTokenValuesTokensIPodPreferenceTokenPreferenceClassID: u8 {
        /// video out setting
        VideoOutSetting = 0,
        /// screen configuration
        ScreenConfiguration = 1,
        /// video signal format
        VideoSignalFormat = 2,
        /// line-out usage
        LineOutUsage = 3,
        /// video-out connection
        VideoOutConnection = 8,
        /// closed captioning
        ClosedCaptioning = 9,
        /// video monitor aspect ratio
        VideoMonitorAspectRatio = 10,
        /// subtitles
        Subtitles = 12,
        /// video alternate audio channel
        VideoAlternateAudioChannel = 13,
        /// pause on power removal
        PauseOnPowerRemoval = 15,
        /// VoiceOver
        VoiceOver = 20,
        /// AssistiveTouch
        AssistiveTouch = 22,
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum SetFIDTokenValuesTokensIPodPreferenceTokenVideoOutSetting: u8 {
        /// off
        Off = 0,
        /// on
        On = 1,
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum SetFIDTokenValuesTokensIPodPreferenceTokenScreenConfiguration: u8 {
        /// fill entire screen
        FillEntireScreen = 0,
        /// fit to screen edge
        FitToScreenEdge = 1,
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum SetFIDTokenValuesTokensIPodPreferenceTokenVideoSignalFormat: u8 {
        /// NTSC
        NTSC = 0,
        /// PAL
        PAL = 1,
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum SetFIDTokenValuesTokensIPodPreferenceTokenLineOutUsage: u8 {
        /// disabled
        Disabled = 0,
        /// enabled
        Enabled = 1,
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum SetFIDTokenValuesTokensIPodPreferenceTokenVideoOutConnection: u8 {
        /// no change
        NoChange = 0,
        /// composite
        Composite = 1,
        /// S-Video
        SVideo = 2,
        /// component
        Component = 3,
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum SetFIDTokenValuesTokensIPodPreferenceTokenClosedCaptioning: u8 {
        /// disabled
        Disabled = 0,
        /// enabled
        Enabled = 1,
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum SetFIDTokenValuesTokensIPodPreferenceTokenVideoMonitorAspectRatio: u8 {
        /// 4:3 aspect ratio (fullscreen)
        Value43AspectRatioFullscreen = 0,
        /// 16:9 aspect ratio (widescreen)
        Value169AspectRatioWidescreen = 1,
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum SetFIDTokenValuesTokensIPodPreferenceTokenVoiceOver: u8 {
        /// VoiceOver off
        VoiceOverOff = 0,
        /// VoiceOver on
        VoiceOverOn = 1,
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum SetFIDTokenValuesTokensIPodPreferenceTokenAssistiveTouch: u8 {
        /// disabled
        Disabled = 0,
        /// enabled
        Enabled = 1,
    }
}

iap1_disjoint! {
    strings = LingoString;
    #[iap1_disjoint(predicate = integer)]
    pub enum SetFIDTokenValuesTokensIPodPreferenceTokenPreferenceSettingIDDisjoint {
        #[iap1_disjoint(decode = decode_video_out_setting, encode = encode_video_out_setting)]
        /// Video Out Setting
        VideoOutSetting(SetFIDTokenValuesTokensIPodPreferenceTokenVideoOutSetting),
        #[iap1_disjoint(decode = decode_screen_configuration, encode = encode_screen_configuration)]
        /// Screen Configuration
        ScreenConfiguration(SetFIDTokenValuesTokensIPodPreferenceTokenScreenConfiguration),
        #[iap1_disjoint(decode = decode_video_signal_format, encode = encode_video_signal_format)]
        /// Video Signal Format
        VideoSignalFormat(SetFIDTokenValuesTokensIPodPreferenceTokenVideoSignalFormat),
        #[iap1_disjoint(decode = decode_line_out_usage, encode = encode_line_out_usage)]
        /// Line Out Usage
        LineOutUsage(SetFIDTokenValuesTokensIPodPreferenceTokenLineOutUsage),
        #[iap1_disjoint(decode = decode_video_out_connection, encode = encode_video_out_connection)]
        /// Video Out Connection
        VideoOutConnection(SetFIDTokenValuesTokensIPodPreferenceTokenVideoOutConnection),
        #[iap1_disjoint(decode = decode_closed_captioning, encode = encode_closed_captioning)]
        /// Closed Captioning
        ClosedCaptioning(SetFIDTokenValuesTokensIPodPreferenceTokenClosedCaptioning),
        #[iap1_disjoint(decode = decode_video_monitor_aspect_ratio, encode = encode_video_monitor_aspect_ratio)]
        /// Video Monitor Aspect Ratio
        VideoMonitorAspectRatio(SetFIDTokenValuesTokensIPodPreferenceTokenVideoMonitorAspectRatio),
        #[iap1_disjoint(decode = decode_subtitles, encode = encode_subtitles)]
        /// Subtitles
        Subtitles(bool),
        #[iap1_disjoint(decode = decode_video_alternate_audio_channel, encode = encode_video_alternate_audio_channel)]
        /// Video Alternate Audio Channel
        VideoAlternateAudioChannel(bool),
        #[iap1_disjoint(decode = decode_pause_on_power_removal, encode = encode_pause_on_power_removal)]
        /// Pause On Power Removal
        PauseOnPowerRemoval(bool),
        #[iap1_disjoint(decode = decode_voice_over, encode = encode_voice_over)]
        /// VoiceOver
        VoiceOver(SetFIDTokenValuesTokensIPodPreferenceTokenVoiceOver),
        #[iap1_disjoint(decode = decode_assistive_touch, encode = encode_assistive_touch)]
        /// AssistiveTouch
        AssistiveTouch(SetFIDTokenValuesTokensIPodPreferenceTokenAssistiveTouch),
    }
}

iap1_record! {
    strings = LingoString;
    pub struct SetFIDTokenValuesTokensIPodPreferenceToken {
        fields {
            /// Preference Class
            // Virtual schema indices: 1
            required pub preference_class_id: SetFIDTokenValuesTokensIPodPreferenceTokenPreferenceClassID => { bit: 0, key: "preferenceClassID", when: (truthy(&true)), predicate_value: true },
            /// Video Out Setting
            // Virtual schema indices: 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13
            optional pub preference_setting_id: SetFIDTokenValuesTokensIPodPreferenceTokenPreferenceSettingIDDisjoint => { bit: 1, key: "preferenceSettingID", when: (eq(&preference_class_id, &0u64)) || (eq(&preference_class_id, &1u64)) || (eq(&preference_class_id, &2u64)) || (eq(&preference_class_id, &3u64)) || (eq(&preference_class_id, &8u64)) || (eq(&preference_class_id, &9u64)) || (eq(&preference_class_id, &10u64)) || (eq(&preference_class_id, &12u64)) || (eq(&preference_class_id, &13u64)) || (eq(&preference_class_id, &15u64)) || (eq(&preference_class_id, &20u64)) || (eq(&preference_class_id, &22u64)), predicate_value: false },
            /// Restore on Exit
            // Virtual schema indices: 14
            required pub restore_on_exit: u8 => { bit: 2, key: "restoreOnExit", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required preference_class_id: SetFIDTokenValuesTokensIPodPreferenceTokenPreferenceClassID => { bit: 0, key: "preferenceClassID", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: true, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            2 => optional preference_setting_id: SetFIDTokenValuesTokensIPodPreferenceTokenPreferenceSettingIDDisjoint => { bit: 1, key: "preferenceSettingID", wire: [disjoint(decode_video_out_setting, encode_video_out_setting, SetFIDTokenValuesTokensIPodPreferenceTokenVideoOutSetting, [scalar])], project_wire: Iap1WireSpec::Disjoint { variant: const { LingoString::require_raw(stringify!(decode_video_out_setting)) }, child: &Iap1WireSpec::Scalar }, predicate_value: false, predicate: { expr: eq(&preference_class_id, &0u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(0u8)] } },
            3 => optional preference_setting_id: SetFIDTokenValuesTokensIPodPreferenceTokenPreferenceSettingIDDisjoint => { bit: 1, key: "preferenceSettingID", wire: [disjoint(decode_screen_configuration, encode_screen_configuration, SetFIDTokenValuesTokensIPodPreferenceTokenScreenConfiguration, [scalar])], project_wire: Iap1WireSpec::Disjoint { variant: const { LingoString::require_raw(stringify!(decode_screen_configuration)) }, child: &Iap1WireSpec::Scalar }, predicate_value: false, predicate: { expr: eq(&preference_class_id, &1u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(1u8)] } },
            4 => optional preference_setting_id: SetFIDTokenValuesTokensIPodPreferenceTokenPreferenceSettingIDDisjoint => { bit: 1, key: "preferenceSettingID", wire: [disjoint(decode_video_signal_format, encode_video_signal_format, SetFIDTokenValuesTokensIPodPreferenceTokenVideoSignalFormat, [scalar])], project_wire: Iap1WireSpec::Disjoint { variant: const { LingoString::require_raw(stringify!(decode_video_signal_format)) }, child: &Iap1WireSpec::Scalar }, predicate_value: false, predicate: { expr: eq(&preference_class_id, &2u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(2u8)] } },
            5 => optional preference_setting_id: SetFIDTokenValuesTokensIPodPreferenceTokenPreferenceSettingIDDisjoint => { bit: 1, key: "preferenceSettingID", wire: [disjoint(decode_line_out_usage, encode_line_out_usage, SetFIDTokenValuesTokensIPodPreferenceTokenLineOutUsage, [scalar])], project_wire: Iap1WireSpec::Disjoint { variant: const { LingoString::require_raw(stringify!(decode_line_out_usage)) }, child: &Iap1WireSpec::Scalar }, predicate_value: false, predicate: { expr: eq(&preference_class_id, &3u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(3u8)] } },
            6 => optional preference_setting_id: SetFIDTokenValuesTokensIPodPreferenceTokenPreferenceSettingIDDisjoint => { bit: 1, key: "preferenceSettingID", wire: [disjoint(decode_video_out_connection, encode_video_out_connection, SetFIDTokenValuesTokensIPodPreferenceTokenVideoOutConnection, [scalar])], project_wire: Iap1WireSpec::Disjoint { variant: const { LingoString::require_raw(stringify!(decode_video_out_connection)) }, child: &Iap1WireSpec::Scalar }, predicate_value: false, predicate: { expr: eq(&preference_class_id, &8u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(8u8)] } },
            7 => optional preference_setting_id: SetFIDTokenValuesTokensIPodPreferenceTokenPreferenceSettingIDDisjoint => { bit: 1, key: "preferenceSettingID", wire: [disjoint(decode_closed_captioning, encode_closed_captioning, SetFIDTokenValuesTokensIPodPreferenceTokenClosedCaptioning, [scalar])], project_wire: Iap1WireSpec::Disjoint { variant: const { LingoString::require_raw(stringify!(decode_closed_captioning)) }, child: &Iap1WireSpec::Scalar }, predicate_value: false, predicate: { expr: eq(&preference_class_id, &9u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(9u8)] } },
            8 => optional preference_setting_id: SetFIDTokenValuesTokensIPodPreferenceTokenPreferenceSettingIDDisjoint => { bit: 1, key: "preferenceSettingID", wire: [disjoint(decode_video_monitor_aspect_ratio, encode_video_monitor_aspect_ratio, SetFIDTokenValuesTokensIPodPreferenceTokenVideoMonitorAspectRatio, [scalar])], project_wire: Iap1WireSpec::Disjoint { variant: const { LingoString::require_raw(stringify!(decode_video_monitor_aspect_ratio)) }, child: &Iap1WireSpec::Scalar }, predicate_value: false, predicate: { expr: eq(&preference_class_id, &10u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(10u8)] } },
            9 => optional preference_setting_id: SetFIDTokenValuesTokensIPodPreferenceTokenPreferenceSettingIDDisjoint => { bit: 1, key: "preferenceSettingID", wire: [disjoint(decode_subtitles, encode_subtitles, bool, [scalar])], project_wire: Iap1WireSpec::Disjoint { variant: const { LingoString::require_raw(stringify!(decode_subtitles)) }, child: &Iap1WireSpec::Scalar }, predicate_value: false, predicate: { expr: eq(&preference_class_id, &12u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(12u8)] } },
            10 => optional preference_setting_id: SetFIDTokenValuesTokensIPodPreferenceTokenPreferenceSettingIDDisjoint => { bit: 1, key: "preferenceSettingID", wire: [disjoint(decode_video_alternate_audio_channel, encode_video_alternate_audio_channel, bool, [scalar])], project_wire: Iap1WireSpec::Disjoint { variant: const { LingoString::require_raw(stringify!(decode_video_alternate_audio_channel)) }, child: &Iap1WireSpec::Scalar }, predicate_value: false, predicate: { expr: eq(&preference_class_id, &13u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(13u8)] } },
            11 => optional preference_setting_id: SetFIDTokenValuesTokensIPodPreferenceTokenPreferenceSettingIDDisjoint => { bit: 1, key: "preferenceSettingID", wire: [disjoint(decode_pause_on_power_removal, encode_pause_on_power_removal, bool, [scalar])], project_wire: Iap1WireSpec::Disjoint { variant: const { LingoString::require_raw(stringify!(decode_pause_on_power_removal)) }, child: &Iap1WireSpec::Scalar }, predicate_value: false, predicate: { expr: eq(&preference_class_id, &15u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(15u8)] } },
            12 => optional preference_setting_id: SetFIDTokenValuesTokensIPodPreferenceTokenPreferenceSettingIDDisjoint => { bit: 1, key: "preferenceSettingID", wire: [disjoint(decode_voice_over, encode_voice_over, SetFIDTokenValuesTokensIPodPreferenceTokenVoiceOver, [scalar])], project_wire: Iap1WireSpec::Disjoint { variant: const { LingoString::require_raw(stringify!(decode_voice_over)) }, child: &Iap1WireSpec::Scalar }, predicate_value: false, predicate: { expr: eq(&preference_class_id, &20u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(20u8)] } },
            13 => optional preference_setting_id: SetFIDTokenValuesTokensIPodPreferenceTokenPreferenceSettingIDDisjoint => { bit: 1, key: "preferenceSettingID", wire: [disjoint(decode_assistive_touch, encode_assistive_touch, SetFIDTokenValuesTokensIPodPreferenceTokenAssistiveTouch, [scalar])], project_wire: Iap1WireSpec::Disjoint { variant: const { LingoString::require_raw(stringify!(decode_assistive_touch)) }, child: &Iap1WireSpec::Scalar }, predicate_value: false, predicate: { expr: eq(&preference_class_id, &22u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(22u8)] } },
            14 => required restore_on_exit: u8 => { bit: 2, key: "restoreOnExit", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1_record! {
    strings = LingoString;
    pub struct SetFIDTokenValuesTokensEAProtocolToken {
        fields {
            /// Protocol Index
            // Virtual schema indices: 1
            required pub protocol_index: u8 => { bit: 0, key: "protocolIndex", when: (truthy(&true)), predicate_value: false },
            /// Protocol String
            // Virtual schema indices: 2
            required pub protocol_string: String => { bit: 1, key: "protocolString", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required protocol_index: u8 => { bit: 0, key: "protocolIndex", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            2 => required protocol_string: String => { bit: 1, key: "protocolString", wire: [raw_string], project_wire: Iap1WireSpec::Raw, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1_record! {
    strings = LingoString;
    pub struct SetFIDTokenValuesTokensBundleSeedIDPrefToken {
        fields {
            /// Preferred Vendor ID
            // Virtual schema indices: 1
            required pub bundle_seed_id_string: String => { bit: 0, key: "bundleSeedIDString", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required bundle_seed_id_string: String => { bit: 0, key: "bundleSeedIDString", wire: [utf8(11usize)], project_wire: Iap1WireSpec::CountedLiteral(11), predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1_bitflags! {
    strings = LingoString;
    pub struct SetFIDTokenValuesTokensScreenInfoTokenScreenFeatures: u8 {
        /// color screen
        const COLOR_SCREEN = 1 << 0;
    }
}

iap1_record! {
    strings = LingoString;
    pub struct SetFIDTokenValuesTokensScreenInfoToken {
        fields {
            /// Accessory Screen Width in Inches
            // Virtual schema indices: 1
            required pub total_screen_width_inches: u16 => { bit: 0, key: "totalScreenWidthInches", when: (truthy(&true)), predicate_value: false },
            /// Accessory Screen Height in Inches
            // Virtual schema indices: 2
            required pub total_screen_height_inches: u16 => { bit: 1, key: "totalScreenHeightInches", when: (truthy(&true)), predicate_value: false },
            /// Accessory Screen Width in Pixels
            // Virtual schema indices: 3
            required pub total_screen_width_pixels: u16 => { bit: 2, key: "totalScreenWidthPixels", when: (truthy(&true)), predicate_value: false },
            /// Accessory Screen Height in Pixels
            // Virtual schema indices: 4
            required pub total_screen_height_pixels: u16 => { bit: 3, key: "totalScreenHeightPixels", when: (truthy(&true)), predicate_value: false },
            /// iPodOut Width Allotment
            // Virtual schema indices: 5
            required pub i_pod_out_screen_width_pixels: u16 => { bit: 4, key: "iPodOutScreenWidthPixels", when: (truthy(&true)), predicate_value: false },
            /// iPodOut Height Allotment
            // Virtual schema indices: 6
            required pub i_pod_out_screen_height_pixels: u16 => { bit: 5, key: "iPodOutScreenHeightPixels", when: (truthy(&true)), predicate_value: false },
            /// Device Screen Features Bits
            // Virtual schema indices: 7
            required pub screen_features: SetFIDTokenValuesTokensScreenInfoTokenScreenFeatures => { bit: 6, key: "screenFeatures", when: (truthy(&true)), predicate_value: false },
            /// Display Gamma
            // Virtual schema indices: 8
            required pub screen_gamma_value: u8 => { bit: 7, key: "screenGammaValue", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required total_screen_width_inches: u16 => { bit: 0, key: "totalScreenWidthInches", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            2 => required total_screen_height_inches: u16 => { bit: 1, key: "totalScreenHeightInches", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            3 => required total_screen_width_pixels: u16 => { bit: 2, key: "totalScreenWidthPixels", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            4 => required total_screen_height_pixels: u16 => { bit: 3, key: "totalScreenHeightPixels", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            5 => required i_pod_out_screen_width_pixels: u16 => { bit: 4, key: "iPodOutScreenWidthPixels", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            6 => required i_pod_out_screen_height_pixels: u16 => { bit: 5, key: "iPodOutScreenHeightPixels", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            7 => required screen_features: SetFIDTokenValuesTokensScreenInfoTokenScreenFeatures => { bit: 6, key: "screenFeatures", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            8 => required screen_gamma_value: u8 => { bit: 7, key: "screenGammaValue", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum SetFIDTokenValuesTokensEAProtocolMetadataTokenMetadataType: u8 {
        /// suppress matching
        SuppressMatching = 0,
        /// default matching
        DefaultMatching = 1,
        /// no automatic matching
        NoAutomaticMatching = 3,
    }
}

iap1_record! {
    strings = LingoString;
    pub struct SetFIDTokenValuesTokensEAProtocolMetadataToken {
        fields {
            /// Protocol Index
            // Virtual schema indices: 1
            required pub protocol_index: u8 => { bit: 0, key: "protocolIndex", when: (truthy(&true)), predicate_value: false },
            /// Metadata Type
            // Virtual schema indices: 2
            required pub metadata_type: SetFIDTokenValuesTokensEAProtocolMetadataTokenMetadataType => { bit: 1, key: "metadataType", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required protocol_index: u8 => { bit: 0, key: "protocolIndex", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            2 => required metadata_type: SetFIDTokenValuesTokensEAProtocolMetadataTokenMetadataType => { bit: 1, key: "metadataType", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum SetFIDTokenValuesTokensAccessoryDigitalAudioSampleRatesTokenSampleRatesSampleRate: u32 {
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
    pub struct SetFIDTokenValuesTokensAccessoryDigitalAudioSampleRatesTokenSampleRates {
        fields {
            /// sample rate
            // Virtual schema indices: 1
            required pub sample_rate: SetFIDTokenValuesTokensAccessoryDigitalAudioSampleRatesTokenSampleRatesSampleRate => { bit: 0, key: "sampleRate", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required sample_rate: SetFIDTokenValuesTokensAccessoryDigitalAudioSampleRatesTokenSampleRatesSampleRate => { bit: 0, key: "sampleRate", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1_record! {
    strings = LingoString;
    pub struct SetFIDTokenValuesTokensAccessoryDigitalAudioSampleRatesToken {
        fields {
            /// Sample Rates
            // Virtual schema indices: 1
            required pub sample_rates: Vec<SetFIDTokenValuesTokensAccessoryDigitalAudioSampleRatesTokenSampleRates> => { bit: 0, key: "sampleRates", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required sample_rates: Vec<SetFIDTokenValuesTokensAccessoryDigitalAudioSampleRatesTokenSampleRates> => { bit: 0, key: "sampleRates", wire: [counted_remaining], project_wire: Iap1WireSpec::CountedRemaining, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1_record! {
    strings = LingoString;
    pub struct SetFIDTokenValuesTokensAccessoryDigitalAudioVideoDelayToken {
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

iap1_bitflags! {
    strings = LingoString;
    pub struct SetFIDTokenValuesTokensMicrophoneCapsTokenMicCaps: u32 {
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

iap1_record! {
    strings = LingoString;
    pub struct SetFIDTokenValuesTokensMicrophoneCapsToken {
        fields {
            /// Microphone Capabilities Bits
            // Virtual schema indices: 1
            required pub mic_caps: SetFIDTokenValuesTokensMicrophoneCapsTokenMicCaps => { bit: 0, key: "micCaps", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required mic_caps: SetFIDTokenValuesTokensMicrophoneCapsTokenMicCaps => { bit: 0, key: "micCaps", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1_enum_tokens! {
    strings = LingoString;
    pub enum SetFIDTokenValuesTokens {
        #[iap1_token(fid_type = 0, fid_subtype = 0)]
        IdentifyToken(SetFIDTokenValuesTokensIdentifyToken),
        #[iap1_token(fid_type = 0, fid_subtype = 1)]
        AccessoryCapsToken(SetFIDTokenValuesTokensAccessoryCapsToken),
        #[iap1_token(fid_type = 0, fid_subtype = 2)]
        AccessoryInfoToken(SetFIDTokenValuesTokensAccessoryInfoToken),
        #[iap1_token(fid_type = 0, fid_subtype = 3)]
        IPodPreferenceToken(SetFIDTokenValuesTokensIPodPreferenceToken),
        #[iap1_token(fid_type = 0, fid_subtype = 4)]
        EAProtocolToken(SetFIDTokenValuesTokensEAProtocolToken),
        #[iap1_token(fid_type = 0, fid_subtype = 5)]
        BundleSeedIDPrefToken(SetFIDTokenValuesTokensBundleSeedIDPrefToken),
        #[iap1_token(fid_type = 0, fid_subtype = 7)]
        ScreenInfoToken(SetFIDTokenValuesTokensScreenInfoToken),
        #[iap1_token(fid_type = 0, fid_subtype = 8)]
        EAProtocolMetadataToken(SetFIDTokenValuesTokensEAProtocolMetadataToken),
        #[iap1_token(fid_type = 0, fid_subtype = 14)]
        AccessoryDigitalAudioSampleRatesToken(SetFIDTokenValuesTokensAccessoryDigitalAudioSampleRatesToken),
        #[iap1_token(fid_type = 0, fid_subtype = 15)]
        AccessoryDigitalAudioVideoDelayToken(SetFIDTokenValuesTokensAccessoryDigitalAudioVideoDelayToken),
        #[iap1_token(fid_type = 1, fid_subtype = 0)]
        MicrophoneCapsToken(SetFIDTokenValuesTokensMicrophoneCapsToken),
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x00, command = 0x0039,
        source = accessory,
        response = false,
        ack = false,
        deprecated = false,
        transaction_id = required
    )]
    pub struct SetFIDTokenValues {
        fields {
            /// Number of Tokens
            // Virtual schema indices: 1
            required pub num_fid_token_values: u8 => { bit: 0, key: "numFIDTokenValues", when: (truthy(&true)), predicate_value: true },
            /// Tokens
            // Virtual schema indices: 2
            required pub tokens: Vec<SetFIDTokenValuesTokens> => { bit: 1, key: "tokens", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required num_fid_token_values: u8 => { bit: 0, key: "numFIDTokenValues", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: true, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            2 => required tokens: Vec<SetFIDTokenValuesTokens> => { bit: 1, key: "tokens", wire: [counted(num_fid_token_values)], project_wire: Iap1WireSpec::CountedField(0), predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum AckFIDTokenValuesTokenACKsIdentifyTokenAckStatus: u8 {
        /// accepted
        Accepted = 0,
        /// required token failed
        RequiredTokenFailed = 1,
        /// optional token failed
        OptionalTokenFailed = 2,
        /// token not supported
        TokenNotSupported = 3,
        /// lingoes busy
        LingoesBusy = 4,
        /// maximum connections reached
        MaximumConnectionsReached = 5,
    }
}

iap1_record! {
    strings = LingoString;
    pub struct AckFIDTokenValuesTokenACKsIdentifyToken {
        fields {
            /// Status
            // Virtual schema indices: 1
            required pub ack_status: AckFIDTokenValuesTokenACKsIdentifyTokenAckStatus => { bit: 0, key: "ackStatus", when: (truthy(&true)), predicate_value: true },
            /// Busy Lingoes
            // Virtual schema indices: 2
            optional pub busy_lingoes: Vec<u8> => { bit: 1, key: "busyLingoes", when: (eq(&ack_status, &4u64)), predicate_value: false },
        }
        steps {
            1 => required ack_status: AckFIDTokenValuesTokenACKsIdentifyTokenAckStatus => { bit: 0, key: "ackStatus", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: true, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            2 => optional busy_lingoes: Vec<u8> => { bit: 1, key: "busyLingoes", wire: [raw_bytes], project_wire: Iap1WireSpec::Raw, predicate_value: false, predicate: { expr: eq(&ack_status, &4u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(4u8)] } },
        }
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum AckFIDTokenValuesTokenACKsAccessoryCapsTokenAckStatus: u8 {
        /// accepted
        Accepted = 0,
        /// required token failed
        RequiredTokenFailed = 1,
        /// optional token failed
        OptionalTokenFailed = 2,
        /// token not supported
        TokenNotSupported = 3,
        /// lingoes busy
        LingoesBusy = 4,
        /// maximum connections reached
        MaximumConnectionsReached = 5,
    }
}

iap1_record! {
    strings = LingoString;
    pub struct AckFIDTokenValuesTokenACKsAccessoryCapsToken {
        fields {
            /// Status
            // Virtual schema indices: 1
            required pub ack_status: AckFIDTokenValuesTokenACKsAccessoryCapsTokenAckStatus => { bit: 0, key: "ackStatus", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required ack_status: AckFIDTokenValuesTokenACKsAccessoryCapsTokenAckStatus => { bit: 0, key: "ackStatus", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum AckFIDTokenValuesTokenACKsAccessoryInfoTokenAckStatus: u8 {
        /// accepted
        Accepted = 0,
        /// required token failed
        RequiredTokenFailed = 1,
        /// optional token failed
        OptionalTokenFailed = 2,
        /// token not supported
        TokenNotSupported = 3,
        /// lingoes busy
        LingoesBusy = 4,
        /// maximum connections reached
        MaximumConnectionsReached = 5,
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum AckFIDTokenValuesTokenACKsAccessoryInfoTokenAccInfoType: u8 {
        /// name
        Name = 1,
        /// firmware version
        FirmwareVersion = 4,
        /// hardware version
        HardwareVersion = 5,
        /// manufacturer
        Manufacturer = 6,
        /// model number
        ModelNumber = 7,
        /// serial number
        SerialNumber = 8,
        /// maximum payload size
        MaximumPayloadSize = 9,
        /// status types
        StatusTypes = 11,
        /// RF certifications
        RFCertifications = 12,
    }
}

iap1_record! {
    strings = LingoString;
    pub struct AckFIDTokenValuesTokenACKsAccessoryInfoToken {
        fields {
            /// Status
            // Virtual schema indices: 1
            required pub ack_status: AckFIDTokenValuesTokenACKsAccessoryInfoTokenAckStatus => { bit: 0, key: "ackStatus", when: (truthy(&true)), predicate_value: false },
            /// Accessory Info Type
            // Virtual schema indices: 2
            required pub acc_info_type: AckFIDTokenValuesTokenACKsAccessoryInfoTokenAccInfoType => { bit: 1, key: "accInfoType", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required ack_status: AckFIDTokenValuesTokenACKsAccessoryInfoTokenAckStatus => { bit: 0, key: "ackStatus", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            2 => required acc_info_type: AckFIDTokenValuesTokenACKsAccessoryInfoTokenAccInfoType => { bit: 1, key: "accInfoType", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum AckFIDTokenValuesTokenACKsIPodPreferenceTokenAckStatus: u8 {
        /// accepted
        Accepted = 0,
        /// required token failed
        RequiredTokenFailed = 1,
        /// optional token failed
        OptionalTokenFailed = 2,
        /// token not supported
        TokenNotSupported = 3,
        /// lingoes busy
        LingoesBusy = 4,
        /// maximum connections reached
        MaximumConnectionsReached = 5,
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum AckFIDTokenValuesTokenACKsIPodPreferenceTokenIPodPrefClass: u8 {
        /// video out setting
        VideoOutSetting = 0,
        /// screen configuration
        ScreenConfiguration = 1,
        /// video signal format
        VideoSignalFormat = 2,
        /// line-out usage
        LineOutUsage = 3,
        /// video-out connection
        VideoOutConnection = 8,
        /// closed captioning
        ClosedCaptioning = 9,
        /// video monitor aspect ratio
        VideoMonitorAspectRatio = 10,
        /// subtitles
        Subtitles = 12,
        /// video alternate audio channel
        VideoAlternateAudioChannel = 13,
        /// pause on power removal
        PauseOnPowerRemoval = 15,
        /// VoiceOver
        VoiceOver = 20,
        /// AssistiveTouch
        AssistiveTouch = 22,
    }
}

iap1_record! {
    strings = LingoString;
    pub struct AckFIDTokenValuesTokenACKsIPodPreferenceToken {
        fields {
            /// Status
            // Virtual schema indices: 1
            required pub ack_status: AckFIDTokenValuesTokenACKsIPodPreferenceTokenAckStatus => { bit: 0, key: "ackStatus", when: (truthy(&true)), predicate_value: false },
            /// iPod Preference Class
            // Virtual schema indices: 2
            required pub i_pod_pref_class: AckFIDTokenValuesTokenACKsIPodPreferenceTokenIPodPrefClass => { bit: 1, key: "iPodPrefClass", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required ack_status: AckFIDTokenValuesTokenACKsIPodPreferenceTokenAckStatus => { bit: 0, key: "ackStatus", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            2 => required i_pod_pref_class: AckFIDTokenValuesTokenACKsIPodPreferenceTokenIPodPrefClass => { bit: 1, key: "iPodPrefClass", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum AckFIDTokenValuesTokenACKsEAProtocolTokenAckStatus: u8 {
        /// accepted
        Accepted = 0,
        /// required token failed
        RequiredTokenFailed = 1,
        /// optional token failed
        OptionalTokenFailed = 2,
        /// token not supported
        TokenNotSupported = 3,
        /// lingoes busy
        LingoesBusy = 4,
        /// maximum connections reached
        MaximumConnectionsReached = 5,
    }
}

iap1_record! {
    strings = LingoString;
    pub struct AckFIDTokenValuesTokenACKsEAProtocolToken {
        fields {
            /// Status
            // Virtual schema indices: 1
            required pub ack_status: AckFIDTokenValuesTokenACKsEAProtocolTokenAckStatus => { bit: 0, key: "ackStatus", when: (truthy(&true)), predicate_value: false },
            /// Protocol Index
            // Virtual schema indices: 2
            required pub protocol_index: u8 => { bit: 1, key: "protocolIndex", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required ack_status: AckFIDTokenValuesTokenACKsEAProtocolTokenAckStatus => { bit: 0, key: "ackStatus", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            2 => required protocol_index: u8 => { bit: 1, key: "protocolIndex", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum AckFIDTokenValuesTokenACKsBundleSeedIDPrefTokenAckStatus: u8 {
        /// accepted
        Accepted = 0,
        /// required token failed
        RequiredTokenFailed = 1,
        /// optional token failed
        OptionalTokenFailed = 2,
        /// token not supported
        TokenNotSupported = 3,
        /// lingoes busy
        LingoesBusy = 4,
        /// maximum connections reached
        MaximumConnectionsReached = 5,
    }
}

iap1_record! {
    strings = LingoString;
    pub struct AckFIDTokenValuesTokenACKsBundleSeedIDPrefToken {
        fields {
            /// Status
            // Virtual schema indices: 1
            required pub ack_status: AckFIDTokenValuesTokenACKsBundleSeedIDPrefTokenAckStatus => { bit: 0, key: "ackStatus", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required ack_status: AckFIDTokenValuesTokenACKsBundleSeedIDPrefTokenAckStatus => { bit: 0, key: "ackStatus", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum AckFIDTokenValuesTokenACKsScreenInfoTokenAckStatus: u8 {
        /// accepted
        Accepted = 0,
        /// required token failed
        RequiredTokenFailed = 1,
        /// optional token failed
        OptionalTokenFailed = 2,
        /// token not supported
        TokenNotSupported = 3,
        /// lingoes busy
        LingoesBusy = 4,
        /// maximum connections reached
        MaximumConnectionsReached = 5,
    }
}

iap1_record! {
    strings = LingoString;
    pub struct AckFIDTokenValuesTokenACKsScreenInfoToken {
        fields {
            /// Status
            // Virtual schema indices: 1
            required pub ack_status: AckFIDTokenValuesTokenACKsScreenInfoTokenAckStatus => { bit: 0, key: "ackStatus", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required ack_status: AckFIDTokenValuesTokenACKsScreenInfoTokenAckStatus => { bit: 0, key: "ackStatus", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum AckFIDTokenValuesTokenACKsEAProtocolMetadataTokenAckStatus: u8 {
        /// accepted
        Accepted = 0,
        /// required token failed
        RequiredTokenFailed = 1,
        /// optional token failed
        OptionalTokenFailed = 2,
        /// token not supported
        TokenNotSupported = 3,
        /// lingoes busy
        LingoesBusy = 4,
        /// maximum connections reached
        MaximumConnectionsReached = 5,
    }
}

iap1_record! {
    strings = LingoString;
    pub struct AckFIDTokenValuesTokenACKsEAProtocolMetadataToken {
        fields {
            /// Status
            // Virtual schema indices: 1
            required pub ack_status: AckFIDTokenValuesTokenACKsEAProtocolMetadataTokenAckStatus => { bit: 0, key: "ackStatus", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required ack_status: AckFIDTokenValuesTokenACKsEAProtocolMetadataTokenAckStatus => { bit: 0, key: "ackStatus", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum AckFIDTokenValuesTokenACKsAccessoryDigitalAudioSampleRatesTokenAckStatus: u8 {
        /// accepted
        Accepted = 0,
        /// required token failed
        RequiredTokenFailed = 1,
        /// optional token failed
        OptionalTokenFailed = 2,
        /// token not supported
        TokenNotSupported = 3,
        /// lingoes busy
        LingoesBusy = 4,
        /// maximum connections reached
        MaximumConnectionsReached = 5,
    }
}

iap1_record! {
    strings = LingoString;
    pub struct AckFIDTokenValuesTokenACKsAccessoryDigitalAudioSampleRatesToken {
        fields {
            /// Status
            // Virtual schema indices: 1
            required pub ack_status: AckFIDTokenValuesTokenACKsAccessoryDigitalAudioSampleRatesTokenAckStatus => { bit: 0, key: "ackStatus", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required ack_status: AckFIDTokenValuesTokenACKsAccessoryDigitalAudioSampleRatesTokenAckStatus => { bit: 0, key: "ackStatus", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum AckFIDTokenValuesTokenACKsAccessoryDigitalAudioVideoDelayTokenAckStatus: u8 {
        /// accepted
        Accepted = 0,
        /// required token failed
        RequiredTokenFailed = 1,
        /// optional token failed
        OptionalTokenFailed = 2,
        /// token not supported
        TokenNotSupported = 3,
        /// lingoes busy
        LingoesBusy = 4,
        /// maximum connections reached
        MaximumConnectionsReached = 5,
    }
}

iap1_record! {
    strings = LingoString;
    pub struct AckFIDTokenValuesTokenACKsAccessoryDigitalAudioVideoDelayToken {
        fields {
            /// Status
            // Virtual schema indices: 1
            required pub ack_status: AckFIDTokenValuesTokenACKsAccessoryDigitalAudioVideoDelayTokenAckStatus => { bit: 0, key: "ackStatus", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required ack_status: AckFIDTokenValuesTokenACKsAccessoryDigitalAudioVideoDelayTokenAckStatus => { bit: 0, key: "ackStatus", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum AckFIDTokenValuesTokenACKsMicrophoneCapsTokenAckStatus: u8 {
        /// accepted
        Accepted = 0,
        /// required token failed
        RequiredTokenFailed = 1,
        /// optional token failed
        OptionalTokenFailed = 2,
        /// token not supported
        TokenNotSupported = 3,
        /// lingoes busy
        LingoesBusy = 4,
        /// maximum connections reached
        MaximumConnectionsReached = 5,
    }
}

iap1_record! {
    strings = LingoString;
    pub struct AckFIDTokenValuesTokenACKsMicrophoneCapsToken {
        fields {
            /// Status
            // Virtual schema indices: 1
            required pub ack_status: AckFIDTokenValuesTokenACKsMicrophoneCapsTokenAckStatus => { bit: 0, key: "ackStatus", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required ack_status: AckFIDTokenValuesTokenACKsMicrophoneCapsTokenAckStatus => { bit: 0, key: "ackStatus", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1_enum_tokens! {
    strings = LingoString;
    pub enum AckFIDTokenValuesTokenACKs {
        #[iap1_token(fid_type = 0, fid_subtype = 0)]
        IdentifyToken(AckFIDTokenValuesTokenACKsIdentifyToken),
        #[iap1_token(fid_type = 0, fid_subtype = 1)]
        AccessoryCapsToken(AckFIDTokenValuesTokenACKsAccessoryCapsToken),
        #[iap1_token(fid_type = 0, fid_subtype = 2)]
        AccessoryInfoToken(AckFIDTokenValuesTokenACKsAccessoryInfoToken),
        #[iap1_token(fid_type = 0, fid_subtype = 3)]
        IPodPreferenceToken(AckFIDTokenValuesTokenACKsIPodPreferenceToken),
        #[iap1_token(fid_type = 0, fid_subtype = 4)]
        EAProtocolToken(AckFIDTokenValuesTokenACKsEAProtocolToken),
        #[iap1_token(fid_type = 0, fid_subtype = 5)]
        BundleSeedIDPrefToken(AckFIDTokenValuesTokenACKsBundleSeedIDPrefToken),
        #[iap1_token(fid_type = 0, fid_subtype = 7)]
        ScreenInfoToken(AckFIDTokenValuesTokenACKsScreenInfoToken),
        #[iap1_token(fid_type = 0, fid_subtype = 8)]
        EAProtocolMetadataToken(AckFIDTokenValuesTokenACKsEAProtocolMetadataToken),
        #[iap1_token(fid_type = 0, fid_subtype = 14)]
        AccessoryDigitalAudioSampleRatesToken(AckFIDTokenValuesTokenACKsAccessoryDigitalAudioSampleRatesToken),
        #[iap1_token(fid_type = 0, fid_subtype = 15)]
        AccessoryDigitalAudioVideoDelayToken(AckFIDTokenValuesTokenACKsAccessoryDigitalAudioVideoDelayToken),
        #[iap1_token(fid_type = 1, fid_subtype = 0)]
        MicrophoneCapsToken(AckFIDTokenValuesTokenACKsMicrophoneCapsToken),
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x00, command = 0x003a,
        source = device,
        response = true,
        ack = false,
        deprecated = false,
        transaction_id = required
    )]
    pub struct AckFIDTokenValues {
        fields {
            /// Number of Token ACKs
            // Virtual schema indices: 1
            required pub num_fid_token_value_ac_ks: u8 => { bit: 0, key: "numFIDTokenValueACKs", when: (truthy(&true)), predicate_value: true },
            /// Token ACKs
            // Virtual schema indices: 2
            required pub token_ac_ks: Vec<AckFIDTokenValuesTokenACKs> => { bit: 1, key: "tokenACKs", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required num_fid_token_value_ac_ks: u8 => { bit: 0, key: "numFIDTokenValueACKs", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: true, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            2 => required token_ac_ks: Vec<AckFIDTokenValuesTokenACKs> => { bit: 1, key: "tokenACKs", wire: [counted(num_fid_token_value_ac_ks)], project_wire: Iap1WireSpec::CountedField(0), predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum EndIDPSAccEndIDPSStatus: u8 {
        /// done
        Done = 0,
        /// clear all IDPS info sent
        ClearAllIDPSInfoSent = 1,
        /// abandoning, iPod missing needed features
        AbandoningIPodMissingNeededFeatures = 2,
        /// done, restarting IDPS on another transport link
        DoneRestartingIDPSOnAnotherTransportLink = 3,
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x00, command = 0x003b,
        source = accessory,
        response = false,
        ack = false,
        deprecated = false,
        transaction_id = required
    )]
    pub struct EndIDPS {
        fields {
            /// Accessory IDPS Status
            // Virtual schema indices: 1
            required pub acc_end_idps_status: EndIDPSAccEndIDPSStatus => { bit: 0, key: "accEndIDPSStatus", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required acc_end_idps_status: EndIDPSAccEndIDPSStatus => { bit: 0, key: "accEndIDPSStatus", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum IDPSStatusStatus: u8 {
        /// all required token-value fields received, proceed to authentication
        AllRequiredTokenValueFieldsReceivedProceedToAuthentication = 0,
        /// required token-value field(s) rejected, IDPS fails
        RequiredTokenValueFieldSRejectedIDPSFails = 1,
        /// required token-value field(s) missing, IDPS fails
        RequiredTokenValueFieldSMissingIDPSFails = 2,
        /// some required token-value field(s) missing and some rejected, IDPS fails
        SomeRequiredTokenValueFieldSMissingAndSomeRejectedIDPSFails = 3,
        /// IDPS time limit not exceeded, retry IDPS or use IdentifyDeviceLingoes
        IDPSTimeLimitNotExceededRetryIDPSOrUseIdentifyDeviceLingoes = 4,
        /// IDPS time limit exceeded, can't retry IDPS, use IdentifyDeviceLingoes
        IDPSTimeLimitExceededCanTRetryIDPSUseIdentifyDeviceLingoes = 5,
        /// IDPS rejected, use IdentifyDeviceLingoes
        IDPSRejectedUseIdentifyDeviceLingoes = 6,
        /// IDPS failed due to inter-token mismatch
        IDPSFailedDueToInterTokenMismatch = 7,
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x00, command = 0x003c,
        source = device,
        response = true,
        ack = false,
        deprecated = false,
        transaction_id = required
    )]
    pub struct IDPSStatus {
        fields {
            /// iPod IDPS Status
            // Virtual schema indices: 1
            required pub status: IDPSStatusStatus => { bit: 0, key: "status", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required status: IDPSStatusStatus => { bit: 0, key: "status", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x00, command = 0x003f,
        source = device,
        response = false,
        ack = false,
        deprecated = false,
        transaction_id = required
    )]
    pub struct OpenDataSessionForProtocol {
        fields {
            /// Session ID
            // Virtual schema indices: 1
            required pub session_id: u16 => { bit: 0, key: "sessionID", when: (truthy(&true)), predicate_value: false },
            /// Protocol Index
            // Virtual schema indices: 2
            required pub protocol_index: u8 => { bit: 1, key: "protocolIndex", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required session_id: u16 => { bit: 0, key: "sessionID", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            2 => required protocol_index: u8 => { bit: 1, key: "protocolIndex", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x00, command = 0x0040,
        source = device,
        response = false,
        ack = false,
        deprecated = false,
        transaction_id = required
    )]
    pub struct CloseDataSession {
        fields {
            /// Session ID
            // Virtual schema indices: 1
            required pub session_id: u16 => { bit: 0, key: "sessionID", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required session_id: u16 => { bit: 0, key: "sessionID", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum AccessoryAckCommandResult: u8 {
        /// OK
        OK = 0,
        /// bad parameter
        BadParameter = 4,
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x00, command = 0x0041,
        source = accessory,
        response = true,
        ack = true,
        deprecated = false,
        transaction_id = required
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
        context = ctx, strings = LingoString, lingo = 0x00, command = 0x0042,
        source = accessory,
        response = false,
        ack = false,
        deprecated = false,
        transaction_id = required
    )]
    pub struct AccessoryDataTransfer {
        fields {
            /// Session ID
            // Virtual schema indices: 1
            required pub session_id: u16 => { bit: 0, key: "sessionID", when: (truthy(&true)), predicate_value: false },
            /// Data the Device Sends
            // Virtual schema indices: 2
            required pub data: Vec<u8> => { bit: 1, key: "data", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required session_id: u16 => { bit: 0, key: "sessionID", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            2 => required data: Vec<u8> => { bit: 1, key: "data", wire: [raw_bytes], project_wire: Iap1WireSpec::Raw, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x00, command = 0x0043,
        source = device,
        response = false,
        ack = false,
        deprecated = false,
        transaction_id = required
    )]
    pub struct IPodDataTransfer {
        fields {
            /// Session ID
            // Virtual schema indices: 1
            required pub session_id: u16 => { bit: 0, key: "sessionID", when: (truthy(&true)), predicate_value: false },
            /// Data the iPod Sends
            // Virtual schema indices: 2
            required pub data: Vec<u8> => { bit: 1, key: "data", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required session_id: u16 => { bit: 0, key: "sessionID", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            2 => required data: Vec<u8> => { bit: 1, key: "data", wire: [raw_bytes], project_wire: Iap1WireSpec::Raw, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1_bitflags! {
    strings = LingoString;
    pub struct SetAccessoryStatusNotificationStatusNotificationsMask: u32 {
        /// Bluetooth device
        const BLUETOOTH_DEVICE = 1 << 1;
        /// fault condition
        const FAULT_CONDITION = 1 << 2;
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x00, command = 0x0046,
        source = device,
        response = false,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct SetAccessoryStatusNotification {
        fields {
            /// Status Notifications Mask
            // Virtual schema indices: 1
            required pub status_notifications_mask: SetAccessoryStatusNotificationStatusNotificationsMask => { bit: 0, key: "statusNotificationsMask", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required status_notifications_mask: SetAccessoryStatusNotificationStatusNotificationsMask => { bit: 0, key: "statusNotificationsMask", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1_bitflags! {
    strings = LingoString;
    pub struct RetAccessoryStatusNotificationStatusNotificationsMask: u32 {
        /// Bluetooth device
        const BLUETOOTH_DEVICE = 1 << 1;
        /// fault condition
        const FAULT_CONDITION = 1 << 2;
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x00, command = 0x0047,
        source = accessory,
        response = true,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct RetAccessoryStatusNotification {
        fields {
            /// Status Notifications Mask
            // Virtual schema indices: 1
            required pub status_notifications_mask: RetAccessoryStatusNotificationStatusNotificationsMask => { bit: 0, key: "statusNotificationsMask", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required status_notifications_mask: RetAccessoryStatusNotificationStatusNotificationsMask => { bit: 0, key: "statusNotificationsMask", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum AccessoryStatusNotificationStatusType: u8 {
        /// Bluetooth
        Bluetooth = 1,
        /// fault
        Fault = 2,
    }
}

iap1_bitflags! {
    strings = LingoString;
    pub struct AccessoryStatusNotificationBluetoothDevicesDevType: u32 {
        /// classic
        const CLASSIC = 1 << 0;
    }
}

iap1_bitflags! {
    strings = LingoString;
    pub struct AccessoryStatusNotificationBluetoothDevicesDevState: u32 {
        /// on
        const ON = 1 << 0;
    }
}

iap1_bitflags! {
    strings = LingoString;
    pub struct AccessoryStatusNotificationBluetoothDevicesPairingType: u8 {
        /// none
        const NONE = 1 << 0;
        /// PIN code
        const PIN_CODE = 1 << 1;
        /// Secure Simple Pairing (SSP)
        const SECURE_SIMPLE_PAIRING_SSP = 1 << 2;
    }
}

iap1_record! {
    strings = LingoString;
    pub struct AccessoryStatusNotificationBluetoothDevices {
        fields {
            /// Device Type
            // Virtual schema indices: 1
            required pub dev_type: AccessoryStatusNotificationBluetoothDevicesDevType => { bit: 0, key: "devType", when: (truthy(&true)), predicate_value: false },
            /// Device State
            // Virtual schema indices: 2
            required pub dev_state: AccessoryStatusNotificationBluetoothDevicesDevState => { bit: 1, key: "devState", when: (truthy(&true)), predicate_value: false },
            /// Device Class
            // Virtual schema indices: 3
            required pub dev_class: u32 => { bit: 2, key: "devClass", when: (truthy(&true)), predicate_value: false },
            /// Pairing Type
            // Virtual schema indices: 4
            required pub pairing_type: AccessoryStatusNotificationBluetoothDevicesPairingType => { bit: 3, key: "pairingType", when: (truthy(&true)), predicate_value: false },
            /// Reserved
            // Virtual schema indices: 5
            required pub address_type: u8 => { bit: 4, key: "addressType", when: (truthy(&true)), predicate_value: false },
            /// PIN Code
            // Virtual schema indices: 6
            required pub pin_code: String => { bit: 5, key: "pinCode", when: (truthy(&true)), predicate_value: false },
            /// MAC Address
            // Virtual schema indices: 7
            required pub mac_address: [u8; 6] => { bit: 6, key: "macAddress", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required dev_type: AccessoryStatusNotificationBluetoothDevicesDevType => { bit: 0, key: "devType", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            2 => required dev_state: AccessoryStatusNotificationBluetoothDevicesDevState => { bit: 1, key: "devState", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            3 => required dev_class: u32 => { bit: 2, key: "devClass", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            4 => required pairing_type: AccessoryStatusNotificationBluetoothDevicesPairingType => { bit: 3, key: "pairingType", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            5 => required address_type: u8 => { bit: 4, key: "addressType", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            6 => required pin_code: String => { bit: 5, key: "pinCode", wire: [utf8(16usize)], project_wire: Iap1WireSpec::CountedLiteral(16), predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            7 => required mac_address: [u8; 6] => { bit: 6, key: "macAddress", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum AccessoryStatusNotificationFaultType: u8 {
        /// no fault
        NoFault = 0,
        /// voltage fault
        VoltageFault = 1,
        /// current fault
        CurrentFault = 2,
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum AccessoryStatusNotificationFaultCondition: u8 {
        /// no fault
        NoFault = 0,
        /// recoverable
        Recoverable = 1,
        /// not recoverable
        NotRecoverable = 2,
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x00, command = 0x0048,
        source = accessory,
        response = false,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct AccessoryStatusNotification {
        fields {
            /// Status Type
            // Virtual schema indices: 1
            required pub status_type: AccessoryStatusNotificationStatusType => { bit: 0, key: "statusType", when: (truthy(&true)), predicate_value: true },
            /// Bluetooth Devices
            // Virtual schema indices: 2
            optional pub bluetooth_devices: Vec<AccessoryStatusNotificationBluetoothDevices> => { bit: 1, key: "bluetoothDevices", when: (eq(&status_type, &1u64)), predicate_value: false },
            /// Fault Type
            // Virtual schema indices: 3
            optional pub fault_type: AccessoryStatusNotificationFaultType => { bit: 2, key: "faultType", when: (eq(&status_type, &2u64)), predicate_value: false },
            /// Fault Condition
            // Virtual schema indices: 4
            optional pub fault_condition: AccessoryStatusNotificationFaultCondition => { bit: 3, key: "faultCondition", when: (eq(&status_type, &2u64)), predicate_value: false },
        }
        steps {
            1 => required status_type: AccessoryStatusNotificationStatusType => { bit: 0, key: "statusType", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: true, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            2 => optional bluetooth_devices: Vec<AccessoryStatusNotificationBluetoothDevices> => { bit: 1, key: "bluetoothDevices", wire: [counted_remaining], project_wire: Iap1WireSpec::CountedRemaining, predicate_value: false, predicate: { expr: eq(&status_type, &1u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(1u8)] } },
            3 => optional fault_type: AccessoryStatusNotificationFaultType => { bit: 2, key: "faultType", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&status_type, &2u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(2u8)] } },
            4 => optional fault_condition: AccessoryStatusNotificationFaultCondition => { bit: 3, key: "faultCondition", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&status_type, &2u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(2u8)] } },
        }
    }
}

iap1_bitflags! {
    strings = LingoString;
    pub struct SetEventNotificationEventNotificationMask: u64 {
        /// Flow Control
        /// Required by the specification.
        const FLOW_CONTROL = 1 << 2;
        /// Radio Tagging
        const RADIO_TAGGING = 1 << 3;
        /// Camera Notifications
        const CAMERA_NOTIFICATIONS = 1 << 4;
        /// Charging Info
        const CHARGING_INFO = 1 << 5;
        /// Database Changed
        const DATABASE_CHANGED = 1 << 9;
        /// Now Playing App Bundle Name
        const NOW_PLAYING_APP_BUNDLE_NAME = 1 << 10;
        /// Session Space Available
        const SESSION_SPACE_AVAILABLE = 1 << 11;
        /// Database Available
        #[deprecated]
        const DATABASE_AVAILABLE = 1 << 12;
        /// Command Complete
        const COMMAND_COMPLETE = 1 << 13;
        /// iPod Out Mode Status
        const I_POD_OUT_MODE_STATUS = 1 << 15;
        /// Bluetooth Connection Status
        const BLUETOOTH_CONNECTION_STATUS = 1 << 17;
        /// Now Playing Application Display Name
        const NOW_PLAYING_APPLICATION_DISPLAY_NAME = 1 << 19;
        /// Assistive Touch status
        const ASSISTIVE_TOUCH_STATUS = 1 << 20;
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x00, command = 0x0049,
        source = accessory,
        response = false,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct SetEventNotification {
        fields {
            /// Event Notification Mask
            // Virtual schema indices: 1
            required pub event_notification_mask: SetEventNotificationEventNotificationMask => { bit: 0, key: "eventNotificationMask", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required event_notification_mask: SetEventNotificationEventNotificationMask => { bit: 0, key: "eventNotificationMask", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum IPodNotificationNotificationType: u8 {
        /// Flow Control
        FlowControl = 2,
        /// Radio Tagging
        RadioTagging = 3,
        /// Camera Notifications
        CameraNotifications = 4,
        /// Charging Info
        ChargingInfo = 5,
        /// Database Changed
        DatabaseChanged = 9,
        /// Now Playing App Bundle Name
        NowPlayingAppBundleName = 10,
        /// Session Space Available
        SessionSpaceAvailable = 11,
        /// Database Available
        #[deprecated]
        DatabaseAvailable = 12,
        /// Command Complete
        CommandComplete = 13,
        /// iPod Out Mode Status
        IPodOutModeStatus = 15,
        /// Bluetooth Connection Status
        BluetoothConnectionStatus = 17,
        /// Now Playing Application Display Name
        NowPlayingApplicationDisplayName = 19,
        /// Assistive Touch status
        AssistiveTouchStatus = 20,
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum IPodNotificationTagStatus: u8 {
        /// tagging operation successful
        TaggingOperationSuccessful = 0,
        /// tagging operation failed
        TaggingOperationFailed = 1,
        /// info available
        InfoAvailable = 2,
        /// info not available
        InfoNotAvailable = 3,
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum IPodNotificationCameraStatus: u8 {
        /// camera app off
        CameraAppOff = 0,
        /// preview
        Preview = 3,
        /// recording
        Recording = 4,
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum IPodNotificationChargingInfoType: u8 {
        /// available current
        AvailableCurrent = 0,
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum IPodNotificationDatabaseAvailableStatus: u8 {
        /// not available
        NotAvailable = 0,
        /// available
        Available = 1,
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum IPodNotificationCommandCompleteStatus: u8 {
        /// successful
        Successful = 0,
        /// failed
        Failed = 1,
        /// cancelled
        Cancelled = 2,
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum IPodNotificationStatus: u8 {
        /// iPod Out Mode inactive
        IPodOutModeInactive = 0,
        /// iPod Out Mode active
        IPodOutModeActive = 1,
    }
}

iap1_enum_open! {
    strings = LingoString;
    #[iap1_enum_open(storage = u8, ranges = [1..=255])]
    /// Permitted range 1..=255: on
    pub enum IPodNotificationAssistiveTouchStatus {
        /// off
        Off = 0,
    }
}

iap1_disjoint! {
    strings = LingoString;
    #[iap1_disjoint(predicate = integer)]
    pub enum IPodNotificationStatusDisjoint {
        #[iap1_disjoint(decode = decode_status, encode = encode_status)]
        /// Status
        Status(IPodNotificationStatus),
        #[iap1_disjoint(decode = decode_assistive_touch_status, encode = encode_assistive_touch_status)]
        /// AssistiveTouch Status
        AssistiveTouchStatus(IPodNotificationAssistiveTouchStatus),
    }
}

iap1_bitflags! {
    strings = LingoString;
    pub struct IPodNotificationProfileBits: u64 {
        /// Hands Free Profile (HFP)
        const HANDS_FREE_PROFILE_HFP = 1 << 0;
        /// Phone Book Access Profile (PBAP)
        const PHONE_BOOK_ACCESS_PROFILE_PBAP = 1 << 1;
        /// Audio/Video Remote Control Profile (AVRCP)
        const AUDIO_VIDEO_REMOTE_CONTROL_PROFILE_AVRCP = 1 << 3;
        /// Advanced Audio Distribution Profile (A2DP)
        const ADVANCED_AUDIO_DISTRIBUTION_PROFILE_A2DP = 1 << 4;
        /// Human Interface Device Profile (HID)
        const HUMAN_INTERFACE_DEVICE_PROFILE_HID = 1 << 5;
        /// Apple iAP Profile
        const APPLE_I_AP_PROFILE = 1 << 7;
        /// Personal Area Network-Network Access Point Profile (PAN-NAP or Network Sharing)
        const PERSONAL_AREA_NETWORK_NETWORK_ACCESS_POINT_PROFILE_PAN_NAP_OR_NETWORK_SHARING = 1 << 8;
        /// Personal Area Network-User Profile (PAN-U or Network Consumer)
        const PERSONAL_AREA_NETWORK_USER_PROFILE_PAN_U_OR_NETWORK_CONSUMER = 1 << 12;
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x00, command = 0x004a,
        source = device,
        response = false,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct IPodNotification {
        fields {
            /// Notification Type
            // Virtual schema indices: 1
            required pub notification_type: IPodNotificationNotificationType => { bit: 0, key: "notificationType", when: (truthy(&true)), predicate_value: true },
            /// Wait Time
            // Virtual schema indices: 2
            optional pub wait_time: u32 => { bit: 1, key: "waitTime", when: (eq(&notification_type, &2u64)), predicate_value: false },
            /// Overflow Transaction ID
            // Virtual schema indices: 3
            optional pub overflow_trans_id: u16 => { bit: 2, key: "overflowTransID", when: ((eq(&notification_type, &2u64)) && (eq(&ctx.has_transaction_id, &true))), predicate_value: false },
            /// Tag Status
            // Virtual schema indices: 4
            optional pub tag_status: IPodNotificationTagStatus => { bit: 3, key: "tagStatus", when: (eq(&notification_type, &3u64)), predicate_value: false },
            /// Camera Status
            // Virtual schema indices: 5
            optional pub camera_status: IPodNotificationCameraStatus => { bit: 4, key: "cameraStatus", when: (eq(&notification_type, &4u64)), predicate_value: false },
            /// Charging Info Type
            // Virtual schema indices: 6
            optional pub charging_info_type: IPodNotificationChargingInfoType => { bit: 5, key: "chargingInfoType", when: (eq(&notification_type, &5u64)), predicate_value: true },
            /// Available Current
            // Virtual schema indices: 7
            optional pub available_current: u16 => { bit: 6, key: "availableCurrent", when: ((eq(&notification_type, &5u64)) && (eq(&charging_info_type, &0u64))), predicate_value: false },
            /// Bundle Seed ID
            // Virtual schema indices: 8
            optional pub bundle_seed_id: String => { bit: 7, key: "bundleSeedID", when: (eq(&notification_type, &10u64)), predicate_value: false },
            /// Session ID
            // Virtual schema indices: 9
            optional pub session_id: u16 => { bit: 8, key: "sessionID", when: (eq(&notification_type, &11u64)), predicate_value: false },
            /// Database Available Status
            // Virtual schema indices: 10
            optional pub database_available_status: IPodNotificationDatabaseAvailableStatus => { bit: 9, key: "databaseAvailableStatus", when: (eq(&notification_type, &12u64)), predicate_value: false },
            /// Completed Lingo ID
            // Virtual schema indices: 11
            optional pub completed_lingo_id: u8 => { bit: 10, key: "completedLingoID", when: (eq(&notification_type, &13u64)), predicate_value: false },
            /// Completed Command ID
            // Virtual schema indices: 12
            optional pub completed_command_id: u16 => { bit: 11, key: "completedCommandID", when: (eq(&notification_type, &13u64)), predicate_value: false },
            /// Command Complete Status
            // Virtual schema indices: 13
            optional pub command_complete_status: IPodNotificationCommandCompleteStatus => { bit: 12, key: "commandCompleteStatus", when: (eq(&notification_type, &13u64)), predicate_value: false },
            /// Status
            // Virtual schema indices: 14, 18
            optional pub status: IPodNotificationStatusDisjoint => { bit: 13, key: "status", when: (eq(&notification_type, &15u64)) || (eq(&notification_type, &20u64)), predicate_value: false },
            /// MAC Address
            // Virtual schema indices: 15
            optional pub mac_address: [u8; 6] => { bit: 14, key: "macAddress", when: (eq(&notification_type, &17u64)), predicate_value: false },
            /// Profile Bits
            // Virtual schema indices: 16
            optional pub profile_bits: IPodNotificationProfileBits => { bit: 15, key: "profileBits", when: (eq(&notification_type, &17u64)), predicate_value: false },
            /// Application Name
            // Virtual schema indices: 17
            optional pub app_name: String => { bit: 16, key: "appName", when: (eq(&notification_type, &19u64)), predicate_value: false },
        }
        steps {
            1 => required notification_type: IPodNotificationNotificationType => { bit: 0, key: "notificationType", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: true, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            2 => optional wait_time: u32 => { bit: 1, key: "waitTime", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&notification_type, &2u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(2u8)] } },
            3 => optional overflow_trans_id: u16 => { bit: 2, key: "overflowTransID", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: (eq(&notification_type, &2u64)) && (eq(&ctx.has_transaction_id, &true)), program: [FlatPredicateToken::And, FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(2u8), FlatPredicateToken::Eq, FlatPredicateToken::HasTransactionId, FlatPredicateToken::Bool(true)] } },
            4 => optional tag_status: IPodNotificationTagStatus => { bit: 3, key: "tagStatus", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&notification_type, &3u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(3u8)] } },
            5 => optional camera_status: IPodNotificationCameraStatus => { bit: 4, key: "cameraStatus", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&notification_type, &4u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(4u8)] } },
            6 => optional charging_info_type: IPodNotificationChargingInfoType => { bit: 5, key: "chargingInfoType", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: true, predicate: { expr: eq(&notification_type, &5u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(5u8)] } },
            7 => optional available_current: u16 => { bit: 6, key: "availableCurrent", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: (eq(&notification_type, &5u64)) && (eq(&charging_info_type, &0u64)), program: [FlatPredicateToken::And, FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(5u8), FlatPredicateToken::Eq, FlatPredicateToken::Field(5), FlatPredicateToken::Integer(0u8)] } },
            8 => optional bundle_seed_id: String => { bit: 7, key: "bundleSeedID", wire: [raw_string], project_wire: Iap1WireSpec::Raw, predicate_value: false, predicate: { expr: eq(&notification_type, &10u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(10u8)] } },
            9 => optional session_id: u16 => { bit: 8, key: "sessionID", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&notification_type, &11u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(11u8)] } },
            10 => optional database_available_status: IPodNotificationDatabaseAvailableStatus => { bit: 9, key: "databaseAvailableStatus", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&notification_type, &12u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(12u8)] } },
            11 => optional completed_lingo_id: u8 => { bit: 10, key: "completedLingoID", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&notification_type, &13u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(13u8)] } },
            12 => optional completed_command_id: u16 => { bit: 11, key: "completedCommandID", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&notification_type, &13u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(13u8)] } },
            13 => optional command_complete_status: IPodNotificationCommandCompleteStatus => { bit: 12, key: "commandCompleteStatus", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&notification_type, &13u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(13u8)] } },
            14 => optional status: IPodNotificationStatusDisjoint => { bit: 13, key: "status", wire: [disjoint(decode_status, encode_status, IPodNotificationStatus, [scalar])], project_wire: Iap1WireSpec::Disjoint { variant: const { LingoString::require_raw(stringify!(decode_status)) }, child: &Iap1WireSpec::Scalar }, predicate_value: false, predicate: { expr: eq(&notification_type, &15u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(15u8)] } },
            15 => optional mac_address: [u8; 6] => { bit: 14, key: "macAddress", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&notification_type, &17u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(17u8)] } },
            16 => optional profile_bits: IPodNotificationProfileBits => { bit: 15, key: "profileBits", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&notification_type, &17u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(17u8)] } },
            17 => optional app_name: String => { bit: 16, key: "appName", wire: [raw_string], project_wire: Iap1WireSpec::Raw, predicate_value: false, predicate: { expr: eq(&notification_type, &19u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(19u8)] } },
            18 => optional status: IPodNotificationStatusDisjoint => { bit: 13, key: "status", wire: [disjoint(decode_assistive_touch_status, encode_assistive_touch_status, IPodNotificationAssistiveTouchStatus, [scalar])], project_wire: Iap1WireSpec::Disjoint { variant: const { LingoString::require_raw(stringify!(decode_assistive_touch_status)) }, child: &Iap1WireSpec::Scalar }, predicate_value: false, predicate: { expr: eq(&notification_type, &20u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(20u8)] } },
        }
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x00, command = 0x004b,
        source = accessory,
        response = false,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct GetiPodOptionsForLingo {
        fields {
            /// Lingo Requested
            // Virtual schema indices: 1
            required pub lingo_id: u8 => { bit: 0, key: "lingoID", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required lingo_id: u8 => { bit: 0, key: "lingoID", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1_bitflags! {
    strings = LingoString;
    pub struct RetiPodOptionsForLingoGeneralOptions: u64 {
        /// line out usage
        const LINE_OUT_USAGE = 1 << 0;
        /// video output
        const VIDEO_OUTPUT = 1 << 1;
        /// NTSC video signal format
        const NTSC_VIDEO_SIGNAL_FORMAT = 1 << 2;
        /// PAL video signal format
        const PAL_VIDEO_SIGNAL_FORMAT = 1 << 3;
        /// composite video out connection
        const COMPOSITE_VIDEO_OUT_CONNECTION = 1 << 4;
        /// S-Video video out connection
        const S_VIDEO_VIDEO_OUT_CONNECTION = 1 << 5;
        /// component video out connection
        const COMPONENT_VIDEO_OUT_CONNECTION = 1 << 6;
        /// closed captioning (video)
        const CLOSED_CAPTIONING_VIDEO = 1 << 7;
        /// video aspect ratio 4:3 (fullscreen)
        const VIDEO_ASPECT_RATIO_4_3_FULLSCREEN = 1 << 8;
        /// video aspect ratio 16:9 (widescreen)
        const VIDEO_ASPECT_RATIO_16_9_WIDESCREEN = 1 << 9;
        /// subtitles (video)
        const SUBTITLES_VIDEO = 1 << 10;
        /// video alternate audio channel
        const VIDEO_ALTERNATE_AUDIO_CHANNEL = 1 << 11;
        /// communication with iOS applications
        const COMMUNICATION_WITH_I_OS_APPLICATIONS = 1 << 13;
        /// iPod notifications
        const I_POD_NOTIFICATIONS = 1 << 14;
        /// pause on power removal preference control
        const PAUSE_ON_POWER_REMOVAL_PREFERENCE_CONTROL = 1 << 19;
        /// restricted IDPS tokens
        const RESTRICTED_IDPS_TOKENS = 1 << 23;
        /// request application launch
        const REQUEST_APPLICATION_LAUNCH = 1 << 24;
        /// analog USB audio routing
        const ANALOG_USB_AUDIO_ROUTING = 1 << 26;
        /// USB Host Mode capable
        const USB_HOST_MODE_CAPABLE = 1 << 27;
        /// USB Host Mode supports output to USB audio devices
        const USB_HOST_MODE_SUPPORTS_OUTPUT_TO_USB_AUDIO_DEVICES = 1 << 28;
        /// USB Host Mode supports input from USB audio devices
        const USB_HOST_MODE_SUPPORTS_INPUT_FROM_USB_AUDIO_DEVICES = 1 << 29;
        /// USB Host Mode does not support maximum current draw in Low Power Mode
        const USB_HOST_MODE_DOES_NOT_SUPPORT_MAXIMUM_CURRENT_DRAW_IN_LOW_POWER_MODE = 1 << 30;
    }
}

iap1_bitflags! {
    strings = LingoString;
    pub struct RetiPodOptionsForLingoMicrophoneOptions: u64 {
        /// analog recording capable
        const ANALOG_RECORDING_CAPABLE = 1 << 3;
        /// USB digital recording capable
        const USB_DIGITAL_RECORDING_CAPABLE = 1 << 4;
    }
}

iap1_bitflags! {
    strings = LingoString;
    pub struct RetiPodOptionsForLingoSimpleRemoteOptions: u64 {
        /// context-specific controls
        const CONTEXT_SPECIFIC_CONTROLS = 1 << 0;
        /// audio media controls
        const AUDIO_MEDIA_CONTROLS = 1 << 1;
        /// video media controls
        const VIDEO_MEDIA_CONTROLS = 1 << 2;
        /// image media controls
        const IMAGE_MEDIA_CONTROLS = 1 << 3;
        /// sports media controls
        const SPORTS_MEDIA_CONTROLS = 1 << 4;
        /// camera media controls
        const CAMERA_MEDIA_CONTROLS = 1 << 8;
        /// USB HID commands
        const USB_HID_COMMANDS = 1 << 9;
        /// VoiceOver controls
        const VOICE_OVER_CONTROLS = 1 << 10;
        /// VoiceOver preferences
        const VOICE_OVER_PREFERENCES = 1 << 11;
        /// AssistiveTouch
        const ASSISTIVE_TOUCH = 1 << 12;
    }
}

iap1_bitflags! {
    strings = LingoString;
    pub struct RetiPodOptionsForLingoDisplayRemoteOptions: u64 {
        /// UI volume control
        const UI_VOLUME_CONTROL = 1 << 0;
        /// absolute volume control
        const ABSOLUTE_VOLUME_CONTROL = 1 << 1;
        /// genius playlist creation
        const GENIUS_PLAYLIST_CREATION = 1 << 2;
    }
}

iap1_bitflags! {
    strings = LingoString;
    pub struct RetiPodOptionsForLingoExtendedInterfaceOptions: u64 {
        /// video browsing
        const VIDEO_BROWSING = 1 << 0;
        /// Extended Interface enhancements
        const EXTENDED_INTERFACE_ENHANCEMENTS = 1 << 1;
        /// nested playlists
        const NESTED_PLAYLISTS = 1 << 2;
        /// genius playlist creation
        const GENIUS_PLAYLIST_CREATION = 1 << 3;
        /// supports SetDisplayImage
        const SUPPORTS_SET_DISPLAY_IMAGE = 1 << 4;
        /// category discovery
        const CATEGORY_DISCOVERY = 1 << 5;
        /// supports PlayControl Play and Pause
        const SUPPORTS_PLAY_CONTROL_PLAY_AND_PAUSE = 1 << 6;
        /// supports UID-based commands
        const SUPPORTS_UID_BASED_COMMANDS = 1 << 7;
    }
}

iap1_bitflags! {
    strings = LingoString;
    pub struct RetiPodOptionsForLingoAccessoryPowerOptions: u64 {
    }
}

iap1_bitflags! {
    strings = LingoString;
    pub struct RetiPodOptionsForLingoUSBHostModeOptions: u64 {
        /// mode invoked by hardware
        const MODE_INVOKED_BY_HARDWARE = 1 << 0;
        /// mode invoked by firmware
        const MODE_INVOKED_BY_FIRMWARE = 1 << 1;
    }
}

iap1_bitflags! {
    strings = LingoString;
    pub struct RetiPodOptionsForLingoRFTunerOptions: u64 {
        /// RDS raw mode support
        const RDS_RAW_MODE_SUPPORT = 1 << 0;
        /// HD radio tuning support
        const HD_RADIO_TUNING_SUPPORT = 1 << 1;
        /// AM radio tuning support
        const AM_RADIO_TUNING_SUPPORT = 1 << 2;
    }
}

iap1_bitflags! {
    strings = LingoString;
    pub struct RetiPodOptionsForLingoAccessoryEqualizerOptions: u64 {
    }
}

iap1_bitflags! {
    strings = LingoString;
    pub struct RetiPodOptionsForLingoSportsOptions: u64 {
        /// Nike + iPod cardio equipment
        const NIKE_I_POD_CARDIO_EQUIPMENT = 1 << 1;
    }
}

iap1_bitflags! {
    strings = LingoString;
    pub struct RetiPodOptionsForLingoDigitalAudioOptions: u64 {
        /// A/V synchronization
        const A_V_SYNCHRONIZATION = 1 << 0;
        /// supports AccessoryDigitalAudioSampleRatesToken
        const SUPPORTS_ACCESSORY_DIGITAL_AUDIO_SAMPLE_RATES_TOKEN = 1 << 2;
        /// supports AccessoryDigitalAudioVideoDelayToken
        const SUPPORTS_ACCESSORY_DIGITAL_AUDIO_VIDEO_DELAY_TOKEN = 1 << 3;
    }
}

iap1_bitflags! {
    strings = LingoString;
    pub struct RetiPodOptionsForLingoTestOptions: u64 {
    }
}

iap1_bitflags! {
    strings = LingoString;
    pub struct RetiPodOptionsForLingoStorageOptions: u64 {
        /// iTunes tagging
        const I_TUNES_TAGGING = 1 << 0;
        /// Nike + iPod cardio equipment
        const NIKE_I_POD_CARDIO_EQUIPMENT = 1 << 1;
    }
}

iap1_bitflags! {
    strings = LingoString;
    pub struct RetiPodOptionsForLingoIPodOutOptions: u64 {
        /// iPod Out available
        const I_POD_OUT_AVAILABLE = 1 << 0;
        /// PAL video available
        const PAL_VIDEO_AVAILABLE = 1 << 2;
    }
}

iap1_bitflags! {
    strings = LingoString;
    pub struct RetiPodOptionsForLingoLocationOptions: u64 {
        /// iPod accepts NMEA GPS location data
        const I_POD_ACCEPTS_NMEA_GPS_LOCATION_DATA = 1 << 0;
        /// iPod can send location assistance data
        const I_POD_CAN_SEND_LOCATION_ASSISTANCE_DATA = 1 << 1;
    }
}

iap1_bitflags! {
    strings = LingoString;
    pub struct RetiPodOptionsForLingoUnknownLingoOptions: u64 {
    }
}

iap1_disjoint! {
    strings = LingoString;
    #[iap1_disjoint(predicate = integer)]
    pub enum RetiPodOptionsForLingoOptionBitsDisjoint {
        #[iap1_disjoint(decode = decode_general_options, encode = encode_general_options)]
        /// General Options
        GeneralOptions(RetiPodOptionsForLingoGeneralOptions),
        #[iap1_disjoint(decode = decode_microphone_options, encode = encode_microphone_options)]
        /// Microphone Options
        MicrophoneOptions(RetiPodOptionsForLingoMicrophoneOptions),
        #[iap1_disjoint(decode = decode_simple_remote_options, encode = encode_simple_remote_options)]
        /// Simple Remote Options
        SimpleRemoteOptions(RetiPodOptionsForLingoSimpleRemoteOptions),
        #[iap1_disjoint(decode = decode_display_remote_options, encode = encode_display_remote_options)]
        /// Display Remote Options
        DisplayRemoteOptions(RetiPodOptionsForLingoDisplayRemoteOptions),
        #[iap1_disjoint(decode = decode_extended_interface_options, encode = encode_extended_interface_options)]
        /// Extended Interface Options
        ExtendedInterfaceOptions(RetiPodOptionsForLingoExtendedInterfaceOptions),
        #[iap1_disjoint(decode = decode_accessory_power_options, encode = encode_accessory_power_options)]
        /// Accessory Power Options
        AccessoryPowerOptions(RetiPodOptionsForLingoAccessoryPowerOptions),
        #[iap1_disjoint(decode = decode_usb_host_mode_options, encode = encode_usb_host_mode_options)]
        /// USB Host Mode Options
        USBHostModeOptions(RetiPodOptionsForLingoUSBHostModeOptions),
        #[iap1_disjoint(decode = decode_rf_tuner_options, encode = encode_rf_tuner_options)]
        /// RF Tuner Options
        RFTunerOptions(RetiPodOptionsForLingoRFTunerOptions),
        #[iap1_disjoint(decode = decode_accessory_equalizer_options, encode = encode_accessory_equalizer_options)]
        /// Accessory Equalizer Options
        AccessoryEqualizerOptions(RetiPodOptionsForLingoAccessoryEqualizerOptions),
        #[iap1_disjoint(decode = decode_sports_options, encode = encode_sports_options)]
        /// Sports Options
        SportsOptions(RetiPodOptionsForLingoSportsOptions),
        #[iap1_disjoint(decode = decode_digital_audio_options, encode = encode_digital_audio_options)]
        /// Digital Audio Options
        DigitalAudioOptions(RetiPodOptionsForLingoDigitalAudioOptions),
        #[iap1_disjoint(decode = decode_test_options, encode = encode_test_options)]
        /// Test Options
        TestOptions(RetiPodOptionsForLingoTestOptions),
        #[iap1_disjoint(decode = decode_storage_options, encode = encode_storage_options)]
        /// Storage Options
        StorageOptions(RetiPodOptionsForLingoStorageOptions),
        #[iap1_disjoint(decode = decode_i_pod_out_options, encode = encode_i_pod_out_options)]
        /// iPod Out Options
        IPodOutOptions(RetiPodOptionsForLingoIPodOutOptions),
        #[iap1_disjoint(decode = decode_location_options, encode = encode_location_options)]
        /// Location Options
        LocationOptions(RetiPodOptionsForLingoLocationOptions),
        #[iap1_disjoint(decode = decode_unknown_lingo_options, encode = encode_unknown_lingo_options)]
        /// Unknown Lingo Options
        UnknownLingoOptions(RetiPodOptionsForLingoUnknownLingoOptions),
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x00, command = 0x004c,
        source = device,
        response = true,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct RetiPodOptionsForLingo {
        fields {
            /// Lingo Returned
            // Virtual schema indices: 1
            required pub lingo_id: u8 => { bit: 0, key: "lingoID", when: (truthy(&true)), predicate_value: true },
            /// General Options
            // Virtual schema indices: 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17
            required pub option_bits: RetiPodOptionsForLingoOptionBitsDisjoint => { bit: 1, key: "optionBits", when: (eq(&lingo_id, &0u64)) || (eq(&lingo_id, &1u64)) || (eq(&lingo_id, &2u64)) || (eq(&lingo_id, &3u64)) || (eq(&lingo_id, &4u64)) || (eq(&lingo_id, &5u64)) || (eq(&lingo_id, &6u64)) || (eq(&lingo_id, &7u64)) || (eq(&lingo_id, &8u64)) || (eq(&lingo_id, &9u64)) || (eq(&lingo_id, &10u64)) || (eq(&lingo_id, &11u64)) || (eq(&lingo_id, &12u64)) || (eq(&lingo_id, &13u64)) || (eq(&lingo_id, &14u64)) || (gt(&lingo_id, &14u64)), predicate_value: false },
        }
        steps {
            1 => required lingo_id: u8 => { bit: 0, key: "lingoID", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: true, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            2 => required option_bits: RetiPodOptionsForLingoOptionBitsDisjoint => { bit: 1, key: "optionBits", wire: [disjoint(decode_general_options, encode_general_options, RetiPodOptionsForLingoGeneralOptions, [scalar])], project_wire: Iap1WireSpec::Disjoint { variant: const { LingoString::require_raw(stringify!(decode_general_options)) }, child: &Iap1WireSpec::Scalar }, predicate_value: false, predicate: { expr: eq(&lingo_id, &0u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(0u8)] } },
            3 => required option_bits: RetiPodOptionsForLingoOptionBitsDisjoint => { bit: 1, key: "optionBits", wire: [disjoint(decode_microphone_options, encode_microphone_options, RetiPodOptionsForLingoMicrophoneOptions, [scalar])], project_wire: Iap1WireSpec::Disjoint { variant: const { LingoString::require_raw(stringify!(decode_microphone_options)) }, child: &Iap1WireSpec::Scalar }, predicate_value: false, predicate: { expr: eq(&lingo_id, &1u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(1u8)] } },
            4 => required option_bits: RetiPodOptionsForLingoOptionBitsDisjoint => { bit: 1, key: "optionBits", wire: [disjoint(decode_simple_remote_options, encode_simple_remote_options, RetiPodOptionsForLingoSimpleRemoteOptions, [scalar])], project_wire: Iap1WireSpec::Disjoint { variant: const { LingoString::require_raw(stringify!(decode_simple_remote_options)) }, child: &Iap1WireSpec::Scalar }, predicate_value: false, predicate: { expr: eq(&lingo_id, &2u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(2u8)] } },
            5 => required option_bits: RetiPodOptionsForLingoOptionBitsDisjoint => { bit: 1, key: "optionBits", wire: [disjoint(decode_display_remote_options, encode_display_remote_options, RetiPodOptionsForLingoDisplayRemoteOptions, [scalar])], project_wire: Iap1WireSpec::Disjoint { variant: const { LingoString::require_raw(stringify!(decode_display_remote_options)) }, child: &Iap1WireSpec::Scalar }, predicate_value: false, predicate: { expr: eq(&lingo_id, &3u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(3u8)] } },
            6 => required option_bits: RetiPodOptionsForLingoOptionBitsDisjoint => { bit: 1, key: "optionBits", wire: [disjoint(decode_extended_interface_options, encode_extended_interface_options, RetiPodOptionsForLingoExtendedInterfaceOptions, [scalar])], project_wire: Iap1WireSpec::Disjoint { variant: const { LingoString::require_raw(stringify!(decode_extended_interface_options)) }, child: &Iap1WireSpec::Scalar }, predicate_value: false, predicate: { expr: eq(&lingo_id, &4u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(4u8)] } },
            7 => required option_bits: RetiPodOptionsForLingoOptionBitsDisjoint => { bit: 1, key: "optionBits", wire: [disjoint(decode_accessory_power_options, encode_accessory_power_options, RetiPodOptionsForLingoAccessoryPowerOptions, [scalar])], project_wire: Iap1WireSpec::Disjoint { variant: const { LingoString::require_raw(stringify!(decode_accessory_power_options)) }, child: &Iap1WireSpec::Scalar }, predicate_value: false, predicate: { expr: eq(&lingo_id, &5u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(5u8)] } },
            8 => required option_bits: RetiPodOptionsForLingoOptionBitsDisjoint => { bit: 1, key: "optionBits", wire: [disjoint(decode_usb_host_mode_options, encode_usb_host_mode_options, RetiPodOptionsForLingoUSBHostModeOptions, [scalar])], project_wire: Iap1WireSpec::Disjoint { variant: const { LingoString::require_raw(stringify!(decode_usb_host_mode_options)) }, child: &Iap1WireSpec::Scalar }, predicate_value: false, predicate: { expr: eq(&lingo_id, &6u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(6u8)] } },
            9 => required option_bits: RetiPodOptionsForLingoOptionBitsDisjoint => { bit: 1, key: "optionBits", wire: [disjoint(decode_rf_tuner_options, encode_rf_tuner_options, RetiPodOptionsForLingoRFTunerOptions, [scalar])], project_wire: Iap1WireSpec::Disjoint { variant: const { LingoString::require_raw(stringify!(decode_rf_tuner_options)) }, child: &Iap1WireSpec::Scalar }, predicate_value: false, predicate: { expr: eq(&lingo_id, &7u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(7u8)] } },
            10 => required option_bits: RetiPodOptionsForLingoOptionBitsDisjoint => { bit: 1, key: "optionBits", wire: [disjoint(decode_accessory_equalizer_options, encode_accessory_equalizer_options, RetiPodOptionsForLingoAccessoryEqualizerOptions, [scalar])], project_wire: Iap1WireSpec::Disjoint { variant: const { LingoString::require_raw(stringify!(decode_accessory_equalizer_options)) }, child: &Iap1WireSpec::Scalar }, predicate_value: false, predicate: { expr: eq(&lingo_id, &8u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(8u8)] } },
            11 => required option_bits: RetiPodOptionsForLingoOptionBitsDisjoint => { bit: 1, key: "optionBits", wire: [disjoint(decode_sports_options, encode_sports_options, RetiPodOptionsForLingoSportsOptions, [scalar])], project_wire: Iap1WireSpec::Disjoint { variant: const { LingoString::require_raw(stringify!(decode_sports_options)) }, child: &Iap1WireSpec::Scalar }, predicate_value: false, predicate: { expr: eq(&lingo_id, &9u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(9u8)] } },
            12 => required option_bits: RetiPodOptionsForLingoOptionBitsDisjoint => { bit: 1, key: "optionBits", wire: [disjoint(decode_digital_audio_options, encode_digital_audio_options, RetiPodOptionsForLingoDigitalAudioOptions, [scalar])], project_wire: Iap1WireSpec::Disjoint { variant: const { LingoString::require_raw(stringify!(decode_digital_audio_options)) }, child: &Iap1WireSpec::Scalar }, predicate_value: false, predicate: { expr: eq(&lingo_id, &10u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(10u8)] } },
            13 => required option_bits: RetiPodOptionsForLingoOptionBitsDisjoint => { bit: 1, key: "optionBits", wire: [disjoint(decode_test_options, encode_test_options, RetiPodOptionsForLingoTestOptions, [scalar])], project_wire: Iap1WireSpec::Disjoint { variant: const { LingoString::require_raw(stringify!(decode_test_options)) }, child: &Iap1WireSpec::Scalar }, predicate_value: false, predicate: { expr: eq(&lingo_id, &11u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(11u8)] } },
            14 => required option_bits: RetiPodOptionsForLingoOptionBitsDisjoint => { bit: 1, key: "optionBits", wire: [disjoint(decode_storage_options, encode_storage_options, RetiPodOptionsForLingoStorageOptions, [scalar])], project_wire: Iap1WireSpec::Disjoint { variant: const { LingoString::require_raw(stringify!(decode_storage_options)) }, child: &Iap1WireSpec::Scalar }, predicate_value: false, predicate: { expr: eq(&lingo_id, &12u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(12u8)] } },
            15 => required option_bits: RetiPodOptionsForLingoOptionBitsDisjoint => { bit: 1, key: "optionBits", wire: [disjoint(decode_i_pod_out_options, encode_i_pod_out_options, RetiPodOptionsForLingoIPodOutOptions, [scalar])], project_wire: Iap1WireSpec::Disjoint { variant: const { LingoString::require_raw(stringify!(decode_i_pod_out_options)) }, child: &Iap1WireSpec::Scalar }, predicate_value: false, predicate: { expr: eq(&lingo_id, &13u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(13u8)] } },
            16 => required option_bits: RetiPodOptionsForLingoOptionBitsDisjoint => { bit: 1, key: "optionBits", wire: [disjoint(decode_location_options, encode_location_options, RetiPodOptionsForLingoLocationOptions, [scalar])], project_wire: Iap1WireSpec::Disjoint { variant: const { LingoString::require_raw(stringify!(decode_location_options)) }, child: &Iap1WireSpec::Scalar }, predicate_value: false, predicate: { expr: eq(&lingo_id, &14u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(14u8)] } },
            17 => required option_bits: RetiPodOptionsForLingoOptionBitsDisjoint => { bit: 1, key: "optionBits", wire: [disjoint(decode_unknown_lingo_options, encode_unknown_lingo_options, RetiPodOptionsForLingoUnknownLingoOptions, [scalar])], project_wire: Iap1WireSpec::Disjoint { variant: const { LingoString::require_raw(stringify!(decode_unknown_lingo_options)) }, child: &Iap1WireSpec::Scalar }, predicate_value: false, predicate: { expr: gt(&lingo_id, &14u64), program: [FlatPredicateToken::Gt, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(14u8)] } },
        }
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x00, command = 0x004d,
        source = accessory,
        response = false,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct GetEventNotification {
        fields {
        }
        steps {
        }
    }
}

iap1_bitflags! {
    strings = LingoString;
    pub struct RetEventNotificationEventNotificationMask: u64 {
        /// Flow Control
        const FLOW_CONTROL = 1 << 2;
        /// Radio Tagging Status
        const RADIO_TAGGING_STATUS = 1 << 3;
        /// Camera Notifications
        const CAMERA_NOTIFICATIONS = 1 << 4;
        /// Charging Info
        const CHARGING_INFO = 1 << 5;
        /// Database Changed
        const DATABASE_CHANGED = 1 << 9;
        /// Now Playing App Bundle Name
        const NOW_PLAYING_APP_BUNDLE_NAME = 1 << 10;
        /// Session Space Available
        const SESSION_SPACE_AVAILABLE = 1 << 11;
        /// Database Available
        #[deprecated]
        const DATABASE_AVAILABLE = 1 << 12;
        /// Command Complete
        const COMMAND_COMPLETE = 1 << 13;
        /// iPod Out Mode Status
        const I_POD_OUT_MODE_STATUS = 1 << 15;
        /// Bluetooth Connection Status
        const BLUETOOTH_CONNECTION_STATUS = 1 << 17;
        /// Now Playing Application Display Name
        const NOW_PLAYING_APPLICATION_DISPLAY_NAME = 1 << 19;
        /// Assistive Touch status
        const ASSISTIVE_TOUCH_STATUS = 1 << 20;
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x00, command = 0x004e,
        source = device,
        response = true,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct RetEventNotification {
        fields {
            /// Event Notification Mask
            // Virtual schema indices: 1
            required pub event_notification_mask: RetEventNotificationEventNotificationMask => { bit: 0, key: "eventNotificationMask", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required event_notification_mask: RetEventNotificationEventNotificationMask => { bit: 0, key: "eventNotificationMask", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x00, command = 0x004f,
        source = accessory,
        response = false,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct GetSupportedEventNotification {
        fields {
        }
        steps {
        }
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x00, command = 0x0050,
        source = accessory,
        response = false,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct CancelCommand {
        fields {
            /// Cancelled Lingo ID
            // Virtual schema indices: 1
            required pub cancelled_lingo_id: u8 => { bit: 0, key: "cancelledLingoID", when: (truthy(&true)), predicate_value: false },
            /// Cancelled Command ID
            // Virtual schema indices: 2
            required pub cancelled_command_id: u16 => { bit: 1, key: "cancelledCommandID", when: (truthy(&true)), predicate_value: false },
            /// Cancelled Transaction ID
            // Virtual schema indices: 3
            required pub cancelled_transaction_id: u16 => { bit: 2, key: "cancelledTransactionID", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required cancelled_lingo_id: u8 => { bit: 0, key: "cancelledLingoID", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            2 => required cancelled_command_id: u16 => { bit: 1, key: "cancelledCommandID", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            3 => required cancelled_transaction_id: u16 => { bit: 2, key: "cancelledTransactionID", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1_bitflags! {
    strings = LingoString;
    pub struct RetSupportedEventNotificationEventNotificationMask: u64 {
        /// Flow Control
        const FLOW_CONTROL = 1 << 2;
        /// Radio Tagging Status
        const RADIO_TAGGING_STATUS = 1 << 3;
        /// Camera Notifications
        const CAMERA_NOTIFICATIONS = 1 << 4;
        /// Charging Info
        const CHARGING_INFO = 1 << 5;
        /// Database Changed
        const DATABASE_CHANGED = 1 << 9;
        /// Now Playing App Bundle Name
        const NOW_PLAYING_APP_BUNDLE_NAME = 1 << 10;
        /// Session Space Available
        const SESSION_SPACE_AVAILABLE = 1 << 11;
        /// Database Available
        #[deprecated]
        const DATABASE_AVAILABLE = 1 << 12;
        /// Command Complete
        const COMMAND_COMPLETE = 1 << 13;
        /// iPod Out Mode Status
        const I_POD_OUT_MODE_STATUS = 1 << 15;
        /// Bluetooth Connection Status
        const BLUETOOTH_CONNECTION_STATUS = 1 << 17;
        /// Now Playing Application Display Name
        const NOW_PLAYING_APPLICATION_DISPLAY_NAME = 1 << 19;
        /// Assistive Touch status
        const ASSISTIVE_TOUCH_STATUS = 1 << 20;
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x00, command = 0x0051,
        source = device,
        response = true,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct RetSupportedEventNotification {
        fields {
            /// Event Notification Mask
            // Virtual schema indices: 1
            required pub event_notification_mask: RetSupportedEventNotificationEventNotificationMask => { bit: 0, key: "eventNotificationMask", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required event_notification_mask: RetSupportedEventNotificationEventNotificationMask => { bit: 0, key: "eventNotificationMask", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x00, command = 0x0054,
        source = accessory,
        response = false,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct SetAvailableCurrent {
        fields {
            /// Current Limit
            // Virtual schema indices: 1
            required pub current_limit: u16 => { bit: 0, key: "currentLimit", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required current_limit: u16 => { bit: 0, key: "currentLimit", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum SetInternalBatteryChargingStateChargingState: u8 {
        /// Apple device should stop charging
        AppleDeviceShouldStopCharging = 0,
        /// Apple device may charge
        AppleDeviceMayCharge = 1,
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x00, command = 0x0056,
        source = accessory,
        response = false,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct SetInternalBatteryChargingState {
        fields {
            /// Charging State
            // Virtual schema indices: 1
            required pub charging_state: SetInternalBatteryChargingStateChargingState => { bit: 0, key: "chargingState", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required charging_state: SetInternalBatteryChargingStateChargingState => { bit: 0, key: "chargingState", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum RequestApplicationLaunchLaunchMethod: u8 {
        /// without alert
        WithoutAlert = 1,
        /// with alert
        WithAlert = 2,
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x00, command = 0x0064,
        source = accessory,
        response = false,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct RequestApplicationLaunch {
        fields {
            /// Reserved
            // Virtual schema indices: 1
            required pub reserved1: u8 => { bit: 0, key: "reserved1", when: (truthy(&true)), predicate_value: false },
            /// Launch Method
            // Virtual schema indices: 2
            required pub launch_method: RequestApplicationLaunchLaunchMethod => { bit: 1, key: "launchMethod", when: (truthy(&true)), predicate_value: false },
            /// Reserved
            // Virtual schema indices: 3
            required pub reserved3: u8 => { bit: 2, key: "reserved3", when: (truthy(&true)), predicate_value: false },
            /// Bundle Seed ID
            // Virtual schema indices: 4
            required pub bundle_seed_id: String => { bit: 3, key: "bundleSeedID", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required reserved1: u8 => { bit: 0, key: "reserved1", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            2 => required launch_method: RequestApplicationLaunchLaunchMethod => { bit: 1, key: "launchMethod", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            3 => required reserved3: u8 => { bit: 2, key: "reserved3", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            4 => required bundle_seed_id: String => { bit: 3, key: "bundleSeedID", wire: [raw_string], project_wire: Iap1WireSpec::Raw, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x00, command = 0x0065,
        source = accessory,
        response = false,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct GetNowPlayingApplicationBundleName {
        fields {
        }
        steps {
        }
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x00, command = 0x0066,
        source = device,
        response = true,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct RetNowPlayingApplicationBundleName {
        fields {
            /// Application ID
            // Virtual schema indices: 1
            required pub application_id: String => { bit: 0, key: "applicationID", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required application_id: String => { bit: 0, key: "applicationID", wire: [raw_string], project_wire: Iap1WireSpec::Raw, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum GetLocalizationInfoType: u8 {
        /// language
        Language = 0,
        /// region
        Region = 1,
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x00, command = 0x0067,
        source = accessory,
        response = false,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct GetLocalizationInfo {
        fields {
            /// Type
            // Virtual schema indices: 1
            required pub r#type: GetLocalizationInfoType => { bit: 0, key: "type", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required r#type: GetLocalizationInfoType => { bit: 0, key: "type", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum RetLocalizationInfoType: u8 {
        /// language
        Language = 0,
        /// region
        Region = 1,
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x00, command = 0x0068,
        source = device,
        response = true,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct RetLocalizationInfo {
        fields {
            /// Type
            // Virtual schema indices: 1
            required pub r#type: RetLocalizationInfoType => { bit: 0, key: "type", when: (truthy(&true)), predicate_value: false },
            /// Tag
            // Virtual schema indices: 2
            required pub tag: String => { bit: 1, key: "tag", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required r#type: RetLocalizationInfoType => { bit: 0, key: "type", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            2 => required tag: String => { bit: 1, key: "tag", wire: [raw_string], project_wire: Iap1WireSpec::Raw, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x00, command = 0x0069,
        source = accessory,
        response = false,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct RequestWiFiConnectionInfo {
        fields {
        }
        steps {
        }
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum WiFiConnectionInfoStatus: u8 {
        /// success
        Success = 0,
        /// connection information unavailable
        ConnectionInformationUnavailable = 1,
        /// user declined
        UserDeclined = 2,
        /// command failed
        CommandFailed = 3,
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum WiFiConnectionInfoSecurity: u8 {
        /// unsecured
        Unsecured = 0,
        /// WEP
        WEP = 1,
        /// WPA
        WPA = 2,
        /// WPA2
        WPA2 = 3,
        /// mixed WPA and WPA2
        MixedWPAAndWPA2 = 4,
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x00, command = 0x006a,
        source = device,
        response = true,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct WiFiConnectionInfo {
        fields {
            /// Status
            // Virtual schema indices: 1
            required pub status: WiFiConnectionInfoStatus => { bit: 0, key: "status", when: (truthy(&true)), predicate_value: true },
            /// Security
            // Virtual schema indices: 2
            optional pub security: WiFiConnectionInfoSecurity => { bit: 1, key: "security", when: (eq(&status, &0u64)), predicate_value: true },
            /// SSID
            // Virtual schema indices: 3
            optional pub ssid: [u8; 32] => { bit: 2, key: "ssid", when: (eq(&status, &0u64)), predicate_value: false },
            /// Passphrase
            // Virtual schema indices: 4
            optional pub passphrase: String => { bit: 3, key: "passphrase", when: ((eq(&status, &0u64)) && (ne(&security, &0u64))), predicate_value: false },
        }
        steps {
            1 => required status: WiFiConnectionInfoStatus => { bit: 0, key: "status", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: true, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            2 => optional security: WiFiConnectionInfoSecurity => { bit: 1, key: "security", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: true, predicate: { expr: eq(&status, &0u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(0u8)] } },
            3 => optional ssid: [u8; 32] => { bit: 2, key: "ssid", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&status, &0u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(0u8)] } },
            4 => optional passphrase: String => { bit: 3, key: "passphrase", wire: [raw_string], project_wire: Iap1WireSpec::Raw, predicate_value: false, predicate: { expr: (eq(&status, &0u64)) && (ne(&security, &0u64)), program: [FlatPredicateToken::And, FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(0u8), FlatPredicateToken::Ne, FlatPredicateToken::Field(1), FlatPredicateToken::Integer(0u8)] } },
        }
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x00, command = 0x00ee,
        source = accessory,
        response = false,
        ack = false,
        deprecated = false,
        transaction_id = prohibited
    )]
    pub struct DetectSequence {
        fields {
        }
        steps {
        }
    }
}

iap1_registry! {
    0x00;
    RequestIdentify => 0x0000,
    Identify => 0x0001,
    IPodAck => 0x0002,
    RequestExtendedInterfaceMode => 0x0003,
    ReturnExtendedInterfaceMode => 0x0004,
    EnterExtendedInterfaceMode => 0x0005,
    ExitExtendedInterfaceMode => 0x0006,
    RequestiPodName => 0x0007,
    ReturniPodName => 0x0008,
    RequestiPodSoftwareVersion => 0x0009,
    ReturniPodSoftwareVersion => 0x000a,
    RequestiPodSerialNum => 0x000b,
    ReturniPodSerialNum => 0x000c,
    RequestiPodModelNum => 0x000d,
    ReturniPodModelNum => 0x000e,
    RequestLingoProtocolVersion => 0x000f,
    ReturnLingoProtocolVersion => 0x0010,
    RequestTransportMaxPayloadSize => 0x0011,
    ReturnTransportMaxPayloadSize => 0x0012,
    IdentifyDeviceLingoes => 0x0013,
    GetAccessoryAuthenticationInfo => 0x0014,
    RetAccessoryAuthenticationInfo => 0x0015,
    AckAccessoryAuthenticationInfo => 0x0016,
    GetAccessoryAuthenticationSignature => 0x0017,
    RetAccessoryAuthenticationSignature => 0x0018,
    AckAccessoryAuthenticationStatus => 0x0019,
    GetiPodAuthenticationInfo => 0x001a,
    RetiPodAuthenticationInfo => 0x001b,
    AckiPodAuthenticationInfo => 0x001c,
    GetiPodAuthenticationSignature => 0x001d,
    RetiPodAuthenticationSignature => 0x001e,
    AckiPodAuthenticationStatus => 0x001f,
    NotifyiPodStateChange => 0x0023,
    GetiPodOptions => 0x0024,
    RetiPodOptions => 0x0025,
    GetAccessoryInfo => 0x0027,
    RetAccessoryInfo => 0x0028,
    GetiPodPreferences => 0x0029,
    RetiPodPreferences => 0x002a,
    SetiPodPreferences => 0x002b,
    GetUIMode => 0x0035,
    RetUIMode => 0x0036,
    SetUIMode => 0x0037,
    StartIDPS => 0x0038,
    SetFIDTokenValues => 0x0039,
    AckFIDTokenValues => 0x003a,
    EndIDPS => 0x003b,
    IDPSStatus => 0x003c,
    OpenDataSessionForProtocol => 0x003f,
    CloseDataSession => 0x0040,
    AccessoryAck => 0x0041,
    AccessoryDataTransfer => 0x0042,
    IPodDataTransfer => 0x0043,
    SetAccessoryStatusNotification => 0x0046,
    RetAccessoryStatusNotification => 0x0047,
    AccessoryStatusNotification => 0x0048,
    SetEventNotification => 0x0049,
    IPodNotification => 0x004a,
    GetiPodOptionsForLingo => 0x004b,
    RetiPodOptionsForLingo => 0x004c,
    GetEventNotification => 0x004d,
    RetEventNotification => 0x004e,
    GetSupportedEventNotification => 0x004f,
    CancelCommand => 0x0050,
    RetSupportedEventNotification => 0x0051,
    SetAvailableCurrent => 0x0054,
    SetInternalBatteryChargingState => 0x0056,
    RequestApplicationLaunch => 0x0064,
    GetNowPlayingApplicationBundleName => 0x0065,
    RetNowPlayingApplicationBundleName => 0x0066,
    GetLocalizationInfo => 0x0067,
    RetLocalizationInfo => 0x0068,
    RequestWiFiConnectionInfo => 0x0069,
    WiFiConnectionInfo => 0x006a,
    DetectSequence => 0x00ee,
}
