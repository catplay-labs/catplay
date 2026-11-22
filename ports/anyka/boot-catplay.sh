#!/bin/sh
# Stock Anyka kernel, CatPlay userspace.
set -eu
export PATH=/bin:/sbin:/usr/bin:/usr/sbin
mount -t proc proc /proc
mount -t sysfs sysfs /sys
mount -t tmpfs -o size=24m tmpfs /tmp
mount -t tmpfs -o size=4m tmpfs /var
mkdir -p /dev/pts /var/run /var/lib/dbus /sys/kernel/config
mount -t devpts devpts /dev/pts
mount -t configfs none /sys/kernel/config
mount -t jffs2 /dev/mtdblock6 /usr/data
mkdir -p /usr/data/catplay/state
mkdir -p /usr/data/catplay/bluetooth
ln -s /usr/data/catplay/bluetooth /var/lib/bluetooth
# The capture helper lives on DATA so it can change without rewriting ROOTFS.
capture=/usr/data/catplay/catplay-capture
if test "$(cat /usr/data/catplay/capture-enabled 2>/dev/null)" = 1 && test -x "$capture"; then
    "$capture" >/tmp/catplay-capture.log 2>&1 </dev/null &
    echo $! >/tmp/catplay-capture.pid
fi
hostname CatPlay-Anyka
ifconfig lo 127.0.0.1 up
echo 2048 >/proc/sys/vm/min_free_kbytes
insmod /usr/modules/ak_efuse.ko
insmod /usr/modules/cfg80211.ko
insmod /usr/modules/ak_mci.ko
echo 0 >/sys/mmc_en/mmc2_card_pwr_en
usleep 50000
echo 1 >/sys/mmc_en/mmc2_card_pwr_en
usleep 50000
uuid=$(sed -n '2p' /sys/kernel/efuse/efuse_OR | awk '{for(i=2;i<=7;i++) {gsub(/0x|,/,"",$i); printf "%s", $i}}')
insmod /usr/modules/aic8800_bsp.ko
insmod /usr/modules/aic8800_fdrv.ko efuse_data="$uuid"
for module in crc16 cmac ecb bluetooth rfcomm hci_uart; do
    insmod /usr/modules/$module.ko
done
/usr/bin/bluetooth-uart --up >/tmp/bluetooth-uart.log 2>&1 &
echo $! >/tmp/catplay-bluetooth-uart.pid
attempts=0
until test -d /sys/class/net/wlan0 && test -d /sys/class/bluetooth/hci0; do
    attempts=$((attempts + 1))
    if test "$attempts" -ge 20; then echo 'Radio startup timed out'; exit 1; fi
    sleep 1
done
ifconfig wlan0 192.168.50.100 netmask 255.255.255.0 up
hostapd -B /etc/hostapd.conf
udhcpd /etc/udhcpd.conf
/bin/busybox httpd -f -p 192.168.50.100:8081 -h /opt/console \
    -c /etc/catplay-httpd.conf >/tmp/catplay-httpd.log 2>&1 </dev/null &
# Writable daemon sockets remain in RAM; their binaries remain on flash.
ln -s /opt/catplay /tmp/catplay-c2a
mkdir -p /tmp/catplay-bt
tar xzf /usr/data/catplay-bt.tar.gz -C /tmp/catplay-bt
cp /etc/machine-id /var/lib/dbus/machine-id
loader=/opt/catplay/lib/ld-linux.so.3
libraries=/tmp/catplay-bt/lib:/opt/catplay/lib
export DBUS_SYSTEM_BUS_ADDRESS=unix:path=/tmp/catplay-bt/system_bus_socket
LD_PRELOAD=/tmp/catplay-bt/lib/libstock-clock.so "$loader" --library-path "$libraries" \
    /tmp/catplay-bt/dbus-daemon --config-file=/tmp/catplay-bt/system.conf \
    --nofork --nopidfile --nosyslog >/tmp/catplay-dbus.log 2>&1 &
attempts=0
until test -S /tmp/catplay-bt/system_bus_socket; do
    attempts=$((attempts + 1))
    if test "$attempts" -ge 5; then cat /tmp/catplay-dbus.log; exit 1; fi
    sleep 1
done
LD_PRELOAD=/tmp/catplay-bt/lib/libstock-clock.so "$loader" --library-path "$libraries" \
    /tmp/catplay-bt/bluetoothd --nodetach --noplugin='*' \
    >/tmp/catplay-bluetoothd.log 2>&1 &
insmod /usr/modules/ak_udc.ko
for module in iap2_char iap2_hid iap2_usb_bulk iap2_scan; do
    insmod /usr/modules/$module.ko
done
identity=$(cat /etc/catplay-identity)
if test -f /usr/data/catplay/diagnostic-identity; then
    identity=$(cat /usr/data/catplay/diagnostic-identity)
fi
case "$identity" in 0|1) ;; *) echo 'Invalid USB identity setting'; exit 1;; esac
insmod /usr/modules/g_iphone.ko diagnostic_identity="$identity" \
    role_helper=/usr/bin/catplay-usb-role
echo 'CatPlay boot services ready'
