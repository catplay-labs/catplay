#!/usr/bin/env python3
"""Start an authenticated RAM server over a temporary stock-kernel USB NCM link."""
import argparse
import json
import os
from pathlib import Path
import shlex

from device import Device


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--session', type=Path, required=True, help='Existing Wi-Fi RAM session')
    parser.add_argument('--build', type=Path, required=True, help='build-stock-usb.sh output directory')
    parser.add_argument('--output-session', type=Path, required=True)
    args = parser.parse_args()
    if args.output_session.exists():
        parser.error('Output session already exists')
    session = json.loads(args.session.read_text())
    device = Device(session['host'], session['port'], session['token'])
    identity = Device(session['host']).request('GET', '/cgi-bin/getversion.cgi')
    if b'platform:AK3918AN' not in identity or b'system_version:20260612170300CA' not in identity:
        parser.error('Modules have only been audited against AK3918AN firmware 20260612170300CA')
    device.run('test ! -e /sys/kernel/config/usb_gadget/catplay_probe && '
               'test ! -e /sys/module/u_ether && test ! -e /sys/module/usb_f_ncm')
    for name in ['u_ether', 'usb_f_ncm']:
        device.upload(f'/tmp/{name}.ko', (args.build / 'stock-usb' / f'{name}.ko').read_bytes())
    # Save the new credentials before changing USB so a timeout remains recoverable.
    usb = dict(session, host='192.168.51.2')
    fd = os.open(args.output_session, os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o600)
    with os.fdopen(fd, 'w') as output:
        json.dump(usb, output)
        output.write('\n')
    device.run('''set -eu
insmod /tmp/u_ether.ko
insmod /tmp/usb_f_ncm.ko
mkdir /sys/kernel/config/usb_gadget/catplay_probe
cd /sys/kernel/config/usb_gadget/catplay_probe
echo 0x0525 > idVendor
echo 0xa4a1 > idProduct
mkdir strings/0x409
echo CatPlay > strings/0x409/manufacturer
echo Anyka_NCM_Probe > strings/0x409/product
echo catplay-anyka-probe > strings/0x409/serialnumber
mkdir configs/c.1
mkdir functions/ncm.usb0
echo 02:43:50:00:00:02 > functions/ncm.usb0/dev_addr
echo 02:43:50:00:00:01 > functions/ncm.usb0/host_addr
ln -s /sys/kernel/config/usb_gadget/catplay_probe/functions/ncm.usb0 configs/c.1/ncm.usb0
echo '' > /sys/kernel/config/usb_gadget/anyka/UDC
echo 20200000.usb > UDC
ifconfig usb0 192.168.51.2 netmask 255.255.255.0 up
''')
    root = shlex.quote(session['root'])
    port = int(session['port'])
    device.run(f'(/bin/busybox httpd -f -p 192.168.51.2:{port} -h {root}/www '
               f'-c {root}/httpd.conf </dev/null >{root}/usb-httpd.log 2>&1 & '
               f'echo $! >{root}/usb-httpd.pid)')
    print('USB NCM gadget ready; Wi-Fi access remains available.')
    print('Reset the old USB device identity with usb-console.py reset if needed.')
    print('Configure the host USB interface as 192.168.51.1/24 without a default route.')
    print(f'USB RAM-server session: {args.output_session}')


if __name__ == '__main__':
    main()
