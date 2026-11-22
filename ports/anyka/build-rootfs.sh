#!/bin/sh
# Build a private persistent image and static installer using existing bundles.
set -eu
if [ "$#" -lt 8 ] || [ "$#" -gt 9 ]; then
    echo "Usage: $0 STOCK_ROOTFS APP_BUNDLE BT_BUNDLE IPHONE_MODULES BT_MODULES NET_MODULES UART_BINARY OUTPUT [--diagnostic-identity]" >&2
    exit 2
fi
port=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
stock=$(realpath "$1")
app=$(realpath "$2")
bt=$(realpath "$3")
modules=$(realpath "$4")
bt_modules=$(realpath "$5")
net_modules=$(realpath "$6")
uart=$(realpath "$7")
mkdir -p "$8"
output=$(realpath "$8")
"$port/build-capture.sh" "$output/catplay-capture"
identity=${9:-}
case "$identity" in ''|--diagnostic-identity) ;; *) echo 'Unknown option' >&2; exit 2;; esac
docker build -f "$port/Dockerfile.image" -t catplay-anyka-image:bookworm "$port"
docker run --rm --user "$(id -u):$(id -g)" \
    -v "$port:/port:ro" -v "$stock:/stock.squashfs:ro" \
    -v "$app:/app:ro" -v "$bt:/bt:ro" -v "$modules:/modules:ro" \
    -v "$bt_modules:/bt-modules:ro" -v "$net_modules:/net-modules:ro" \
    -v "$uart:/bluetooth-uart:ro" \
    -v "$output:/output" catplay-anyka-image:bookworm \
    python3 /port/package-rootfs.py --stock /stock.squashfs --app /app \
    --bluetooth /bt --modules /modules --bluetooth-modules /bt-modules \
    --net-modules /net-modules \
    --uart /bluetooth-uart \
    --output /output ${identity:+"$identity"}
docker run --rm --user "$(id -u):$(id -g)" \
    -v "$port:/port:ro" -v "$output:/output" catplay-anyka-image:bookworm \
    arm-linux-gnueabi-gcc -static -Os -march=armv5te -mfloat-abi=soft \
    -Wall -Wextra -Werror /port/flash-rootfs.c -o /output/flash-rootfs
