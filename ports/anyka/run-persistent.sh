#!/bin/sh
# BusyBox init restarts this process if the application exits.
export DBUS_SYSTEM_BUS_ADDRESS=unix:path=/tmp/catplay-bt/system_bus_socket
# The AIC8800 controller fails Secure Connections pairing with iPhones.
export CATPLAY_BT_SECURE_CONNECTIONS=0
if test "$(cat /usr/data/catplay/capture-enabled 2>/dev/null)" = 1; then
    export RUST_BACKTRACE=1
fi
cd /opt/catplay || exit 1
./run >>/tmp/catplay.log 2>&1 &
app_pid=$!
echo "$app_pid" >/tmp/catplay-c2a.pid
printf 'LAUNCH uptime=%s pid=%s\n' "$(cat /proc/uptime)" "$app_pid" >>/tmp/catplay.log
wait "$app_pid"
result=$?
printf 'EXIT uptime=%s pid=%s status=%s\n' "$(cat /proc/uptime)" "$app_pid" "$result" >>/tmp/catplay.log
sleep 2
