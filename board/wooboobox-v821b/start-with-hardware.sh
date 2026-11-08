#!/bin/sh
# Optional image wrapper. Keep the ELF basename for process-name compatibility.
# The module and DTB must have passed bring-up on the exact kernel/board first.
set -eu
HWCONF=/usr/share/catplay/wooboobox-hardware.toml
# carlinkit_otalib regenerates the main TOML immediately before starting us.
# Append only our public board settings; never print the generated credentials.
if [ -r "$HWCONF" ] && [ -w /etc/catplay/catplay.conf ]; then
    if ! grep -Eq '^[[:space:]]*\[hardware\]' /etc/catplay/catplay.conf; then
        cat "$HWCONF" >> /etc/catplay/catplay.conf
    fi
fi
CLOCK=/proc/device-tree/soc/catplay-spi1-clocks@42001000
if [ -r "$CLOCK/compatible" ] && tr '\000' '\n' < "$CLOCK/compatible" | grep -qx 'catplay,v821-spi1-clocks'; then
    if [ ! -d /sys/module/catplay_spi1_clocks ]; then
        insmod "/lib/modules/$(uname -r)/kernel/drivers/clk/catplay-spi1-clocks.ko" || echo 'CatPlay: optional SPI1 clocks unavailable' >&2
    fi
    attempt=0
    while [ ! -d /sys/bus/spi/devices/spi1.0 ] && [ "$attempt" -lt 20 ]; do
        sleep 0.1
        attempt=$((attempt + 1))
    done
fi
SPI=/sys/bus/spi/devices/spi1.0
if [ -r "$SPI/of_node/compatible" ] && tr '\000' '\n' < "$SPI/of_node/compatible" | grep -qx 'catplay,ws2812b'; then
    if [ ! -d /sys/module/spidev ]; then
        insmod "/lib/modules/$(uname -r)/kernel/drivers/spi/spidev.ko" || echo 'CatPlay: optional SPI module unavailable' >&2
    fi
    if [ -d /sys/bus/spi/drivers/spidev ] && [ ! -e "$SPI/driver" ]; then
        if ! { echo spidev > "$SPI/driver_override" && echo spi1.0 > /sys/bus/spi/drivers/spidev/bind; }; then
            echo 'CatPlay: optional LED binding failed' >&2
        fi
    fi
fi
exec /usr/libexec/catplay/catplay_c2a "$@"
