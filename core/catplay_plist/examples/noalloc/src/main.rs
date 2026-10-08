//! Link check without std or a global allocator:
//! cargo build --manifest-path core/catplay_plist/examples/noalloc/Cargo.toml --target x86_64-unknown-uefi
#![cfg_attr(target_os = "uefi", no_std)]
#![cfg_attr(target_os = "uefi", no_main)]

use catplay_plist::bplist::wire::{
    decode::{Document, Integer, Object},
    encode::{Encoder, Scratch},
};

fn round_trip() -> bool {
    let mut output = [0u8; 64];
    let mut used = 0;
    let mut offsets = [0; 1];
    let mut encoder = Encoder::new(
        Scratch {
            offsets: &mut offsets,
            collections: &mut [],
            edges: &mut [],
        },
        |chunk: &[u8]| -> Result<(), core::convert::Infallible> {
            output[used..used + chunk.len()].copy_from_slice(chunk);
            used += chunk.len();
            Ok(())
        },
    )
    .unwrap();
    let root = encoder.integer(42).unwrap();
    encoder.finish(root).unwrap();
    let doc = Document::parse(&output[..used]).unwrap();
    matches!(doc.object(doc.root()).unwrap(), Object::Integer(Integer::Unsigned(42)))
}

#[cfg(not(target_os = "uefi"))]
fn main() {
    assert!(round_trip());
}

#[cfg(target_os = "uefi")]
#[unsafe(no_mangle)]
pub extern "efiapi" fn efi_main(_: usize, _: usize) -> usize {
    usize::from(!round_trip())
}

#[cfg(target_os = "uefi")]
#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {
        core::hint::spin_loop();
    }
}
