#!/bin/sh
# Build a USB console that can run on the stock kernel while Wi-Fi stays active.
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
patch_file="$port_dir/patches/0001-fix-acm-notify-variable.patch"
if git -C "$sdk_dir" apply --check "$patch_file" 2>/dev/null; then
    git -C "$sdk_dir" apply "$patch_file"
elif ! git -C "$sdk_dir" apply --reverse --check "$patch_file" 2>/dev/null; then
    echo 'SDK ACM source differs from the expected patch state' >&2
    exit 1
fi
mkdir -p "$output_dir/usb-console"
for source in u_serial f_acm f_serial f_obex; do
    cp "$sdk_dir/os/kernel/drivers/usb/gadget/function/$source.c" "$output_dir/usb-console/"
done
cp "$sdk_dir/os/kernel/drivers/usb/gadget/legacy/serial.c" "$output_dir/usb-console/"
cat >"$output_dir/usb-console/Makefile" <<'MAKEFILE'
obj-m := u_serial.o usb_f_acm.o usb_f_serial.o usb_f_obex.o g_serial.o
usb_f_acm-y := f_acm.o
usb_f_serial-y := f_serial.o
usb_f_obex-y := f_obex.o
g_serial-y := serial.o
ccflags-y := -I$(srctree)/drivers/usb/gadget/ -I$(srctree)/drivers/usb/gadget/function/ -I$(srctree)/drivers/usb/gadget/udc/
MAKEFILE
docker run --rm --user "$(id -u):$(id -g)" \
    -v "$sdk_dir:/sdk:ro" -v "$output_dir:/output" -v "$port_dir:/port:ro" \
    -w /sdk/os/kernel catplay-anyka-toolchain:bookworm sh -ec '
    make O=/output/kernel-build ARCH=arm CROSS_COMPILE=arm-linux-gnueabi- M=/output/usb-console modules
    arm-linux-gnueabi-gcc -Os -static -march=armv5te -o /output/usb-shell /port/ramboot/usb-shell.c
    '
echo "Built $output_dir/usb-console/*.ko and $output_dir/usb-shell"
