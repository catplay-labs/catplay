#!/bin/sh
# Use Rust's bundled linker; no cross C compiler is needed for this helper.
set -eu
if [ "$#" -ne 1 ]; then
    echo "Usage: $0 OUTPUT_EXECUTABLE" >&2
    exit 2
fi
port_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
rustup target add armv5te-unknown-linux-musleabi
sysroot=$(rustc --print sysroot)
rust_host=$(rustc -vV | sed -n 's/^host: //p')
rustc --edition=2024 --target armv5te-unknown-linux-musleabi \
    -C linker="$sysroot/lib/rustlib/$rust_host/bin/rust-lld" \
    -C linker-flavor=ld.lld -C link-self-contained=yes \
    -C panic=abort -C strip=symbols -O "$port_dir/bluetooth-uart.rs" -o "$1"
