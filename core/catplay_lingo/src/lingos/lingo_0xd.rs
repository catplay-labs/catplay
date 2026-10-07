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

// Lingo 0x0d: iPod Out
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
        context = ctx, strings = LingoString, lingo = 0x0d, command = 0x0000,
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

iap1_enum! {
    strings = LingoString;
    pub enum GetiPodOutOptionsEnabledOptions: u8 {
        /// all capable
        AllCapable = 0,
        /// currently set
        CurrentlySet = 1,
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x0d, command = 0x0001,
        source = accessory,
        response = false,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct GetiPodOutOptions {
        fields {
            /// Enabled Options
            // Virtual schema indices: 1
            required pub enabled_options: GetiPodOutOptionsEnabledOptions => { bit: 0, key: "enabledOptions", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required enabled_options: GetiPodOutOptionsEnabledOptions => { bit: 0, key: "enabledOptions", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum RetiPodOutOptionsEnabledOptions: u8 {
        /// all capable
        AllCapable = 0,
        /// currently set
        CurrentlySet = 1,
    }
}

iap1_bitflags! {
    strings = LingoString;
    pub struct RetiPodOutOptionsOptionsBits: u32 {
        /// audio content
        const AUDIO_CONTENT = 1 << 0;
        /// incoming phone call UI
        const INCOMING_PHONE_CALL_UI = 1 << 1;
        /// incoming SMS/MMS text UI
        const INCOMING_SMS_MMS_TEXT_UI = 1 << 2;
        /// incoming voicemail
        const INCOMING_VOICEMAIL = 1 << 4;
        /// incoming push notification
        const INCOMING_PUSH_NOTIFICATION = 1 << 5;
        /// clock alarm notification
        const CLOCK_ALARM_NOTIFICATION = 1 << 6;
        /// test pattern
        const TEST_PATTERN = 1 << 8;
        /// minimum user interface
        const MINIMUM_USER_INTERFACE = 1 << 9;
        /// full user interface
        const FULL_USER_INTERFACE = 1 << 10;
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x0d, command = 0x0002,
        source = device,
        response = true,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct RetiPodOutOptions {
        fields {
            /// Enabled Options
            // Virtual schema indices: 1
            required pub enabled_options: RetiPodOutOptionsEnabledOptions => { bit: 0, key: "enabledOptions", when: (truthy(&true)), predicate_value: false },
            /// Options Bits
            // Virtual schema indices: 2
            required pub options_bits: RetiPodOutOptionsOptionsBits => { bit: 1, key: "optionsBits", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required enabled_options: RetiPodOutOptionsEnabledOptions => { bit: 0, key: "enabledOptions", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            2 => required options_bits: RetiPodOutOptionsOptionsBits => { bit: 1, key: "optionsBits", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1_bitflags! {
    strings = LingoString;
    pub struct SetiPodOutOptionsOptionsBits: u32 {
        /// audio content
        const AUDIO_CONTENT = 1 << 0;
        /// incoming phone call UI
        const INCOMING_PHONE_CALL_UI = 1 << 1;
        /// incoming SMS/MMS text UI
        const INCOMING_SMS_MMS_TEXT_UI = 1 << 2;
        /// incoming voicemail
        const INCOMING_VOICEMAIL = 1 << 4;
        /// incoming push notification
        const INCOMING_PUSH_NOTIFICATION = 1 << 5;
        /// clock alarm notification
        const CLOCK_ALARM_NOTIFICATION = 1 << 6;
        /// test pattern
        const TEST_PATTERN = 1 << 8;
        /// minimum user interface
        const MINIMUM_USER_INTERFACE = 1 << 9;
        /// full user interface
        const FULL_USER_INTERFACE = 1 << 10;
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x0d, command = 0x0003,
        source = accessory,
        response = false,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct SetiPodOutOptions {
        fields {
            /// Options Bits
            // Virtual schema indices: 1
            required pub options_bits: SetiPodOutOptionsOptionsBits => { bit: 0, key: "optionsBits", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required options_bits: SetiPodOutOptionsOptionsBits => { bit: 0, key: "optionsBits", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum AccessoryStateChangeEventStateChange: u8 {
        /// switching from iPod Out
        SwitchingFromIPodOut = 0,
        /// switching to iPod Out
        SwitchingToIPodOut = 1,
        /// switching from iPod audio
        SwitchingFromIPodAudio = 2,
        /// switching to iPod audio
        SwitchingToIPodAudio = 3,
        /// entering daytime mode
        EnteringDaytimeMode = 4,
        /// entering nighttime mode
        EnteringNighttimeMode = 5,
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x0d, command = 0x0004,
        source = accessory,
        response = false,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct AccessoryStateChangeEvent {
        fields {
            /// State Change
            // Virtual schema indices: 1
            required pub state_change: AccessoryStateChangeEventStateChange => { bit: 0, key: "stateChange", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required state_change: AccessoryStateChangeEventStateChange => { bit: 0, key: "stateChange", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1_bitflags! {
    strings = LingoString;
    pub struct AccessoryVideoScreenInfoScreenFeatures: u8 {
        /// color screen
        const COLOR_SCREEN = 1 << 0;
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x0d, command = 0x0006,
        source = accessory,
        response = false,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct AccessoryVideoScreenInfo {
        fields {
            /// Accessory Screen Width in Inches
            // Virtual schema indices: 1
            required pub total_screen_width_inches: u16 => { bit: 0, key: "totalScreenWidthInches", when: (truthy(&true)), predicate_value: false },
            /// Accessory Height Width in Inches
            // Virtual schema indices: 2
            required pub total_screen_height_inches: u16 => { bit: 1, key: "totalScreenHeightInches", when: (truthy(&true)), predicate_value: false },
            /// Accessory Screen Width in Pixels
            // Virtual schema indices: 3
            required pub total_screen_width_pixels: u16 => { bit: 2, key: "totalScreenWidthPixels", when: (truthy(&true)), predicate_value: false },
            /// Accessory Screen Height in Pixels
            // Virtual schema indices: 4
            required pub total_screen_height_pixels: u16 => { bit: 3, key: "totalScreenHeightPixels", when: (truthy(&true)), predicate_value: false },
            /// iPod Out Width Allotment
            // Virtual schema indices: 5
            required pub i_pod_out_screen_width_pixels: u16 => { bit: 4, key: "iPodOutScreenWidthPixels", when: (truthy(&true)), predicate_value: false },
            /// iPod Out Height Allotment
            // Virtual schema indices: 6
            required pub i_pod_out_screen_height_pixels: u16 => { bit: 5, key: "iPodOutScreenHeightPixels", when: (truthy(&true)), predicate_value: false },
            /// Device Screen Features Bits
            // Virtual schema indices: 7
            required pub screen_features: AccessoryVideoScreenInfoScreenFeatures => { bit: 6, key: "screenFeatures", when: (truthy(&true)), predicate_value: false },
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
            7 => required screen_features: AccessoryVideoScreenInfoScreenFeatures => { bit: 6, key: "screenFeatures", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            8 => required screen_gamma_value: u8 => { bit: 7, key: "screenGammaValue", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1_registry! {
    0x0d;
    IPodAck => 0x0000,
    GetiPodOutOptions => 0x0001,
    RetiPodOutOptions => 0x0002,
    SetiPodOutOptions => 0x0003,
    AccessoryStateChangeEvent => 0x0004,
    AccessoryVideoScreenInfo => 0x0006,
}
