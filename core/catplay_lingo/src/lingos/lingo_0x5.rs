// @generated from Apple ATS Default.lktspec
// Do not edit manually. Regenerate with catplay_lingo.

#[allow(unused_imports)]
use crate::{__String as String, __Vec as Vec};
#[allow(unused_imports)]
use crate::{
    FlatPredicateToken, Iap1WireSpec, iap1, iap1_bitfield, iap1_bitflags, iap1_disjoint, iap1_enum, iap1_enum_open, iap1_enum_tokens,
    iap1_record, iap1_registry, predicate_eval::*,
};

use super::strings::LingoString;

// Lingo 0x05: Accessory Power [deprecated]
iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x05, command = 0x0002,
        source = device,
        response = false,
        ack = false,
        deprecated = true,
        transaction_id = permitted
    )]
    pub struct BeginHighPower {
        fields {
        }
        steps {
        }
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x05, command = 0x0003,
        source = device,
        response = false,
        ack = false,
        deprecated = true,
        transaction_id = permitted
    )]
    pub struct EndHighPower {
        fields {
        }
        steps {
        }
    }
}

iap1_registry! {
    0x05;
    BeginHighPower => 0x0002,
    EndHighPower => 0x0003,
}
