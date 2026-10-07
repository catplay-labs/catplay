//! LTO size probe: keep registry decode and encode dispatch reachable at runtime.
//!
//! `lingo` and `command` are read from argv so release LTO cannot specialize
//! the registry down to one statically known packet. The registry's full match
//! dispatch retains the codecs reachable for every generated message.

use catplay_lingo::{DecodeContext, EncodeContext, ProtocolRegistry, Writer, lingos::LingoRegistry};
use std::{env, hint::black_box, process::ExitCode};

fn main() -> ExitCode {
    let mut args = env::args().skip(1);
    let lingo = args.next().and_then(|v| v.parse().ok()).unwrap_or(0);
    let command = args.next().and_then(|v| v.parse().ok()).unwrap_or(0);
    let payload = args.next().map_or_else(Vec::new, String::into_bytes);

    let decoded = match LingoRegistry::decode(lingo, command, &DecodeContext::default(), &payload) {
        Ok(value) => value,
        Err(_) => return ExitCode::from(1),
    };
    let mut encoded = Vec::new();
    if LingoRegistry::encode(black_box(&decoded), &EncodeContext::default(), &mut Writer::new(&mut encoded)).is_err() {
        return ExitCode::from(2);
    }
    black_box(encoded);
    ExitCode::SUCCESS
}
