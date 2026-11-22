#!/bin/sh
# Build the Rust USB diagnostic with a private, version-pinned compatibility patch.
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
cargo fetch --locked --manifest-path "$port_dir/usb-smoke/Cargo.toml"
# Discover the exact registry source through Cargo; never edit its shared cache.
cargo metadata --locked --format-version 1 --manifest-path "$port_dir/usb-smoke/Cargo.toml" \
    >"$output_dir/cargo-metadata.json"
python3 - "$output_dir" "$port_dir" "$(rustc --print sysroot)" "$(id -u):$(id -g)" <<'PY'
from pathlib import Path
import json
import shlex
import shutil
import subprocess
import sys
output, port, sysroot = map(Path, sys.argv[1:4])
metadata = json.loads((output / 'cargo-metadata.json').read_text())
packages = [p for p in metadata['packages'] if p['name'] == 'usb-gadget' and p['version'] == '0.7.6']
if len(packages) != 1 or not packages[0]['source'].startswith('registry+'):
    raise SystemExit('Expected registry usb-gadget 0.7.6')
source = Path(packages[0]['manifest_path']).parent
private_crate = output / 'usb-gadget'
shutil.copytree(source, private_crate, dirs_exist_ok=True)
patched_file = private_crate / 'src/function/custom/ffs.rs'
patched_file.chmod(0o644)
subprocess.run(['git', 'apply', '--check', str(port / 'usb-smoke/legacy-os-descriptor.patch')],
               cwd=private_crate, check=True)
subprocess.run(['git', 'apply', str(port / 'usb-smoke/legacy-os-descriptor.patch')],
               cwd=private_crate, check=True)
staged = output / 'usb-smoke'
shutil.copytree(port / 'usb-smoke', staged, dirs_exist_ok=True)
with (staged / 'Cargo.toml').open('a') as manifest:
    manifest.write('\n[patch.crates-io]\nusb-gadget = { path = "../usb-gadget" }\n')
shutil.copyfile(port / 'stock-clock.rs', output / 'stock-clock.rs')
repo = port.parent.parent
args = ['docker', 'run', '--rm', '--user', sys.argv[4],
        '-v', f'{repo}:{repo}:ro', '-v', f'{output}:{output}',
        '-v', f'{sysroot}:{sysroot}:ro', '-w', str(repo),
        'catplay-anyka-toolchain:bookworm', 'arm-linux-gnueabi-gcc']
linker = output / 'arm-linker'
linker.write_text('#!/bin/sh\nexec ' + shlex.join(args) + ' "$@"\n')
linker.chmod(0o700)
PY
# Replace only this package's registry entry with the patched private path.
cargo update --offline --manifest-path "$output_dir/usb-smoke/Cargo.toml" -p usb-gadget
CARGO_TARGET_ARMV5TE_UNKNOWN_LINUX_MUSLEABI_LINKER="$output_dir/arm-linker" \
    CARGO_TARGET_DIR="$output_dir/rust-target" TMPDIR="$output_dir/rust-tmp" \
    cargo build --locked --manifest-path "$output_dir/usb-smoke/Cargo.toml" \
    --target armv5te-unknown-linux-musleabi --release
cp "$output_dir/rust-target/armv5te-unknown-linux-musleabi/release/catplay-anyka-usb-smoke" \
    "$output_dir/catplay-usb-smoke"
echo "Built $output_dir/catplay-usb-smoke"
