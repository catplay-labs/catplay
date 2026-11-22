#!/usr/bin/env python3
"""Capture reconnects and device logs without sending USB control requests."""
import argparse
from datetime import datetime, timezone
import json
import os
from pathlib import Path
import subprocess
import time

from device import Device

p = argparse.ArgumentParser(description=__doc__)
p.add_argument('--session', required=True, type=Path)
p.add_argument('--output', required=True, type=Path)
p.add_argument('--duration', type=float, default=3600)
p.add_argument('--interval', type=float, default=5)
a = p.parse_args()
a.output.mkdir(parents=True, exist_ok=True, mode=0o700)
os.chmod(a.output, 0o700)
session = json.loads(a.session.read_text())
d = Device(session['host'], session['port'], session['token'], timeout=3)
sections = {
    'status': 'cat /proc/uptime; cat /proc/sys/kernel/random/boot_id; '
              'cat /sys/class/iphone/default/status; '
              'cat /sys/class/udc/20200000.usb/state 2>/dev/null; cat /proc/modules',
    'processes': 'ps',
    'memory': 'cat /proc/meminfo',
    'kernel': 'dmesg',
    'application': 'cat /tmp/catplay.log 2>/dev/null',
    'bluetooth': 'cat /tmp/catplay-bluetoothd.log 2>/dev/null',
    'uart': 'cat /tmp/bluetooth-uart.log 2>/dev/null',
}
for source in ['events', 'kernel', 'application']:
    for suffix in ['', '.previous']:
        name = f'capture_{source}{suffix.replace(".", "_")}'
        sections[name] = f'cat /usr/data/catplay/capture/{source}{suffix}.log 2>/dev/null'
command = '\n'.join(f"printf '\\n__CATPLAY_CAPTURE_{name}__\\n'; {cmd}"
                    for name, cmd in sections.items()) + '\ntrue'
previous = {}


def timestamp():
    return datetime.now(timezone.utc).isoformat()


def event(kind, **details):
    line = json.dumps(dict(time=timestamp(), event=kind, **details))
    with (a.output / 'events.jsonl').open('a') as out:
        out.write(line + '\n')
    print(line, flush=True)


def usb_state():
    result = []
    for node in Path('/sys/bus/usb/devices').iterdir():
        try:
            serial = (node / 'serial').read_text().strip()
            if serial not in ['catplay-anyka-probe',
                              '00008130000E044E384B1D3ADDDDDDD000BADCAD']:
                continue
            result.append(dict(path=node.name, **{
                field: (node / field).read_text().strip()
                for field in ['idVendor', 'idProduct', 'busnum', 'devnum',
                              'bConfigurationValue']}))
        except (OSError, ValueError):
            continue
    return sorted(result, key=lambda item: item['path'])


host_log = (a.output / 'host-kernel.log').open('a')
journal = subprocess.Popen(['journalctl', '-k', '-f', '--since', 'now',
                            '--no-pager', '-o', 'short-iso'],
                           stdout=host_log, stderr=subprocess.STDOUT)
event('armed', output=str(a.output), duration=a.duration)
deadline = time.monotonic() + a.duration
connected = None
usb = None
try:
    while time.monotonic() < deadline:
        current_usb = usb_state()
        if current_usb != usb:
            event('usb', devices=current_usb)
            usb = current_usb
        try:
            raw = d.run(command)
            if connected is not True:
                event('device_connected')
            connected = True
            for index, name in enumerate(sections):
                marker = f'\n__CATPLAY_CAPTURE_{name}__\n'.encode()
                body = raw.split(marker, 1)[1]
                if index + 1 < len(sections):
                    next_name = list(sections)[index + 1]
                    body = body.split(f'\n__CATPLAY_CAPTURE_{next_name}__\n'.encode(), 1)[0]
                (a.output / f'{name}-latest.txt').write_bytes(body)
                old = previous.get(name, b'')
                if body == old:
                    continue
                append = body[len(old):] if body.startswith(old) else body
                with (a.output / f'{name}.log').open('ab') as out:
                    out.write(f'\n[{timestamp()}]\n'.encode())
                    out.write(append)
                previous[name] = body
        except (OSError, RuntimeError, IndexError) as error:
            if connected is not False:
                event('device_unreachable', reason=str(error)[:300])
            connected = False
        remaining = deadline - time.monotonic()
        if remaining > 0:
            time.sleep(min(a.interval, remaining))
finally:
    journal.terminate()
    journal.wait(timeout=5)
    host_log.close()
    event('stopped')
