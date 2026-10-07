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

// Lingo 0x06: USB Host Mode
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
        /// power mode error
        PowerModeError = 9,
        /// timeout
        Timeout = 15,
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x06, command = 0x0000,
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
        context = ctx, strings = LingoString, lingo = 0x06, command = 0x0001,
        source = device,
        response = false,
        ack = false,
        deprecated = true,
        transaction_id = permitted
    )]
    pub struct GetUSBPowerState {
        fields {
        }
        steps {
        }
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x06, command = 0x0002,
        source = accessory,
        response = true,
        ack = false,
        deprecated = true,
        transaction_id = permitted
    )]
    pub struct RetUSBPowerState {
        fields {
            /// Power State
            // Virtual schema indices: 1
            required pub power_state: bool => { bit: 0, key: "powerState", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required power_state: bool => { bit: 0, key: "powerState", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x06, command = 0x0003,
        source = device,
        response = false,
        ack = false,
        deprecated = true,
        transaction_id = permitted
    )]
    pub struct SetUSBPowerState {
        fields {
            /// Power State
            // Virtual schema indices: 1
            required pub power_state: bool => { bit: 0, key: "powerState", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required power_state: bool => { bit: 0, key: "powerState", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum NotifyUSBModeUsbMode: u8 {
        /// iPod USB interface disabled/unavailable
        IPodUSBInterfaceDisabledUnavailable = 0,
        /// iPod USB interface in device mode
        IPodUSBInterfaceInDeviceMode = 1,
        /// iPod USB interface in host mode
        IPodUSBInterfaceInHostMode = 2,
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x06, command = 0x0004,
        source = device,
        response = false,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct NotifyUSBMode {
        fields {
            /// USB Mode
            // Virtual schema indices: 1
            required pub usb_mode: NotifyUSBModeUsbMode => { bit: 0, key: "usbMode", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required usb_mode: NotifyUSBModeUsbMode => { bit: 0, key: "usbMode", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
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
        context = ctx, strings = LingoString, lingo = 0x06, command = 0x0080,
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
        context = ctx, strings = LingoString, lingo = 0x06, command = 0x0081,
        source = accessory,
        response = false,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct GetiPodUSBMode {
        fields {
        }
        steps {
        }
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum RetiPodUSBModeUsbMode: u8 {
        /// iPod USB interface disabled/unavailable
        IPodUSBInterfaceDisabledUnavailable = 0,
        /// iPod USB interface in device mode
        IPodUSBInterfaceInDeviceMode = 1,
        /// iPod USB interface in host mode
        IPodUSBInterfaceInHostMode = 2,
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x06, command = 0x0082,
        source = device,
        response = true,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct RetiPodUSBMode {
        fields {
            /// USB Mode
            // Virtual schema indices: 1
            required pub usb_mode: RetiPodUSBModeUsbMode => { bit: 0, key: "usbMode", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required usb_mode: RetiPodUSBModeUsbMode => { bit: 0, key: "usbMode", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum SetiPodUSBModeUsbMode: u8 {
        /// iPod USB interface in host mode
        IPodUSBInterfaceInHostMode = 2,
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x06, command = 0x0083,
        source = accessory,
        response = false,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct SetiPodUSBMode {
        fields {
            /// USB Mode
            // Virtual schema indices: 1
            required pub usb_mode: SetiPodUSBModeUsbMode => { bit: 0, key: "usbMode", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required usb_mode: SetiPodUSBModeUsbMode => { bit: 0, key: "usbMode", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1_registry! {
    0x06;
    AccessoryAck => 0x0000,
    GetUSBPowerState => 0x0001,
    RetUSBPowerState => 0x0002,
    SetUSBPowerState => 0x0003,
    NotifyUSBMode => 0x0004,
    IPodAck => 0x0080,
    GetiPodUSBMode => 0x0081,
    RetiPodUSBMode => 0x0082,
    SetiPodUSBMode => 0x0083,
}
