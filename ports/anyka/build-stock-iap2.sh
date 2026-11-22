#!/bin/sh
# Compile the shared iAP2 transports. This does not build the iPhone gadget.
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
python3 - "$port_dir" "$output_dir" <<'PY'
from pathlib import Path
import shutil
import sys

port, output = map(Path, sys.argv[1:])
source = port.parents[1] / 'usb/catplay_iap2_usb_host/g_iphone'
stage = output / 'stock-iap2'
stage.mkdir(exist_ok=True)
for name in ['iap2_char', 'iap2_hid', 'iap2_usb_bulk']:
    shutil.copytree(source / name, stage / name, dirs_exist_ok=True,
                    ignore=shutil.ignore_patterns('*.o', '*.ko', '*.cmd', '*.mod.c'))
shutil.copyfile(port / 'stock-iap2-compat.h', stage / 'compat.h')
(stage / 'Makefile').write_text('''obj-m := iap2_char.o iap2_hid.o iap2_usb_bulk.o
iap2_char-y := iap2_char/iap2_char_core.o
iap2_hid-y := iap2_hid/iap2_hid.o iap2_hid/hid_framing.o
iap2_usb_bulk-y := iap2_usb_bulk/iap2_usb_bulk.o
ccflags-y := -include $(src)/compat.h
''')
PY
docker run --rm --user "$(id -u):$(id -g)" \
    -v "$sdk_dir:/sdk:ro" -v "$output_dir:/output" \
    -w /sdk/os/kernel catplay-anyka-toolchain:bookworm \
    make O=/output/kernel-build ARCH=arm CROSS_COMPILE=arm-linux-gnueabi- M=/output/stock-iap2 modules
echo "Built iAP2 transport modules; stock runtime ABI validation is still required."
