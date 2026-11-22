#!/usr/bin/env python3
"""Build a private bootable stock-kernel filesystem with the CatPlay runtime."""
import argparse
import hashlib
import json
from pathlib import Path
import shutil
import secrets
import subprocess
import tarfile
import uuid

p = argparse.ArgumentParser(description=__doc__)
p.add_argument('--stock', type=Path, required=True)
p.add_argument('--app', type=Path, required=True)
p.add_argument('--bluetooth', type=Path, required=True)
p.add_argument('--modules', type=Path, required=True)
p.add_argument('--bluetooth-modules', type=Path, required=True)
p.add_argument('--net-modules', type=Path, required=True)
p.add_argument('--uart', type=Path, required=True)
p.add_argument('--output', type=Path, required=True)
p.add_argument('--diagnostic-identity', action='store_true')
a = p.parse_args()
a.output.mkdir(parents=True, exist_ok=True)
stock = a.output / 'stock'
root = a.output / 'root'
if stock.exists() or root.exists():
    p.error('Use a new output directory')
subprocess.run(['unsquashfs', '-d', str(stock), str(a.stock)], check=True)
root.mkdir()

def copy(source, dest):
    dest.parent.mkdir(parents=True, exist_ok=True)
    if source.is_dir():
        shutil.copytree(source, dest, symlinks=True, dirs_exist_ok=True)
    else:
        shutil.copy2(source, dest, follow_symlinks=False)

for directory in ['bin', 'sbin', 'lib', 'usr/modules']:
    copy(stock / directory, root / directory)
for pattern in ['libstdc++*', 'libiconv*', 'libcharset*', 'libitm*', 'libtirpc*',
                'libatomic*', 'libiw*', 'libprop*']:
    for file in (root / 'lib').glob(pattern):
        file.unlink()
(root / 'sbin/update.sh').unlink()
# Retain the BusyBox applets; omit proprietary application services.
for directory in ['usr/bin', 'usr/sbin']:
    (root / directory).mkdir(parents=True, exist_ok=True)
    for file in (stock / directory).iterdir():
        if file.is_symlink():
            copy(file, root / directory / file.name)
for name in ['hostapd']:
    copy(stock / 'usr/sbin' / name, root / 'usr/sbin' / name)
for file in (stock / 'usr/lib').glob('libnl*.so*'):
    copy(file, root / 'usr/lib' / file.name)
for directory in ['dev', 'proc', 'sys', 'tmp', 'var', 'mnt', 'etc', 'usr/data']:
    (root / directory).mkdir(parents=True, exist_ok=True)
copy(a.app, root / 'opt/catplay')
# The existing DATA partition holds the smaller Bluetooth userspace bundle.
# Map bluetoothd from squashfs; expand its supporting libraries into RAM.
bt = a.output / 'bluetooth'
copy(a.bluetooth, bt)
copy(bt / 'bluetoothd', root / 'opt/catplay-bluetoothd')
(bt / 'bluetoothd').unlink()
(bt / 'bluetoothd').symlink_to('/opt/catplay-bluetoothd')
for name in ['btmgmt', 'lib/libreadline.so.8', 'lib/libtinfo.so.6']:
    (bt / name).unlink(missing_ok=True)
archive = a.output / 'catplay-bt.tar.gz'
with tarfile.open(archive, 'w:gz') as tar:
    for item in sorted(bt.iterdir()):
        tar.add(item, arcname=item.name)
port = Path(__file__).resolve().parent
copy(port / 'usb-role.sh', root / 'usr/bin/catplay-usb-role')
copy(a.uart, root / 'usr/bin/bluetooth-uart')
copy(port / 'boot-catplay.sh', root / 'etc/init.d/rcS')
copy(port / 'run-persistent.sh', root / 'opt/catplay/run-persistent')
for path in ['usr/bin/catplay-usb-role', 'usr/bin/bluetooth-uart',
             'etc/init.d/rcS', 'opt/catplay/run-persistent']:
    (root / path).chmod(0o755)
