#!/bin/sh
# Build CatPlay's authentication backend for the ARMv5 dongle, without a new kernel.
set -eu
if [ "$#" -ne 1 ]; then
    echo "Usage: $0 PRIVATE_OUTPUT_DIRECTORY" >&2
    exit 2
fi
port_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
mkdir -p "$1"
output_dir=$(realpath "$1")
mkdir -p "$output_dir/rust-tmp"
rustup target add armv5te-unknown-linux-musleabi
docker build -t catplay-anyka-toolchain:bookworm "$port_dir"
# Rust ships a static musl libc with this target. GCC supplies only the linker.
python3 - "$output_dir" "$port_dir" "$(rustc --print sysroot)" "$(id -u):$(id -g)" <<'PY'
from pathlib import Path
import shlex
import sys
output, port, sysroot = map(Path, sys.argv[1:4])
repo = port.parent.parent
args = ['docker', 'run', '--rm', '--user', sys.argv[4],
        '-v', f'{repo}:{repo}:ro', '-v', f'{output}:{output}',
        '-v', f'{sysroot}:{sysroot}:ro', '-w', str(repo),
        'catplay-anyka-toolchain:bookworm', 'arm-linux-gnueabi-gcc']
linker = output / 'arm-linker'
linker.write_text('#!/bin/sh\nexec ' + shlex.join(args) + ' "$@"\n')
linker.chmod(0o700)
PY
CARGO_TARGET_ARMV5TE_UNKNOWN_LINUX_MUSLEABI_LINKER="$output_dir/arm-linker" \
    CARGO_TARGET_DIR="$output_dir/rust-target" TMPDIR="$output_dir/rust-tmp" \
    cargo build --locked --manifest-path "$port_dir/mfi-smoke/Cargo.toml" \
    --target armv5te-unknown-linux-musleabi --release
cp "$output_dir/rust-target/armv5te-unknown-linux-musleabi/release/catplay-anyka-mfi-smoke" \
    "$output_dir/catplay-mfi-smoke"
echo "Built $output_dir/catplay-mfi-smoke"
