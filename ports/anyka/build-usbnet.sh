#!/bin/sh
# Build usbnet and cdc_ncm for the stock kernel, which ships without them.
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
mkdir -p "$output_dir/modules"
cp "$sdk_dir/os/kernel/drivers/net/usb/usbnet.c" \
    "$sdk_dir/os/kernel/drivers/net/usb/cdc_ncm.c" "$output_dir/modules/"
echo 'obj-m += usbnet.o cdc_ncm.o' >"$output_dir/modules/Makefile"
docker run --rm --network host --user "$(id -u):$(id -g)" \
    -v "$sdk_dir:/sdk:ro" -v "$output_dir:/output" \
    -w /sdk/os/kernel catplay-anyka-toolchain:bookworm \
    make O=/output/kernel-build ARCH=arm CROSS_COMPILE=arm-linux-gnueabi- \
    M=/output/modules -j8 modules
echo "Built usbnet.ko and cdc_ncm.ko under $output_dir/modules"
