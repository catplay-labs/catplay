#!/usr/bin/env python3
"""Create a private initramfs manifest from the local stock-firmware backup."""
import argparse
from pathlib import Path

p=argparse.ArgumentParser(description=__doc__)
p.add_argument('stock_rootfs',type=Path)
p.add_argument('output',type=Path)
a=p.parse_args()
a.output.mkdir(parents=True,exist_ok=True)
# The kernel build runs in Docker with stock files under /work/rootfs and port source /port.
lines=['dir /dev 0755 0 0','dir /proc 0755 0 0','dir /sys 0755 0 0',
       'dir /tmp 1777 0 0','dir /bin 0755 0 0','dir /sbin 0755 0 0','dir /lib 0755 0 0',
       'nod /dev/console 0600 0 0 c 5 1','file /init /port/ramboot/init 0755 0 0',
       'file /bin/busybox /work/rootfs/bin/busybox 0755 0 0',
       'file /bin/usb-shell /output/usb-shell 0755 0 0']
for applet in ['sh','mount','insmod','cat','echo','sleep','ls','dmesg','id','grep','reboot']:
 lines.append(f'slink /bin/{applet} busybox 0777 0 0')
for name in ['ld-uClibc-1.0.31.so','libuClibc-1.0.31.so']:
 if not (a.stock_rootfs/'lib'/name).is_file():raise SystemExit(f'Missing stock library: {name}')
 lines.append(f'file /lib/{name} /work/rootfs/lib/{name} 0755 0 0')
lines+=['slink /lib/ld-uClibc.so.0 ld-uClibc-1.0.31.so 0777 0 0',
        'slink /lib/libc.so.0 libuClibc-1.0.31.so 0777 0 0',
        'file /lib/ak_udc.ko /work/rootfs/usr/modules/ak_udc.ko 0600 0 0']
(a.output/'initramfs.list').write_text('\n'.join(lines)+'\n')
print(a.output/'initramfs.list')
