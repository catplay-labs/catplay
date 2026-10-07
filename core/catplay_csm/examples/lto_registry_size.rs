//! LTO size probe: keep the generated iAP2 registry and its codecs reachable.
//!
//! The packet ID and payload come from argv, so release LTO cannot reduce the
//! registry to one statically selected message.

use catplay_csm::decoder::{CsmPacketRegistry, CsmPacketWithPayload, CsmPayloadEncode};
use std::{env, hint::black_box, process::ExitCode};

fn main() -> ExitCode {
    let mut args = env::args().skip(1);
    let id = args
        .next()
        .and_then(|value| value.parse().ok())
        .unwrap_or(0);
    let payload = args.next().map_or_else(Vec::new, String::into_bytes);
    let frame = match CsmPacketWithPayload::new(id, &payload) {
        Ok(frame) => frame.serialize(),
        Err(_) => return ExitCode::from(1),
    };

    let registry = CsmPacketRegistry::static_registry();
    let decoded = match registry.decode(&frame) {
        Some(packet) => packet,
        None => return ExitCode::from(2),
    };
    let encoded = match registry.encode(black_box(&decoded)) {
        Ok(bytes) => bytes,
        Err(_) => return ExitCode::from(3),
    };
    black_box(encoded);
    ExitCode::SUCCESS
}
