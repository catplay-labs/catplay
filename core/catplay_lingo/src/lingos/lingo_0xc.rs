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

// Lingo 0x0c: Storage
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
        context = ctx, strings = LingoString, lingo = 0x0c, command = 0x0000,
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
            /// File Handle
            // Virtual schema indices: 6
            required pub file_handle: u8 => { bit: 5, key: "fileHandle", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required command_result: IPodAckCommandResult => { bit: 0, key: "commandResult", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: true, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            2 => required acked_command_id: u8 => { bit: 1, key: "ackedCommandID", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            3 => optional maximum_pending_wait: u32 => { bit: 2, key: "maximumPendingWait", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&command_result, &6u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(6u8)] } },
            4 => optional session_id: u16 => { bit: 3, key: "sessionID", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&command_result, &23u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(23u8)] } },
            5 => optional num_bytes_dropped: u32 => { bit: 4, key: "numBytesDropped", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&command_result, &23u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(23u8)] } },
            6 => required file_handle: u8 => { bit: 5, key: "fileHandle", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x0c, command = 0x0001,
        source = accessory,
        response = false,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct GetiPodCaps {
        fields {
        }
        steps {
        }
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x0c, command = 0x0002,
        source = device,
        response = true,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct RetiPodCaps {
        fields {
            /// Total Space
            // Virtual schema indices: 1
            required pub total_space: u64 => { bit: 0, key: "totalSpace", when: (truthy(&true)), predicate_value: false },
            /// Maximum File Size
            // Virtual schema indices: 2
            required pub max_file_size: u32 => { bit: 1, key: "maxFileSize", when: (truthy(&true)), predicate_value: false },
            /// Maximum Write Size
            // Virtual schema indices: 3
            required pub max_write_size: u16 => { bit: 2, key: "maxWriteSize", when: (truthy(&true)), predicate_value: false },
            /// Reserved
            // Virtual schema indices: 4
            required pub reserved_bytes: [u8; 6] => { bit: 3, key: "reservedBytes", when: (truthy(&true)), predicate_value: false },
            /// Major Version
            // Virtual schema indices: 5
            required pub major_version: u8 => { bit: 4, key: "majorVersion", when: (truthy(&true)), predicate_value: false },
            /// Minor Version
            // Virtual schema indices: 6
            required pub minor_version: u8 => { bit: 5, key: "minorVersion", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required total_space: u64 => { bit: 0, key: "totalSpace", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            2 => required max_file_size: u32 => { bit: 1, key: "maxFileSize", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            3 => required max_write_size: u16 => { bit: 2, key: "maxWriteSize", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            4 => required reserved_bytes: [u8; 6] => { bit: 3, key: "reservedBytes", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            5 => required major_version: u8 => { bit: 4, key: "majorVersion", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            6 => required minor_version: u8 => { bit: 5, key: "minorVersion", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x0c, command = 0x0004,
        source = device,
        response = true,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct RetiPodFileHandle {
        fields {
            /// File Handle
            // Virtual schema indices: 1
            required pub file_handle: u8 => { bit: 0, key: "fileHandle", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required file_handle: u8 => { bit: 0, key: "fileHandle", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x0c, command = 0x0007,
        source = accessory,
        response = false,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct WriteiPodFileData {
        fields {
            /// Write Offset
            // Virtual schema indices: 1
            required pub write_offset: u32 => { bit: 0, key: "writeOffset", when: (truthy(&true)), predicate_value: false },
            /// File Handle
            // Virtual schema indices: 2
            required pub file_handle: u8 => { bit: 1, key: "fileHandle", when: (truthy(&true)), predicate_value: false },
            /// File Data
            // Virtual schema indices: 3
            required pub file_data: Vec<u8> => { bit: 2, key: "fileData", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required write_offset: u32 => { bit: 0, key: "writeOffset", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            2 => required file_handle: u8 => { bit: 1, key: "fileHandle", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            3 => required file_data: Vec<u8> => { bit: 2, key: "fileData", wire: [raw_bytes], project_wire: Iap1WireSpec::Raw, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x0c, command = 0x0008,
        source = accessory,
        response = false,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct CloseiPodFile {
        fields {
            /// fileHandle
            // Virtual schema indices: 1
            required pub file_handle: u8 => { bit: 0, key: "fileHandle", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required file_handle: u8 => { bit: 0, key: "fileHandle", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x0c, command = 0x0010,
        source = accessory,
        response = false,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct GetiPodFreeSpace {
        fields {
        }
        steps {
        }
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x0c, command = 0x0011,
        source = device,
        response = true,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct RetiPodFreeSpace {
        fields {
            /// Free Space
            // Virtual schema indices: 1
            required pub free_space: u64 => { bit: 0, key: "freeSpace", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required free_space: u64 => { bit: 0, key: "freeSpace", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum OpeniPodFeatureFileFeatureType: u8 {
        /// radio tagging
        RadioTagging = 1,
        /// cardio equipment workout
        #[deprecated]
        CardioEquipmentWorkout = 2,
    }
}

iap1_bitflags! {
    strings = LingoString;
    pub struct OpeniPodFeatureFileFileOptionsMask: u32 {
        /// append data on close or detach
        const APPEND_DATA_ON_CLOSE_OR_DETACH = 1 << 0;
        /// append ipodInfo element on close or detach
        const APPEND_IPOD_INFO_ELEMENT_ON_CLOSE_OR_DETACH = 1 << 1;
        /// insert Signature element on close or detach
        const INSERT_SIGNATURE_ELEMENT_ON_CLOSE_OR_DETACH = 1 << 3;
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x0c, command = 0x0012,
        source = accessory,
        response = false,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct OpeniPodFeatureFile {
        fields {
            /// Feature Type
            // Virtual schema indices: 1
            required pub feature_type: OpeniPodFeatureFileFeatureType => { bit: 0, key: "featureType", when: (truthy(&true)), predicate_value: true },
            /// File Options
            // Virtual schema indices: 2
            optional pub file_options_mask: OpeniPodFeatureFileFileOptionsMask => { bit: 1, key: "fileOptionsMask", when: (ne(&feature_type, &1u64)), predicate_value: false },
            /// File Data Appended at Close
            // Virtual schema indices: 3
            optional pub file_data_appended_at_close: Vec<u8> => { bit: 2, key: "fileDataAppendedAtClose", when: (ne(&feature_type, &1u64)), predicate_value: false },
        }
        steps {
            1 => required feature_type: OpeniPodFeatureFileFeatureType => { bit: 0, key: "featureType", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: true, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            2 => optional file_options_mask: OpeniPodFeatureFileFileOptionsMask => { bit: 1, key: "fileOptionsMask", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: ne(&feature_type, &1u64), program: [FlatPredicateToken::Ne, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(1u8)] } },
            3 => optional file_data_appended_at_close: Vec<u8> => { bit: 2, key: "fileDataAppendedAtClose", wire: [raw_bytes], project_wire: Iap1WireSpec::Raw, predicate_value: false, predicate: { expr: ne(&feature_type, &1u64), program: [FlatPredicateToken::Ne, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(1u8)] } },
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
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x0c, command = 0x0080,
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
            /// File Handle
            // Virtual schema indices: 3
            required pub file_handle: u8 => { bit: 2, key: "fileHandle", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required command_result: AccessoryAckCommandResult => { bit: 0, key: "commandResult", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            2 => required acked_command_id: u8 => { bit: 1, key: "ackedCommandID", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            3 => required file_handle: u8 => { bit: 2, key: "fileHandle", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x0c, command = 0x0081,
        source = device,
        response = false,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct GetAccessoryCaps {
        fields {
        }
        steps {
        }
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x0c, command = 0x0082,
        source = accessory,
        response = true,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct RetAccessoryCaps {
        fields {
            /// Reserved
            // Virtual schema indices: 1
            required pub reserved_bytes1: [u8; 19] => { bit: 0, key: "reservedBytes1", when: (truthy(&true)), predicate_value: false },
            /// Reserved
            // Virtual schema indices: 2
            required pub reserved_bytes2: u8 => { bit: 1, key: "reservedBytes2", when: (truthy(&true)), predicate_value: false },
            /// Major Storage Lingo Protocol Version
            // Virtual schema indices: 3
            required pub major_storage_lingo_protocol_version: u8 => { bit: 2, key: "majorStorageLingoProtocolVersion", when: (truthy(&true)), predicate_value: false },
            /// Minor Storage Lingo Protocol Version
            // Virtual schema indices: 4
            required pub minor_storage_lingo_protocol_version: u8 => { bit: 3, key: "minorStorageLingoProtocolVersion", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required reserved_bytes1: [u8; 19] => { bit: 0, key: "reservedBytes1", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            2 => required reserved_bytes2: u8 => { bit: 1, key: "reservedBytes2", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            3 => required major_storage_lingo_protocol_version: u8 => { bit: 2, key: "majorStorageLingoProtocolVersion", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            4 => required minor_storage_lingo_protocol_version: u8 => { bit: 3, key: "minorStorageLingoProtocolVersion", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1_registry! {
    0x0c;
    IPodAck => 0x0000,
    GetiPodCaps => 0x0001,
    RetiPodCaps => 0x0002,
    RetiPodFileHandle => 0x0004,
    WriteiPodFileData => 0x0007,
    CloseiPodFile => 0x0008,
    GetiPodFreeSpace => 0x0010,
    RetiPodFreeSpace => 0x0011,
    OpeniPodFeatureFile => 0x0012,
    AccessoryAck => 0x0080,
    GetAccessoryCaps => 0x0081,
    RetAccessoryCaps => 0x0082,
}
