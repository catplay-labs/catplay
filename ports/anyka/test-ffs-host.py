#!/usr/bin/env python3
"""Verify zero-pattern bulk traffic against the SDK FunctionFS test, leaving NCM attached."""
import argparse
import ctypes
import os
from pathlib import Path


class BulkTransfer(ctypes.Structure):
    _fields_ = [('ep', ctypes.c_uint), ('length', ctypes.c_uint),
                ('timeout', ctypes.c_uint), ('data', ctypes.c_void_p)]


class ControlTransfer(ctypes.Structure):
    _fields_ = [('request_type', ctypes.c_ubyte), ('request', ctypes.c_ubyte),
                ('value', ctypes.c_ushort), ('index', ctypes.c_ushort),
                ('length', ctypes.c_ushort), ('timeout', ctypes.c_uint),
                ('data', ctypes.c_void_p)]


def ioctl_number(direction, number, size):
    return (direction << 30) | (size << 16) | (ord('U') << 8) | number


def find_interface(descriptors):
    current = None
    interfaces = {}
    offset = 0
    while offset < len(descriptors):
        length = descriptors[offset]
        if length < 2 or offset + length > len(descriptors):
            raise ValueError('Malformed USB descriptor stream')
        item = descriptors[offset:offset + length]
        if item[1] == 4:
            current = item[2] if length >= 9 and item[3] == 0 and item[5] == 255 else None
            if current is not None:
                interfaces[current] = []
        elif item[1] == 5 and current is not None and length >= 7 and (item[3] & 3) == 2:
            interfaces[current].append(item[2])
        offset += length
    candidates = [(number, eps) for number, eps in interfaces.items()
                  if len(eps) == 2 and any(ep & 128 for ep in eps) and any(not ep & 128 for ep in eps)]
    if len(candidates) != 1:
        raise ValueError('Expected one vendor interface with two bulk endpoints; reset the USB identity first')
    return candidates[0]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('device', type=Path, help='Explicit /dev/bus/usb/BUS/DEVICE')
    mode = parser.add_mutually_exclusive_group()
    mode.add_argument('--echo', action='store_true', help='Test the Rust echo gadget with varied payloads')
    mode.add_argument('--os-descriptor', action='store_true', help='Check iAP2 discovery descriptors instead of bulk traffic')
    args = parser.parse_args()
    path = args.device.resolve()
    if path.parent.parent != Path('/dev/bus/usb'):
        parser.error('Expected /dev/bus/usb/BUS/DEVICE')
    nodes = [node for node in Path('/sys/bus/usb/devices').iterdir()
             if (node / 'devnum').exists()
             and int((node / 'busnum').read_text()) == int(path.parent.name)
             and int((node / 'devnum').read_text()) == int(path.name)]
    if len(nodes) != 1:
        parser.error('USB device is no longer present')
    node = nodes[0]
    if ((node / 'idVendor').read_text().strip(), (node / 'idProduct').read_text().strip(),
        (node / 'serial').read_text().strip()) != ('0525', 'a4a1', 'catplay-anyka-probe'):
        parser.error('Unexpected development USB identity')
    interface, endpoints = find_interface((node / 'descriptors').read_bytes())
    libc = ctypes.CDLL(None, use_errno=True)
    libc.ioctl.argtypes = [ctypes.c_int, ctypes.c_ulong, ctypes.c_void_p]
    libc.ioctl.restype = ctypes.c_int

    def ioctl(fd, request, argument):
        result = libc.ioctl(fd, request, ctypes.byref(argument))
        if result < 0:
            error = ctypes.get_errno()
            raise OSError(error, os.strerror(error))
        return result

    fd = os.open(path, os.O_RDWR)
    number = ctypes.c_uint(interface)
    claimed = False
    try:
        if args.os_descriptor:
            def control(request_type, request, value, index):
                buffer = ctypes.create_string_buffer(255)
                transfer = ControlTransfer(request_type, request, value, index, len(buffer), 3000,
                                           ctypes.addressof(buffer))
                count = ioctl(fd, ioctl_number(3, 0, ctypes.sizeof(transfer)), transfer)
                return buffer.raw[:count]

            signature = control(0x80, 6, 0x03ee, 0)
            if len(signature) != 18 or signature[2:16] != 'MSFT100'.encode('utf-16le'):
                raise RuntimeError('Invalid Microsoft OS discovery string')
            descriptors = control(0xc0, signature[16], 0, 4)
            if len(descriptors) < 16 or int.from_bytes(descriptors[:4], 'little') != len(descriptors):
                raise RuntimeError('Invalid extended compatibility descriptor length')
            count = descriptors[8]
            entries = [descriptors[16 + i*24:40 + i*24] for i in range(count)]
            if not any(len(entry) == 24 and entry[0] == interface and entry[2:10] == b'iAP2\0\0\0\0'
                       for entry in entries):
                raise RuntimeError('iAP2 compatible ID missing for the vendor interface')
            print(f'iAP2 discovery descriptor verified for interface {interface}')
            return
        ioctl(fd, ioctl_number(2, 15, ctypes.sizeof(number)), number)
        claimed = True
        if args.echo:
            source = next(ep for ep in endpoints if ep & 128)
            sink = next(ep for ep in endpoints if not ep & 128)
            total = 0
            for size in [1, 63, 127, 511, 512] * 4:
                payload = os.urandom(size)
                outgoing = ctypes.create_string_buffer(payload, size)
                request = BulkTransfer(sink, size, 3000, ctypes.addressof(outgoing))
                if ioctl(fd, ioctl_number(3, 2, ctypes.sizeof(request)), request) != size:
                    raise RuntimeError('Short USB bulk write')
                incoming = ctypes.create_string_buffer(512)
                ctypes.memset(incoming, 0xA5, len(incoming))
                request = BulkTransfer(source, len(incoming), 3000, ctypes.addressof(incoming))
                count = ioctl(fd, ioctl_number(3, 2, ctypes.sizeof(request)), request)
                if count != size or incoming.raw[:count] != payload:
                    raise RuntimeError(f'USB echo mismatch for {size}-byte payload')
                total += size
            print(f'Rust FunctionFS interface {interface}: 20 varied payloads echoed byte for byte ({total} bytes)')
        else:
            for _ in range(4):
                for endpoint in sorted(endpoints):
                    buffer = ctypes.create_string_buffer(8192)
                    if endpoint & 128:
                        ctypes.memset(buffer, 0xA5, len(buffer))
                    transfer = BulkTransfer(endpoint, len(buffer), 3000, ctypes.addressof(buffer))
                    count = ioctl(fd, ioctl_number(3, 2, ctypes.sizeof(transfer)), transfer)
                    if count != len(buffer) or (endpoint & 128 and any(buffer.raw)):
                        raise RuntimeError(f'Bulk endpoint 0x{endpoint:02x}: invalid length or zero pattern')
            print(f'FunctionFS interface {interface}: 32 KiB in each direction passed the zero-pattern test')
    finally:
        if claimed:
            ioctl(fd, ioctl_number(2, 16, ctypes.sizeof(number)), number)
        os.close(fd)


if __name__ == '__main__':
    main()
