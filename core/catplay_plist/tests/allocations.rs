#![cfg(feature = "serde")]

use std::{
    alloc::{GlobalAlloc, Layout, System},
    cell::Cell,
};

use catplay_plist::{
    CachingSerializer, PlistByteArray, Value,
    bplist::{
        serde::decode,
        wire::{
            decode::Document,
            encode::{Collection, RefEdge, Scratch},
            scalar::ScalarEncoder,
            xml,
        },
    },
    from_bytes, plist_struct,
};
use serde::Serialize;

thread_local! {
    static ACTIVE: Cell<bool> = const { Cell::new(false) };
    static COUNT: Cell<usize> = const { Cell::new(0) };
}

fn count_allocs(callback: impl FnOnce()) -> usize {
    struct StopCounting;

    impl Drop for StopCounting {
        fn drop(&mut self) {
            ACTIVE.with(|active| active.set(false));
        }
    }

    COUNT.with(|count| count.set(0));
    ACTIVE.with(|active| active.set(true));
    let guard = StopCounting;
    callback();
    let allocations = COUNT.with(Cell::get);
    drop(guard);
    allocations
}

#[test]
fn xml_wire_scalar_round_trip_has_no_allocations() {
    let mut output = [0u8; 512];
    let mut used = 0usize;
    let write_allocations = count_allocs(|| {
        let mut writer = xml::encode::Encoder::new(
            |chunk: &[u8]| -> Result<(), std::convert::Infallible> {
                output[used..used + chunk.len()].copy_from_slice(chunk);
                used += chunk.len();
                Ok(())
            },
            xml::encode::Options::default(),
        )
        .unwrap();
        writer.integer(42).unwrap();
        writer.finish().unwrap();
    });
    let read_allocations = count_allocs(|| {
        let mut reader = xml::decode::Decoder::new(&output[..used]).unwrap();
        loop {
            if matches!(reader.next().unwrap(), xml::decode::Event::Eof) {
                break;
            }
        }
    });
    assert_eq!((write_allocations, read_allocations), (0, 0));
}

struct CountAllocator;

#[global_allocator]
static ALLOCATOR: CountAllocator = CountAllocator;

unsafe impl GlobalAlloc for CountAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        ACTIVE.with(|active| {
            if active.get() {
                COUNT.with(|count| count.set(count.get() + 1));
            }
        });
        unsafe { System.alloc(layout) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { System.dealloc(ptr, layout) }
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        ACTIVE.with(|active| {
            if active.get() {
                COUNT.with(|count| count.set(count.get() + 1));
            }
        });
        unsafe { System.realloc(ptr, layout, new_size) }
    }
}

#[derive(Serialize)]
struct Message<'a> {
    name: &'a str,
    icon: &'a PlistByteArray,
    values: [u64; 3],
}

plist_struct! {
    struct BorrowedMessage<'a> {
        // This fixture tests encoding; the macro also derives Deserialize.
        #[serde(skip_deserializing)]
        pub payload: &'a [u8],
        pub label: &'a str,
    }
}

plist_struct! {
    struct BorrowedDecoded<'a> {
        pub payload: &'a [u8],
        pub label: &'a str,
    }
}

#[test]
fn borrowed_plist_struct_decodes_without_allocations() {
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct Encoded<'a> {
        payload: &'a PlistByteArray,
        label: &'a str,
    }

    let payload = PlistByteArray::from([1, 2, 3, 4]);
    let mut serializer = CachingSerializer::default();
    let bytes = serializer
        .serialize(&Encoded {
            payload: &payload,
            label: "CarPlay",
        })
        .unwrap();

    let allocations = count_allocs(|| {
        let decoded: BorrowedDecoded<'_> = decode::from_slice(&bytes).unwrap();
        assert_eq!(decoded.payload, &[1, 2, 3, 4]);
        assert_eq!(decoded.label, "CarPlay");
    });
    assert_eq!(allocations, 0);
}

