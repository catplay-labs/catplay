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

// Lingo 0x09: Sports
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
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x09, command = 0x0000,
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

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x09, command = 0x0001,
        source = device,
        response = false,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct GetAccessoryVersion {
        fields {
        }
        steps {
        }
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x09, command = 0x0002,
        source = accessory,
        response = true,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct RetAccessoryVersion {
        fields {
            /// Sports Lingo Major Version Supported
            // Virtual schema indices: 1
            required pub major_sports_lingo_protocol_version: u8 => { bit: 0, key: "majorSportsLingoProtocolVersion", when: (truthy(&true)), predicate_value: false },
            /// Sports Lingo Minor version supported
            // Virtual schema indices: 2
            required pub minor_sports_lingo_protocol_version: u8 => { bit: 1, key: "minorSportsLingoProtocolVersion", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required major_sports_lingo_protocol_version: u8 => { bit: 0, key: "majorSportsLingoProtocolVersion", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            2 => required minor_sports_lingo_protocol_version: u8 => { bit: 1, key: "minorSportsLingoProtocolVersion", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x09, command = 0x0003,
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

iap1_bitflags! {
    strings = LingoString;
    pub struct RetAccessoryCapsCapsMask: u16 {
        /// accessory supports Sports lingo commands
        const ACCESSORY_SUPPORTS_SPORTS_LINGO_COMMANDS = 1 << 9;
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x09, command = 0x0004,
        source = accessory,
        response = true,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct RetAccessoryCaps {
        fields {
            /// Capabilities
            // Virtual schema indices: 1
            required pub caps_mask: RetAccessoryCapsCapsMask => { bit: 0, key: "capsMask", when: (truthy(&true)), predicate_value: false },
            /// Reserved
            // Virtual schema indices: 2
            required pub reserved1: u8 => { bit: 1, key: "reserved1", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required caps_mask: RetAccessoryCapsCapsMask => { bit: 0, key: "capsMask", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            2 => required reserved1: u8 => { bit: 1, key: "reserved1", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
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
        context = ctx, strings = LingoString, lingo = 0x09, command = 0x0080,
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
        context = ctx, strings = LingoString, lingo = 0x09, command = 0x0083,
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

iap1_bitflags! {
    strings = LingoString;
    pub struct RetiPodCapsCapsMask: u16 {
        /// cardio equipment
        const CARDIO_EQUIPMENT = 1 << 0;
        /// user data
        const USER_DATA = 1 << 1;
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x09, command = 0x0084,
        source = device,
        response = true,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct RetiPodCaps {
        fields {
            /// Capabilities
            // Virtual schema indices: 1
            required pub caps_mask: RetiPodCapsCapsMask => { bit: 0, key: "capsMask", when: (truthy(&true)), predicate_value: false },
            /// Number of User Data Profiles on iPod
            // Virtual schema indices: 2
            required pub user_count: u8 => { bit: 1, key: "userCount", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required caps_mask: RetiPodCapsCapsMask => { bit: 0, key: "capsMask", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            2 => required user_count: u8 => { bit: 1, key: "userCount", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x09, command = 0x0085,
        source = accessory,
        response = false,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct GetUserIndex {
        fields {
        }
        steps {
        }
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x09, command = 0x0086,
        source = device,
        response = true,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct RetUserIndex {
        fields {
            /// User Index
            // Virtual schema indices: 1
            required pub user_index: u8 => { bit: 0, key: "userIndex", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required user_index: u8 => { bit: 0, key: "userIndex", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum GetUserDataUserDataType: u8 {
        /// preferred unit system
        PreferredUnitSystem = 0,
        /// name
        Name = 1,
        /// gender
        Gender = 2,
        /// weight
        Weight = 3,
        /// age
        Age = 4,
        /// workout recording preference
        WorkoutRecordingPreference = 5,
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x09, command = 0x0088,
        source = accessory,
        response = false,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct GetUserData {
        fields {
            /// User Data Type
            // Virtual schema indices: 1
            required pub user_data_type: GetUserDataUserDataType => { bit: 0, key: "userDataType", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required user_data_type: GetUserDataUserDataType => { bit: 0, key: "userDataType", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum RetUserDataUserDataType: u8 {
        /// preferred unit system
        PreferredUnitSystem = 0,
        /// name
        Name = 1,
        /// gender
        Gender = 2,
        /// weight
        Weight = 3,
        /// age
        Age = 4,
        /// workout recording preference
        WorkoutRecordingPreference = 5,
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum RetUserDataPreferredUnitSystem: u8 {
        /// no information
        NoInformation = 0,
        /// imperial units (lbs/miles)
        ImperialUnitsLbsMiles = 1,
        /// metric units (kg/km)
        MetricUnitsKgKm = 2,
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum RetUserDataGender: u8 {
        /// no information
        NoInformation = 0,
        /// female
        Female = 1,
        /// male
        Male = 2,
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum RetUserDataWorkoutRecordingPreference: u8 {
        /// no information
        NoInformation = 0,
        /// never record workout data
        NeverRecordWorkoutData = 1,
        /// ask if workout should be recorded
        AskIfWorkoutShouldBeRecorded = 2,
        /// always record workout data
        AlwaysRecordWorkoutData = 3,
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x09, command = 0x0089,
        source = device,
        response = true,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct RetUserData {
        fields {
            /// User Data Type
            // Virtual schema indices: 1
            required pub user_data_type: RetUserDataUserDataType => { bit: 0, key: "userDataType", when: (truthy(&true)), predicate_value: true },
            /// Preferred Unit System
            // Virtual schema indices: 2
            optional pub preferred_unit_system: RetUserDataPreferredUnitSystem => { bit: 1, key: "preferredUnitSystem", when: (eq(&user_data_type, &0u64)), predicate_value: false },
            /// Name
            // Virtual schema indices: 3
            optional pub name: String => { bit: 2, key: "name", when: (eq(&user_data_type, &1u64)), predicate_value: false },
            /// Gender
            // Virtual schema indices: 4
            optional pub gender: RetUserDataGender => { bit: 3, key: "gender", when: (eq(&user_data_type, &2u64)), predicate_value: false },
            /// Weight
            // Virtual schema indices: 5
            optional pub weight: u16 => { bit: 4, key: "weight", when: (eq(&user_data_type, &3u64)), predicate_value: false },
            /// Age
            // Virtual schema indices: 6
            optional pub age: u8 => { bit: 5, key: "age", when: (eq(&user_data_type, &4u64)), predicate_value: false },
            /// Workout Recording Preference
            // Virtual schema indices: 7
            optional pub workout_recording_preference: RetUserDataWorkoutRecordingPreference => { bit: 6, key: "workoutRecordingPreference", when: (eq(&user_data_type, &5u64)), predicate_value: false },
        }
        steps {
            1 => required user_data_type: RetUserDataUserDataType => { bit: 0, key: "userDataType", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: true, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            2 => optional preferred_unit_system: RetUserDataPreferredUnitSystem => { bit: 1, key: "preferredUnitSystem", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&user_data_type, &0u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(0u8)] } },
            3 => optional name: String => { bit: 2, key: "name", wire: [raw_string], project_wire: Iap1WireSpec::Raw, predicate_value: false, predicate: { expr: eq(&user_data_type, &1u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(1u8)] } },
            4 => optional gender: RetUserDataGender => { bit: 3, key: "gender", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&user_data_type, &2u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(2u8)] } },
            5 => optional weight: u16 => { bit: 4, key: "weight", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&user_data_type, &3u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(3u8)] } },
            6 => optional age: u8 => { bit: 5, key: "age", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&user_data_type, &4u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(4u8)] } },
            7 => optional workout_recording_preference: RetUserDataWorkoutRecordingPreference => { bit: 6, key: "workoutRecordingPreference", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&user_data_type, &5u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(5u8)] } },
        }
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum SetUserDataUserDataType: u8 {
        /// preferred unit system
        PreferredUnitSystem = 0,
        /// gender
        Gender = 2,
        /// weight
        Weight = 3,
        /// age
        Age = 4,
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum SetUserDataPreferredUnitSystem: u8 {
        /// no information
        NoInformation = 0,
        /// imperial units (lbs/miles)
        ImperialUnitsLbsMiles = 1,
        /// metric units (kg/km)
        MetricUnitsKgKm = 2,
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum SetUserDataGender: u8 {
        /// no information
        NoInformation = 0,
        /// female
        Female = 1,
        /// male
        Male = 2,
    }
}

iap1_disjoint! {
    strings = LingoString;
    #[iap1_disjoint(predicate = integer)]
    pub enum SetUserDataPreferredUnitSystemDisjoint {
        #[iap1_disjoint(decode = decode_preferred_unit_system, encode = encode_preferred_unit_system)]
        /// Preferred Unit System
        PreferredUnitSystem(SetUserDataPreferredUnitSystem),
        #[iap1_disjoint(decode = decode_gender, encode = encode_gender)]
        /// Gender
        Gender(SetUserDataGender),
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x09, command = 0x008a,
        source = accessory,
        response = false,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct SetUserData {
        fields {
            /// User Data Type
            // Virtual schema indices: 1
            required pub user_data_type: SetUserDataUserDataType => { bit: 0, key: "userDataType", when: (truthy(&true)), predicate_value: true },
            /// Preferred Unit System
            // Virtual schema indices: 2, 3
            optional pub preferred_unit_system: SetUserDataPreferredUnitSystemDisjoint => { bit: 1, key: "preferredUnitSystem", when: (eq(&user_data_type, &0u64)) || (eq(&user_data_type, &2u64)), predicate_value: false },
            /// Weight
            // Virtual schema indices: 4
            optional pub weight: u16 => { bit: 2, key: "weight", when: (eq(&user_data_type, &3u64)), predicate_value: false },
            /// Age
            // Virtual schema indices: 5
            optional pub age: u8 => { bit: 3, key: "age", when: (eq(&user_data_type, &4u64)), predicate_value: false },
        }
        steps {
            1 => required user_data_type: SetUserDataUserDataType => { bit: 0, key: "userDataType", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: true, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            2 => optional preferred_unit_system: SetUserDataPreferredUnitSystemDisjoint => { bit: 1, key: "preferredUnitSystem", wire: [disjoint(decode_preferred_unit_system, encode_preferred_unit_system, SetUserDataPreferredUnitSystem, [scalar])], project_wire: Iap1WireSpec::Disjoint { variant: const { LingoString::require_raw(stringify!(decode_preferred_unit_system)) }, child: &Iap1WireSpec::Scalar }, predicate_value: false, predicate: { expr: eq(&user_data_type, &0u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(0u8)] } },
            3 => optional preferred_unit_system: SetUserDataPreferredUnitSystemDisjoint => { bit: 1, key: "preferredUnitSystem", wire: [disjoint(decode_gender, encode_gender, SetUserDataGender, [scalar])], project_wire: Iap1WireSpec::Disjoint { variant: const { LingoString::require_raw(stringify!(decode_gender)) }, child: &Iap1WireSpec::Scalar }, predicate_value: false, predicate: { expr: eq(&user_data_type, &2u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(2u8)] } },
            4 => optional weight: u16 => { bit: 2, key: "weight", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&user_data_type, &3u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(3u8)] } },
            5 => optional age: u8 => { bit: 3, key: "age", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&user_data_type, &4u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(4u8)] } },
        }
    }
}

iap1_registry! {
    0x09;
    AccessoryAck => 0x0000,
    GetAccessoryVersion => 0x0001,
    RetAccessoryVersion => 0x0002,
    GetAccessoryCaps => 0x0003,
    RetAccessoryCaps => 0x0004,
    IPodAck => 0x0080,
    GetiPodCaps => 0x0083,
    RetiPodCaps => 0x0084,
    GetUserIndex => 0x0085,
    RetUserIndex => 0x0086,
    GetUserData => 0x0088,
    RetUserData => 0x0089,
    SetUserData => 0x008a,
}
