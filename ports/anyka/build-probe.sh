#!/bin/sh
# Build an ARMv5 probe against the public AV130 SDK, without changing that checkout.
set -eu
if [ "$#" -ne 2 ]; then
    echo "Usage: $0 SDK_CHECKOUT OUTPUT_DIRECTORY" >&2
    exit 2
fi
sdk_dir=$(realpath "$1")
mkdir -p "$2"
output_dir=$(realpath "$2")
port_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
expected_commit=a7c6e75c154db2b5c270603d76f85fa66e0a3127
if [ "$(git -C "$sdk_dir" rev-parse HEAD)" != "$expected_commit" ]; then
    echo "SDK revision mismatch: expected $expected_commit" >&2
    exit 1
fi
docker build -t catplay-anyka-toolchain:bookworm "$port_dir"
docker run --rm --user "$(id -u):$(id -g)" \
    -v "$sdk_dir:/sdk:ro" -v "$output_dir:/output" -v "$port_dir:/port" \
    -w /sdk/os/kernel catplay-anyka-toolchain:bookworm sh -ec '
    make O=/output/kernel-build ARCH=arm CROSS_COMPILE=arm-linux-gnueabi- anycloud_ak3918av130_mini_defconfig
    scripts/config --file /output/kernel-build/.config --set-str LOCALVERSION "" --disable LOCALVERSION_AUTO
    make O=/output/kernel-build ARCH=arm CROSS_COMPILE=arm-linux-gnueabi- modules_prepare
    test "$(cat /output/kernel-build/include/config/kernel.release)" = 4.4.302-cip94
    make O=/output/kernel-build ARCH=arm CROSS_COMPILE=arm-linux-gnueabi- M=/port/probe modules
    cp /port/probe/catplay_anyka_probe.ko /output/
    arm-linux-gnueabi-readelf -p .modinfo /output/catplay_anyka_probe.ko
    '
echo "Built $output_dir/catplay_anyka_probe.ko"
