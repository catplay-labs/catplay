#!/bin/sh
# Build the pinned SDK's Linux FunctionFS integration test as a static ARMv5 binary.
set -eu
if [ "$#" -ne 2 ]; then
    echo "Usage: $0 SDK_CHECKOUT OUTPUT_DIRECTORY" >&2
    exit 2
fi
port_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
sdk_dir=$(realpath "$1")
mkdir -p "$2"
output_dir=$(realpath "$2")
if [ "$(git -C "$sdk_dir" rev-parse HEAD)" != a7c6e75c154db2b5c270603d76f85fa66e0a3127 ]; then
    echo 'Unexpected SDK revision' >&2
    exit 1
fi
docker build -t catplay-anyka-toolchain:bookworm "$port_dir"
docker run --rm --user "$(id -u):$(id -g)" \
    -v "$sdk_dir:/sdk:ro" -v "$output_dir:/output" \
    catplay-anyka-toolchain:bookworm arm-linux-gnueabi-gcc \
    -Os -static -march=armv5te -D_DEFAULT_SOURCE -I/sdk/os/kernel/tools/include \
    /sdk/os/kernel/tools/usb/ffs-test.c -pthread -o /output/ffs-test
echo "Built $output_dir/ffs-test"
