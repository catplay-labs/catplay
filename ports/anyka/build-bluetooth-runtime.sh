#!/bin/sh
set -eu
if [ "$#" -ne 2 ]; then
    echo "Usage: $0 APPLICATION_BUNDLE OUTPUT_DIRECTORY" >&2
    exit 2
fi
port_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
app_bundle=$(realpath "$1")
mkdir -p "$2"
output_dir=$(realpath "$2")
python3 "$port_dir/fetch-bluetooth-runtime.py" "$output_dir"
docker run --rm --network host --user "$(id -u):$(id -g)" \
    -v "$port_dir:/port:ro" -v "$output_dir:/output" \
    catplay-anyka-toolchain:bookworm arm-linux-gnueabi-gcc \
    -shared -fPIC -Os -nostdlib -Wl,-soname,libstock-clock.so \
    /port/stock-clock.c -o /output/libstock-clock.so
python3 "$port_dir/package-bluetooth-runtime.py" "$output_dir/sysroot" "$app_bundle" \
    "$output_dir/libstock-clock.so" "$output_dir/bundle"
echo "Built $output_dir/bundle/start"
