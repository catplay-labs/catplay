#!/bin/sh
# Build NCM modules for the audited stock 20260612170300CA firmware layout.
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
python3 "$port_dir/prepare-stock-usb.py" "$sdk_dir" "$output_dir/stock-usb"
docker run --rm --user "$(id -u):$(id -g)" \
    -v "$sdk_dir:/sdk:ro" -v "$output_dir:/output" \
    -w /sdk/os/kernel catplay-anyka-toolchain:bookworm \
    make O=/output/kernel-build ARCH=arm CROSS_COMPILE=arm-linux-gnueabi- M=/output/stock-usb modules
echo "Built $output_dir/stock-usb/u_ether.ko and usb_f_ncm.ko"
