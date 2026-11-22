#!/bin/sh
# Run in your terminal: sudo can request your password without sharing it.
set -eu
port_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
if [ "$(id -u)" -ne 0 ]; then
    exec sudo -- "$port_dir/install-usb-access.sh" "$@"
fi
target=/etc/udev/rules.d/70-catplay-anyka.rules
if [ -e "$target" ] && ! cmp -s "$port_dir/70-catplay-anyka.rules" "$target"; then
    echo "Refusing to overwrite a different existing rule: $target" >&2
    exit 1
fi
install -o root -g root -m 0644 "$port_dir/70-catplay-anyka.rules" "$target"
udevadm control --reload-rules
udevadm trigger --action=change --subsystem-match=usb \
    --attr-match=idVendor=0525 --attr-match=idProduct=a4a1 \
    --attr-match=serial=catplay-anyka-probe
udevadm settle
echo 'Development USB access rule installed and applied.'