#[test]
fn borrowed_plist_struct_encodes_without_allocations_with_fixed_storage() {
    let message = BorrowedMessage {
        payload: &[1, 2, 3, 4],
        label: "CarPlay",
    };
    let mut offsets = [0; 64];
    let mut collections = [Collection::default(); 16];
    let mut edges = [RefEdge::default(); 64];
    let mut output = [0u8; 1024];
    let mut used = 0;

    let allocations = count_allocs(|| {
        catplay_plist::bplist::serde::encode::encode(
            &message,
            Scratch {
                offsets: &mut offsets,
                collections: &mut collections,
                edges: &mut edges,
            },
            |chunk| {
                output[used..used + chunk.len()].copy_from_slice(chunk);
                used += chunk.len();
                Ok::<(), std::convert::Infallible>(())
            },
        )
        .unwrap();
    });
    assert_eq!(allocations, 0);

    let decoded: Value = from_bytes(&output[..used]).unwrap();
    let dict = decoded.as_dictionary().unwrap();
    assert_eq!(dict.get("label").unwrap().as_string().unwrap(), "CarPlay");
    // A plain &[u8] is serialized by Serde as a sequence, not plist Data.
    assert_eq!(dict.get("payload").unwrap().as_array().unwrap().len(), 4);

    let mut serializer = CachingSerializer::new(1024);
    let warm = serializer.serialize(&message).unwrap();
    serializer.reclaim(warm);
    let warm_allocations = count_allocs(|| {
        let output = serializer.serialize(&message).unwrap();
        serializer.reclaim(output);
    });
    assert_eq!(warm_allocations, 0);
}

#[test]
fn warm_encode_reclaim_has_no_allocations() {
    let icon = PlistByteArray::from([1, 2, 3, 4]);
    let msg = Message {
        name: "CatPlay",
        icon: &icon,
        values: [1, 2, 3],
    };
    let mut serializer = CachingSerializer::new(4096);
    let warm = serializer.serialize(&msg).unwrap();
    serializer.reclaim(warm);

    let allocations = count_allocs(|| {
        for _ in 0..5 {
            let output = serializer.serialize(&msg).unwrap();
            serializer.reclaim(output);
        }
    });
    assert_eq!(allocations, 0);
}

#[test]
fn borrowed_decode_has_no_allocations() {
    let mut serializer = CachingSerializer::default();
    let text = serializer.serialize(&"hello").unwrap();
    let data = serializer
        .serialize(&PlistByteArray::from([1, 2, 3, 4]))
        .unwrap();

    let allocations = count_allocs(|| {
        let doc = Document::parse(&text).unwrap();
        let decoded: &str = decode::from_slice(&text).unwrap();
        let decoded_data: &[u8] = decode::from_slice(&data).unwrap();

        assert_eq!(doc.object_count(), 1);
        assert_eq!(decoded, "hello");
        assert_eq!(decoded_data, &[1, 2, 3, 4]);
    });
    assert_eq!(allocations, 0);
}

#[test]
fn unicode_decode_allocates_only_the_result_string() {
    // Three-byte BMP characters and surrogate pairs exercise both UTF-16 paths.
    let text = "ł漢🚗".repeat(1024);
    let mut serializer = CachingSerializer::default();
    let bytes = serializer.serialize(&text).unwrap();
    let allocations = count_allocs(|| {
        let decoded: String = serializer.deserialize(&bytes).unwrap();
        assert_eq!(decoded, text);
    });
    assert_eq!(allocations, 1);
}

#[test]
fn warm_date_encode_and_decode_do_not_allocate() {
    let date = catplay_plist::Date::from_xml_format("2026-09-24T12:34:56.125Z").unwrap();
    let mut serializer = CachingSerializer::default();
    let warm = serializer.serialize(&date).unwrap();
    serializer.reclaim(warm);
    let allocations = count_allocs(|| {
        let bytes = serializer.serialize(&date).unwrap();
        let decoded: catplay_plist::Date = serializer.deserialize(&bytes).unwrap();
        assert_eq!(decoded, date);
        serializer.reclaim(bytes);
    });
    assert_eq!(allocations, 0);
}
