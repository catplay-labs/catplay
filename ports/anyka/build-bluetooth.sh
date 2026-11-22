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
python3 "$port_dir/prepare-bluetooth.py" "$sdk_dir" "$output_dir/modules"
docker run --rm --network host --user "$(id -u):$(id -g)" \
    -v "$sdk_dir:/sdk:ro" -v "$output_dir:/output" \
    -w /sdk/os/kernel catplay-anyka-toolchain:bookworm sh -ec '
    scripts/config --file /output/kernel-build/.config \
      --enable IPV6 --module BT --enable BT_BREDR --disable BT_LE \
      --disable BT_HS --disable BT_DEBUGFS --module BT_RFCOMM \
      --disable BT_RFCOMM_TTY --disable BT_BNEP --disable BT_HIDP \
      --disable BT_CMTP --module BT_HCIUART --enable BT_HCIUART_H4 \
      --disable BT_HCIUART_BCSP --disable BT_HCIUART_LL --disable BT_HCIUART_ATH3K
    make O=/output/kernel-build ARCH=arm CROSS_COMPILE=arm-linux-gnueabi- olddefconfig modules_prepare
    make O=/output/kernel-build ARCH=arm CROSS_COMPILE=arm-linux-gnueabi- M=/output/modules -j8 modules
    '
echo "Built stock-kernel Bluetooth modules under $output_dir/modules"
