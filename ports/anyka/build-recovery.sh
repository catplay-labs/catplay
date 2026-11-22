#!/bin/sh
set -eu
if [ "$#" -ne 2 ]; then
    echo "Usage: $0 SDK_CHECKOUT OUTPUT_DIRECTORY" >&2
    exit 2
fi
port_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
sdk_dir=$(realpath "$1")
mkdir -p "$2"
output_dir=$(realpath "$2")
"$port_dir/build-probe.sh" "$sdk_dir" "$output_dir"
mkdir -p "$output_dir/recovery"
cp "$port_dir/recovery/Makefile" "$port_dir/recovery/catplay_anyka_restart.c" "$output_dir/recovery/"
docker run --rm --user "$(id -u):$(id -g)" \
    -v "$sdk_dir:/sdk:ro" -v "$output_dir:/output" \
    -w /sdk/os/kernel catplay-anyka-toolchain:bookworm \
    make O=/output/kernel-build ARCH=arm CROSS_COMPILE=arm-linux-gnueabi- M=/output/recovery modules
echo "Built $output_dir/recovery/catplay_anyka_restart.ko"