for name in ['g_iphone', 'iap2_char', 'iap2_hid', 'iap2_usb_bulk', 'iap2_scan']:
    copy(a.modules / (name + '.ko'), root / 'usr/modules' / (name + '.ko'))
# The stock kernel lacks USB networking; the car's NCM interface needs it
# after a role switch.
for name in ['usbnet', 'cdc_ncm']:
    copy(a.net_modules / (name + '.ko'), root / 'usr/modules' / (name + '.ko'))
for name in ['crc16', 'cmac', 'ecb', 'bluetooth', 'rfcomm', 'hci_uart']:
    matches = list(a.bluetooth_modules.rglob(name + '.ko'))
    if len(matches) != 1:
        raise RuntimeError(f'Expected one module: {name}')
    copy(matches[0], root / 'usr/modules' / (name + '.ko'))
config = (port / 'c2a-bringup.toml').read_text().replace(
    '/tmp/catplay-c2a/state', '/usr/data/catplay/state')
(root / 'opt/catplay/c2a.toml').write_text(config)
(root / 'etc/machine-id').write_text(uuid.uuid4().hex + '\n')
(root / 'etc/catplay-identity').write_text('1\n' if a.diagnostic_identity else '0\n')
(root / 'etc/inittab').write_text('''::sysinit:/etc/init.d/rcS
ttySAK0::respawn:/bin/sh
::respawn:/opt/catplay/run-persistent
::restart:/sbin/init
::ctrlaltdel:/sbin/reboot
::shutdown:/bin/umount -a -r
''')
(root / 'etc/passwd').write_text('root:x:0:0:root:/root:/bin/sh\n')
(root / 'etc/group').write_text('root:x:0:\n')
(root / 'etc/hostapd.conf').write_text('''interface=wlan0
driver=nl80211
ctrl_interface=/var/run/hostapd
ssid=CatPlay-Anyka
hw_mode=a
channel=36
ieee80211n=1
ieee80211ac=1
wpa=2
wpa_key_mgmt=WPA-PSK
wpa_passphrase=catplay-dev
rsn_pairwise=CCMP
''')
(root / 'etc/udhcpd.conf').write_text('''start 192.168.50.101
end 192.168.50.254
interface wlan0
lease_file /var/udhcpd.leases
option subnet 255.255.255.0
option lease 86400
''')
token = secrets.token_hex(24)
(root / 'etc/catplay-httpd.conf').write_text(f'/:catplay:{token}\n')
(root / 'etc/catplay-httpd.conf').chmod(0o600)
cgi = root / 'opt/console/cgi-bin/run.cgi'
cgi.parent.mkdir(parents=True)
cgi.write_text('#!/bin/sh\nprintf "Content-Type: application/octet-stream\\r\\n\\r\\n"\nexec /bin/sh 2>&1\n')
cgi.chmod(0o755)
session = a.output / 'device-session.json'
session.write_text(json.dumps(dict(host='192.168.50.100', port=8081, token=token,
                                   root='/opt/console')) + '\n')
session.chmod(0o600)
# Keep flash binaries directly mapped, rather than copied into tmpfs.
image = a.output / 'catplay-rootfs.squashfs'
subprocess.run(['mksquashfs', str(root), str(image), '-noappend', '-all-root',
                '-comp', 'xz', '-b', '131072', '-processors', '2',
                '-no-progress'], check=True)
if image.stat().st_size > 0x880000:
    raise RuntimeError(f'Image exceeds ROOTFS partition: {image.stat().st_size} bytes')
manifest = dict(size=image.stat().st_size,
                sha256=hashlib.sha256(image.read_bytes()).hexdigest(),
                partition='ROOTFS', partition_size=0x880000,
                diagnostic_identity=a.diagnostic_identity,
                bluetooth_size=archive.stat().st_size,
                bluetooth_sha256=hashlib.sha256(archive.read_bytes()).hexdigest())
(a.output / 'manifest.json').write_text(json.dumps(manifest, indent=2) + '\n')
print(json.dumps(manifest, indent=2))
