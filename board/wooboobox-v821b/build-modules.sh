#!/bin/bash
# Requires an already prepared, matching Linux 7.2.13 build tree.
# No device access, downloads, flashing or config changes.
set -euo pipefail
: "${KERNEL_SRC:?Set KERNEL_SRC to Linux 7.2.13 sources}"
: "${KERNEL_OUT:?Set KERNEL_OUT to the matching prepared kernel build directory}"
: "${MODULE_OUT:?Set MODULE_OUT to a separate output directory}"
BOARD_DIR="$(cd "$(dirname "$0")" && pwd)"
grep -q 'CONFIG_SPI_SPIDEV=m' "$KERNEL_OUT/.config"
grep -q 'UTS_RELEASE "7.2.13"' "$KERNEL_OUT/include/generated/utsrelease.h"
mkdir -p "$MODULE_OUT"
cp "$BOARD_DIR/kernel/Makefile" "$BOARD_DIR/kernel/catplay-spi1-clocks.c" "$MODULE_OUT/"
args=(-C "$KERNEL_SRC" "O=$KERNEL_OUT" ARCH=riscv KBUILD_MODPOST_WARN=1 -j4)
for key in LLVM LD HOSTCC HOSTCFLAGS HOSTLDFLAGS; do
    if declare -p "$key" >/dev/null 2>&1; then args+=("$key=${!key}"); fi
done
# Module-only builds have no vmlinux symbol table. Verify the result on the
# exact running kernel without force-load; successful linking alone is not ABI proof.
"${MAKE:-make}" "${args[@]}" drivers/spi/spidev.ko
"${MAKE:-make}" "${args[@]}" "M=$MODULE_OUT" modules
printf 'Built %s\n' "$KERNEL_OUT/drivers/spi/spidev.ko" "$MODULE_OUT/catplay-spi1-clocks.ko"
