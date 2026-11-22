#!/bin/sh
# g_iphone unbinds its gadget before invoking this helper.
set -eu
case "${1:-}" in
    host)
        if [ -d /sys/module/ak_udc ]; then
            rmmod ak_udc
        fi
        if [ ! -d /sys/module/ak_hcd ]; then
            insmod /usr/modules/ak_hcd.ko
        fi
        test -d /sys/bus/platform/drivers/ak-hshcd/20200000.usb
        # The car's NCM interface needs USB networking. Use the vendor's
        # in-tree modules; they match this kernel's USB core.
        for module in usbnet cdc_ncm; do
            if [ ! -d /sys/module/$module ] && [ -f /usr/modules/$module.ko ]; then
                insmod /usr/modules/$module.ko
            fi
        done
        ;;
    device)
        # Stop traffic before unloading: removing cdc_ncm with the link up and
        # transfers in flight has hung the vendor host controller.
        if [ -d /sys/class/net/usb0 ]; then
            ifconfig usb0 down 2>/dev/null || true
            usleep 200000
        fi
        for module in cdc_ncm usbnet; do
            if [ -d /sys/module/$module ]; then
                rmmod $module
            fi
        done
        if [ -d /sys/module/ak_hcd ]; then
            rmmod ak_hcd
        fi
        if [ ! -d /sys/module/ak_udc ]; then
            insmod /usr/modules/ak_udc.ko
        fi
        test -d /sys/class/udc/20200000.usb
        ;;
    *)
        echo "Usage: $0 host|device" >&2
        exit 2
        ;;
esac
