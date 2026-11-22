#!/usr/bin/env python3
"""Reset the development dongle's USB link, or run a command on its USB console."""
import argparse
import fcntl
import os
from pathlib import Path
import select
import termios
import time
import tty


def reset(path):
    path = Path(path).resolve()
    if path.parent.parent != Path('/dev/bus/usb'):
        raise ValueError('Expected /dev/bus/usb/BUS/DEVICE')
    bus, number = int(path.parent.name), int(path.name)
    matches = []
    for entry in Path('/sys/bus/usb/devices').iterdir():
        if not (entry / 'devnum').exists():
            continue
        if int((entry / 'busnum').read_text()) == bus and int((entry / 'devnum').read_text()) == number:
            matches.append(entry)
    if len(matches) != 1:
        raise ValueError('USB device is no longer present')
    entry = matches[0]
    ids = ((entry / 'idVendor').read_text().strip(), (entry / 'idProduct').read_text().strip())
    if ids == ('05ac', '12a8'):
        if (entry / 'serial').read_text().strip() != 'c23e250022411234567890123456':
            raise ValueError('This is not the development dongle')
    elif ids == ('0525', 'a4a1'):
        if (entry / 'serial').read_text().strip() != 'catplay-anyka-probe':
            raise ValueError('This is not the development NCM gadget')
    elif ids != ('0525', 'a4a7'):
        raise ValueError(f'Unexpected USB identity: {ids}')
    fd = os.open(path, os.O_WRONLY)
    try:
        try:
            fcntl.ioctl(fd, 0x5514, 0)  # USBDEVFS_RESET
        except OSError as error:
            # Changed descriptors make Linux remove the stale device during reset.
            if error.errno != 19:
                raise
            print('Old USB identity removed; wait for gadget enumeration.')
    finally:
        os.close(fd)


def command(path, text, timeout):
    fd = os.open(path, os.O_RDWR | os.O_NOCTTY | os.O_NONBLOCK)
    try:
        tty.setraw(fd)
        termios.tcflush(fd, termios.TCIFLUSH)
        os.write(fd, text.encode() + b'\n')
        deadline = time.monotonic() + timeout
        while time.monotonic() < deadline:
            if select.select([fd], [], [], min(0.2, max(0, deadline-time.monotonic())))[0]:
                data = os.read(fd, 8192)
                if not data:
                    break
                print(data.decode(errors='replace'), end='', flush=True)
    finally:
        os.close(fd)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    sub = parser.add_subparsers(dest='action', required=True)
    p = sub.add_parser('reset', help='Reset only the specified development USB device')
    p.add_argument('device', help='/dev/bus/usb/BUS/DEVICE from lsusb')
    p = sub.add_parser('command', help='Send one shell command and collect output')
    p.add_argument('device', help='For example /dev/ttyACM0')
    p.add_argument('text')
    p.add_argument('--timeout', type=float, default=3)
    args = parser.parse_args()
    if args.action == 'reset':
        reset(args.device)
    else:
        command(args.device, args.text, args.timeout)


if __name__ == '__main__':
    main()
