#!/usr/bin/env python3
"""Create a separate RAM-test DTB; never modify input or flash a device."""
import argparse
from pathlib import Path
import struct
import libfdt

p = argparse.ArgumentParser(description=__doc__)
p.add_argument("input", type=Path)
p.add_argument("output", type=Path)
a = p.parse_args()
if a.input.resolve() == a.output.resolve() or a.output.exists():
    p.error("output must be a new file, distinct from input")
dt = libfdt.Fdt(a.input.read_bytes())
compatible = bytes(dt.getprop(dt.path_offset("/"), "compatible")).split(b"\0")
if b"allwinner,v821b" not in compatible:
    p.error("requires the documented V821B board DTB")
dt.resize(dt.totalsize() + 8192)

def get(path, name):
    return bytes(dt.getprop(dt.path_offset(path), name))

def cells(data):
    return list(struct.unpack(">" + "I" * (len(data) // 4), data))

def prop(path, name, data):
    dt.setprop(dt.path_offset(path), name, data)

def ints(path, name, values):
    prop(path, name, struct.pack(">" + "I" * len(values), *values))

def add(parent, name):
    dt.add_subnode(dt.path_offset(parent), name)
    return parent.rstrip("/") + "/" + name

spi0 = "/soc/spi@44025000"
pio = "/soc/pinctrl@42000000"
# Derive phandles from the input, never reuse vendor-DTB phandle numbers.
clocks = cells(get(spi0, "clocks"))
resets = cells(get(spi0, "resets"))
irq = cells(get(spi0, "interrupts-extended"))
if len(clocks) != 4 or len(resets) != 2 or len(irq) != 3 or irq[1:] != [16, 4]:
    p.error("unexpected SPI0 clock/reset/interrupt layout")
if cells(get(spi0, "reg")) != [0, 0x44025000, 0, 0x1000]:
    p.error("unexpected SPI0 register layout")
highest = 0
node, depth = -1, -1
while True:
    node, depth = dt.next_node(node, depth)
    if node < 0 or depth < 0:
        break
    try:
        highest = max(highest, cells(bytes(dt.getprop(node, "phandle")))[0])
    except libfdt.FdtException:
        pass

pin = add(pio, "catplay-spi1-led-pin")
prop(pin, "pins", b"PD13\0")
prop(pin, "function", b"spi1\0")
ints(pin, "phandle", [highest + 1])
ccu = add("/soc", "catplay-spi1-clocks@42001000")
prop(ccu, "compatible", b"catplay,v821-spi1-clocks\0")
ints(ccu, "reg", [0, 0x42001000, 0, 0x100])
ints(ccu, "#clock-cells", [1])
ints(ccu, "#reset-cells", [1])
ints(ccu, "phandle", [highest + 2])
spi = add("/soc", "spi@44026000")
# V821 SPI v1.2 needs its bit clock selected in the CCU (R329 model), not
# the older H3 controller's internal divider. Verified with RAM SPI transfers.
prop(spi, "compatible", b"allwinner,sun300i-v821-spi\0allwinner,sun50i-r329-spi\0")
ints(spi, "reg", [0, 0x44026000, 0, 0x1000])
ints(spi, "interrupts-extended", [irq[0], 17, 4])
ints(spi, "clocks", [highest + 2, 0, highest + 2, 1])
prop(spi, "clock-names", b"ahb\0mod\0")
ints(spi, "resets", [highest + 2, 0])
ints(spi, "#address-cells", [1])
ints(spi, "#size-cells", [0])
ints(spi, "num-cs", [1])
ints(spi, "pinctrl-0", [highest + 1])
prop(spi, "pinctrl-names", b"default\0")
prop(spi, "status", b"okay\0")
led = add(spi, "catplay-rgb@0")
prop(led, "compatible", b"catplay,ws2812b\0")
ints(led, "reg", [0])
ints(led, "spi-max-frequency", [2_400_000])
try:
    dt.path_offset("/aliases")
except libfdt.FdtException:
    add("/", "aliases")
prop("/aliases", "spi1", (spi + "\0").encode())
dt.pack()
a.output.write_bytes(bytes(dt.as_bytearray()))
print(f"Created separate SPI1 RAM-test DTB: {a.output}")
