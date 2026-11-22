#!/bin/sh
# Build a diagnostic kernel with an embedded initramfs and no MTD flash support.
set -eu
if [ "$#" -ne 3 ]; then
    echo "Usage: $0 SDK_CHECKOUT PRIVATE_CAPTURE_DIRECTORY OUTPUT_DIRECTORY" >&2
    exit 2
fi
port_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
sdk_dir=$(realpath "$1")
capture_dir=$(realpath "$2")
mkdir -p "$3"
output_dir=$(realpath "$3")
if [ "$(git -C "$sdk_dir" rev-parse HEAD)" != a7c6e75c154db2b5c270603d76f85fa66e0a3127 ]; then
    echo 'SDK revision mismatch' >&2
    exit 1
fi
for patch_file in "$port_dir"/patches/*.patch; do
if git -C "$sdk_dir" apply --check "$patch_file" 2>/dev/null; then
    git -C "$sdk_dir" apply "$patch_file"
elif ! git -C "$sdk_dir" apply --reverse --check "$patch_file" 2>/dev/null; then
    echo "SDK source differs from the expected patch state: $patch_file" >&2
    exit 1
fi
done
python3 "$port_dir/ramboot/prepare-initramfs.py" "$capture_dir/rootfs" "$output_dir"
docker build -t catplay-anyka-toolchain:bookworm "$port_dir"
docker run --rm --user "$(id -u):$(id -g)" \
    -v "$sdk_dir:/sdk:ro" -v "$capture_dir:/work:ro" \
    -v "$output_dir:/output" -v "$port_dir:/port:ro" \
    -w /sdk/os/kernel catplay-anyka-toolchain:bookworm sh -ec '
    arm-linux-gnueabi-gcc -Os -static -march=armv5te -o /output/usb-shell /port/ramboot/usb-shell.c
    make O=/output/kernel-build ARCH=arm CROSS_COMPILE=arm-linux-gnueabi- anycloud_ak3918av130_mini_defconfig
    scripts/config --file /output/kernel-build/.config \
        --enable KEXEC --enable BLK_DEV_INITRD --enable BLK_DEV_RAM --enable RD_GZIP \
        --enable DEVTMPFS --enable DEVTMPFS_MOUNT --enable USB --enable USB_COMMON \
        --enable USB_GADGET --enable USB_LIBCOMPOSITE --enable USB_G_SERIAL \
        --disable USB_MASS_STORAGE --disable MTD --enable KERNEL_GZIP --disable KERNEL_XZ \
        --set-str INITRAMFS_SOURCE /output/initramfs.list \
        --set-str LOCALVERSION "" --disable LOCALVERSION_AUTO
    make O=/output/kernel-build ARCH=arm CROSS_COMPILE=arm-linux-gnueabi- olddefconfig
    make -j8 O=/output/kernel-build ARCH=arm CROSS_COMPILE=arm-linux-gnueabi- HOSTCFLAGS=-fcommon zImage
    cp /work/mtd3-dtb.bin /output/ramboot.dtb
    fdtput -t s /output/ramboot.dtb /chosen bootargs \
        "console=ttySAK0,115200n8 rdinit=/init mem=64M memsize=64M loglevel=8 ignore_loglevel"
    fdtput -t x /output/ramboot.dtb /memory reg 80000000 4000000
    cp /output/kernel-build/arch/arm/boot/zImage /output/zImage
    '
echo "Built $output_dir/zImage and $output_dir/ramboot.dtb"
