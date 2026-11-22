#!/bin/sh
# Build the complete application and a GNU ARM runtime bundle.
set -eu
if [ "$#" -ne 1 ]; then
    echo "Usage: $0 PRIVATE_OUTPUT_DIRECTORY" >&2
    exit 2
fi
port_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
repo_dir=$(realpath "$port_dir/../..")
mkdir -p "$1"
output_dir=$(realpath "$1")
python3 "$port_dir/prepare-c2a.py" "$repo_dir" "$output_dir"
docker build --network host -f "$port_dir/Dockerfile.c2a" -t catplay-anyka-c2a:bookworm "$port_dir"
docker run --rm --network host --user "$(id -u):$(id -g)" \
    -e CARGO_HOME=/cargo -e CARGO_TARGET_DIR=/output/target \
    -v "${CARGO_HOME:-$HOME/.cargo}:/cargo" -v "$output_dir:/output" \
    -v "$port_dir:/port:ro" -w /output/source catplay-anyka-c2a:bookworm sh -ec '
    cargo build --release --target armv5te-unknown-linux-gnueabi -p catplay_c2a --no-default-features -j 8
    python3 /port/package-c2a.py /output/target/armv5te-unknown-linux-gnueabi/release/catplay_c2a /output/bundle
    cp /port/c2a-bringup.toml /output/bundle/c2a.toml
    '
echo "Built $output_dir/bundle/run"
