use catplay_lingo::lingos::{
    lingo_0x0::RetSupportedEventNotification,
    lingo_0x4::{GetDBTrackInfo, GetPBTrackInfo, GetUIDTrackInfo},
};
use catplay_lingo::{DecodeContext, EncodeContext, Iap1Decode, Iap1Encode, Iap1Message, Reader, Writer, iap1_bitfield};

iap1_bitfield! {
    struct DataGroups: u16, little_endian {
        #[group(mask = 0x300, shift = 8)]
        mode: DataMode { First = 1, Second = 2, },
    }
}

#[test]
fn captured_track_mask_matches_response_types_and_preserves_wire_bytes() {
    let mut payload = vec![0, 0, 0, 0, 0, 0, 0, 1];
    payload.extend([0x7f, 0x11, 0x08, 0]);
    let expected = 0x0008_117f;
    let pb = GetPBTrackInfo::decode(&DecodeContext::default(), &payload).unwrap();
    assert_eq!(pb.track_information_type_bits.bits(), expected);
    let db = GetDBTrackInfo::decode(&DecodeContext::default(), &payload).unwrap();
    assert_eq!(db.track_information_type_bits.bits(), expected);
    let uid = GetUIDTrackInfo::decode(&DecodeContext::default(), &payload).unwrap();
    assert_eq!(uid.track_information_type_bits.bits(), expected);
    for message in [&pb as &dyn TestEncode, &db, &uid] {
        assert_eq!(message.bytes(), payload);
    }
}
trait TestEncode {
    fn bytes(&self) -> Vec<u8>;
}
impl<T: Iap1Message> TestEncode for T {
    fn bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        self.encode(&EncodeContext::default(), &mut Writer::new(&mut bytes))
            .unwrap();
        bytes
    }
}

#[test]
fn data_mask_preserves_each_bit_including_unknown_bits() {
    for bit in 0..32 {
        let mask = 1u32 << bit;
        let mut payload = vec![0u8; 8];
        payload.extend(mask.to_le_bytes());
        let message = GetPBTrackInfo::decode(&DecodeContext::default(), &payload).unwrap();
        assert_eq!(message.track_information_type_bits.bits(), mask);
        assert_eq!(message.bytes(), payload);
    }
}

#[test]
fn scalar_notification_mask_remains_big_endian() {
    let bytes = 0x1a_fea5u64.to_be_bytes();
    let message = RetSupportedEventNotification::decode(&DecodeContext::default(), &bytes).unwrap();
    assert_eq!(message.event_notification_mask.bits(), 0x1a_fea5);
    assert_eq!(message.bytes(), bytes);
}

#[test]
fn grouped_data_fields_use_the_same_byte_order_and_preserve_reserved_bits() {
    let bytes = [0x80, 0x82];
    let message = DataGroups::decode(&mut Reader::new(&bytes)).unwrap();
    assert_eq!(message.mode, DataMode::Second);
    assert_eq!(message.reserved_bits, 0x8080);
    let mut encoded = Vec::new();
    message.encode(&mut Writer::new(&mut encoded)).unwrap();
    assert_eq!(encoded, bytes);
    assert!(DataGroups::decode(&mut Reader::new(&[0])).is_err());
}
