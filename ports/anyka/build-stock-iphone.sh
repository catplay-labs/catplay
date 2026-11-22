#!/bin/sh
# Build CatPlay's iPhone gadget against the audited stock Linux 4.4 SDK.
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
python3 "$port_dir/prepare-stock-iphone.py" "$sdk_dir" "$output_dir/stock-iphone"
docker run --rm --user "$(id -u):$(id -g)" \
    -v "$sdk_dir:/sdk:ro" -v "$output_dir:/output" \
    -w /sdk/os/kernel catplay-anyka-toolchain:bookworm \
    make O=/output/kernel-build ARCH=arm CROSS_COMPILE=arm-linux-gnueabi- M=/output/stock-iphone modules
echo "Built $output_dir/stock-iphone/g_iphone.ko and its iAP2 dependencies"
