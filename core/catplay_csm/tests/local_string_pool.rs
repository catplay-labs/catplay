#![cfg(feature = "project")]

use catplay_csm::{
    decoder::{CsmDecode, CsmPayloadEncode},
    group_type,
};
use catplay_reflect::static_strings;

static_strings! {
    pub struct LocalString(u8) {
        RECORD = "LocalRecord",
        VALUE = "value",
    }
}

group_type! {
    strings = LocalString;
    pub struct LocalRecord {
        #[csm_id(1)]
        #[csm_count(one)]
        pub value: u16,
    }
}

#[test]
fn local_struct_uses_its_own_pool() {
    let value = LocalRecord { value: 0x1234 };
    assert_eq!(format!("{value:?}"), "LocalRecord { value: 4660 }");
    let encoded = value.serialize();
    assert_eq!(encoded, [0, 6, 0, 1, 0x12, 0x34]);
    assert_eq!(LocalRecord::decode_from_bytes(&encoded), value);
}
