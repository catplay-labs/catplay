#!/bin/sh
# Run on the dongle after loading the Bluetooth modules and attaching UART1.
set -eu
bt=/tmp/catplay-bt
loader=/tmp/catplay-c2a/lib/ld-linux.so.3
libraries=$bt/lib:/tmp/catplay-c2a/lib
export DBUS_SYSTEM_BUS_ADDRESS=unix:path=$bt/system_bus_socket
mkdir -p /var/lib/dbus
if ! test -f /var/lib/dbus/machine-id; then
    cp "$bt/machine-id" /var/lib/dbus/machine-id
fi
if ! test -S "$bt/system_bus_socket"; then
    LD_PRELOAD=$bt/lib/libstock-clock.so "$loader" --library-path "$libraries" \
        "$bt/dbus-daemon" --config-file="$bt/system.conf" --nofork --nopidfile --nosyslog \
        >/tmp/catplay-dbus.log 2>&1 </dev/null &
    echo $! >/tmp/catplay-dbus.pid
    attempts=0
    until test -S "$bt/system_bus_socket"; do
        attempts=$((attempts + 1))
        if test "$attempts" -ge 5; then cat /tmp/catplay-dbus.log; exit 1; fi
        sleep 1
    done
fi
if test -f /tmp/catplay-bluetoothd.pid && kill -0 "$(cat /tmp/catplay-bluetoothd.pid)" 2>/dev/null; then
    exit 0
fi
LD_PRELOAD=$bt/lib/libstock-clock.so "$loader" --library-path "$libraries" \
    "$bt/bluetoothd" --nodetach --noplugin='*' \
    >/tmp/catplay-bluetoothd.log 2>&1 </dev/null &
echo $! >/tmp/catplay-bluetoothd.pid
