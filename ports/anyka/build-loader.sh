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
python3 "$port_dir/prepare-loader.py" "$sdk_dir" "$output_dir/loader"
docker run --rm --user "$(id -u):$(id -g)" \
    -v "$sdk_dir:/sdk:ro" -v "$output_dir:/output" -v "$port_dir:/port:ro" \
    -w /sdk/os/kernel catplay-anyka-toolchain:bookworm sh -ec '
    make O=/output/kernel-build ARCH=arm CROSS_COMPILE=arm-linux-gnueabi- M=/output/loader modules
    arm-linux-gnueabi-gcc -Os -static -march=armv5te -marm -mfloat-abi=soft \
        -Wall -Wextra -Werror -o /output/catplay-kexec /port/loader/client.c
    '
echo "Built $output_dir/loader/catplay_anyka_kexec.ko and $output_dir/catplay-kexec"
