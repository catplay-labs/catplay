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

// Lingo 0x08: Accessory Equalizer
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
        /// mismatched authentication protocol version
        MismatchedAuthenticationProtocolVersion = 8,
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x08, command = 0x0000,
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
        context = ctx, strings = LingoString, lingo = 0x08, command = 0x0001,
        source = device,
        response = false,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct GetCurrentEQIndex {
        fields {
        }
        steps {
        }
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x08, command = 0x0002,
        source = accessory,
        response = true,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct RetCurrentEQIndex {
        fields {
            /// Equalizer Index
            // Virtual schema indices: 1
            required pub equalizer_index: u8 => { bit: 0, key: "equalizerIndex", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required equalizer_index: u8 => { bit: 0, key: "equalizerIndex", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x08, command = 0x0003,
        source = device,
        response = false,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct SetCurrentEQIndex {
        fields {
            /// Equalizer Index
            // Virtual schema indices: 1
            required pub equalizer_index: u8 => { bit: 0, key: "equalizerIndex", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required equalizer_index: u8 => { bit: 0, key: "equalizerIndex", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x08, command = 0x0004,
        source = device,
        response = false,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct GetEQSettingCount {
        fields {
        }
        steps {
        }
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x08, command = 0x0005,
        source = accessory,
        response = true,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct RetEQSettingCount {
        fields {
            /// Equalizer Count
            // Virtual schema indices: 1
            required pub equalizer_count: u8 => { bit: 0, key: "equalizerCount", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required equalizer_count: u8 => { bit: 0, key: "equalizerCount", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x08, command = 0x0006,
        source = device,
        response = false,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct GetEQIndexName {
        fields {
            /// Equalizer Index
            // Virtual schema indices: 1
            required pub equalizer_index: u8 => { bit: 0, key: "equalizerIndex", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required equalizer_index: u8 => { bit: 0, key: "equalizerIndex", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x08, command = 0x0007,
        source = accessory,
        response = true,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct RetEQIndexName {
        fields {
            /// Equalizer Index
            // Virtual schema indices: 1
            required pub equalizer_index: u8 => { bit: 0, key: "equalizerIndex", when: (truthy(&true)), predicate_value: false },
            /// Equalizer Name
            // Virtual schema indices: 2
            required pub equalizer_name: String => { bit: 1, key: "equalizerName", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required equalizer_index: u8 => { bit: 0, key: "equalizerIndex", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            2 => required equalizer_name: String => { bit: 1, key: "equalizerName", wire: [raw_string], project_wire: Iap1WireSpec::Raw, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1_registry! {
    0x08;
    AccessoryAck => 0x0000,
    GetCurrentEQIndex => 0x0001,
    RetCurrentEQIndex => 0x0002,
    SetCurrentEQIndex => 0x0003,
    GetEQSettingCount => 0x0004,
    RetEQSettingCount => 0x0005,
    GetEQIndexName => 0x0006,
    RetEQIndexName => 0x0007,
}
