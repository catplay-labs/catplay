#![cfg(feature = "alloc")]

use catplay_plist::{
    Date, Dictionary, PlistByteArray, Uid, Value,
    bplist::{
        ScratchBuffers,
        wire::{
            decode::{Document, Object},
            encode::{Encoder, ScratchKind},
        },
    },
};

#[test]
fn owned_types_and_wire_scratch_work_without_serde() {
    let date = Date::from_xml_format("2026-09-24T12:34:56.123456789Z").unwrap();
    let mut values = Dictionary::new();
    values.insert("date".into(), date.into());
    values.insert("uid".into(), Uid::new(42).into());
    let value = Value::from(values);
    assert_eq!(value.as_dictionary().unwrap()["date"].as_date(), Some(date));
    let data = PlistByteArray::from([1, 2, 3]);
    let mut scratch = ScratchBuffers::default();
    scratch.grow(ScratchKind::Offsets, 1).unwrap();
    let mut bytes = Vec::new();
    let mut encoder = Encoder::new(scratch.as_scratch(), |chunk: &[u8]| {
        bytes.extend_from_slice(chunk);
        Ok::<(), core::convert::Infallible>(())
    })
    .unwrap();
    let root = encoder.data(&data).unwrap();
    encoder.finish(root).unwrap();
    let doc = Document::parse(&bytes).unwrap();
    assert!(matches!(doc.object(doc.root()).unwrap(), Object::Data([1, 2, 3])));
}
