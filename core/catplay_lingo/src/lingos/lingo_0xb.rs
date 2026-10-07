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

// Lingo 0x0b: Test
iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x0b, command = 0x003c,
        source = accessory,
        response = false,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct GetModelString {
        fields {
        }
        steps {
        }
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x0b, command = 0x003d,
        source = device,
        response = true,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct RetModelString {
        fields {
            /// Model String
            // Virtual schema indices: 1
            required pub model_string: String => { bit: 0, key: "modelString", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required model_string: String => { bit: 0, key: "modelString", wire: [raw_string], project_wire: Iap1WireSpec::Raw, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1_registry! {
    0x0b;
    GetModelString => 0x003c,
    RetModelString => 0x003d,
}
