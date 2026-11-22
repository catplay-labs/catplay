#!/bin/sh
set -eu
if [ "$#" -ne 1 ]; then echo "Usage: $0 OUTPUT" >&2; exit 2; fi
port=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
sysroot=$(rustc --print sysroot)
host=$(rustc -vV | sed -n 's/^host: //p')
rustc --edition=2024 --target armv5te-unknown-linux-musleabi \
    -C linker="$sysroot/lib/rustlib/$host/bin/rust-lld" \
    -C linker-flavor=ld.lld -C link-self-contained=yes \
    -C panic=abort -C strip=symbols -C opt-level=z -C lto=yes \
    "$port/capture.rs" -o "$1"
