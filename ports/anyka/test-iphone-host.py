#!/usr/bin/env python3
"""Check the development iPhone gadget's descriptors and early vendor requests."""
import argparse
import ctypes
import hashlib
import os
from pathlib import Path


class Control(ctypes.Structure):
    _fields_ = [('request_type', ctypes.c_ubyte), ('request', ctypes.c_ubyte),
                ('value', ctypes.c_ushort), ('index', ctypes.c_ushort),
                ('length', ctypes.c_ushort), ('timeout', ctypes.c_uint),
                ('data', ctypes.c_void_p)]


class DisconnectClaim(ctypes.Structure):
    _fields_ = [('interface', ctypes.c_uint), ('flags', ctypes.c_uint),
                ('driver', ctypes.c_char * 256)]


def ioctl_number(direction, number, size):
    return (direction << 30) | (size << 16) | (ord('U') << 8) | number


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('device', type=Path)
    parser.add_argument('--hid', action='store_true',
                        help='Select configuration 2 and check HID control requests')
    parser.add_argument('--role-switch', action='store_true',
                        help='Request host mode after checks; the USB link disconnects')
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
    identity = tuple((node / field).read_text().strip()
                     for field in ['idVendor', 'idProduct', 'serial'])
    if identity != ('0525', 'a4a1', 'catplay-anyka-probe'):
        parser.error('Use g_iphone diagnostic_identity=1; Apple identity is unsafe on this host kernel')
    original = int((node / 'bConfigurationValue').read_text().strip() or '0')
    libc = ctypes.CDLL(None, use_errno=True)
    libc.ioctl.argtypes = [ctypes.c_int, ctypes.c_ulong, ctypes.c_void_p]
    libc.ioctl.restype = ctypes.c_int
    fd = os.open(path, os.O_RDWR)
    claimed = False

    def ioctl(number, arg):
        result = libc.ioctl(fd, number, ctypes.byref(arg))
        if result < 0:
            error = ctypes.get_errno()
            raise OSError(error, os.strerror(error))
        return result

    def control(request_type, request, value=0, index=0, length=4):
        buf = ctypes.create_string_buffer(length)
        transfer = Control(request_type, request, value, index, length, 3000,
                           ctypes.addressof(buf))
        count = ioctl(ioctl_number(3, 0, ctypes.sizeof(transfer)), transfer)
        return buf.raw[:count]

    def configure(value):
        # Audio drivers claim interfaces as soon as configuration 2 appears.
        # Release those claims before changing configurations through usbfs.
        for interface in node.parent.glob(node.name + ':*'):
            if (interface / 'driver').exists():
                number = int((interface / 'bInterfaceNumber').read_text(), 16)
                claim = DisconnectClaim(number, 0, b'')
                ioctl(ioctl_number(2, 27, ctypes.sizeof(claim)), claim)
                ioctl(ioctl_number(2, 16, ctypes.sizeof(ctypes.c_uint)),
                      ctypes.c_uint(number))
        ioctl(ioctl_number(2, 5, ctypes.sizeof(ctypes.c_uint)), ctypes.c_uint(value))

    try:
        device = control(0x80, 6, 0x0100, length=18)
        if len(device) != 18 or device[17] != 4:
            raise RuntimeError('Expected four USB configurations')
        for index in range(4):
            header = control(0x80, 6, 0x0200 | index, length=9)
            length = int.from_bytes(header[2:4], 'little')
            data = control(0x80, 6, 0x0200 | index, length=length)
            if len(data) != length or data[5] != index + 1:
                raise RuntimeError(f'Invalid configuration {index + 1}')
            print(f'Configuration {index + 1}: {length} descriptor bytes, {data[4]} interfaces')
        configure(0)
        if control(0xc0, 0x53) != bytes([1, 0, 0, 0]):
            raise RuntimeError('Wrong capability response before configuration')
        print('Capabilities before SET_CONFIGURATION: 01000000')
        configure(1)
        if control(0xc0, 0x53) != bytes([1, 0, 0, 0]):
            raise RuntimeError('Wrong configured capability response')
        # A host-to-device vendor request with no data must complete its ZLP.
        if control(0x40, 0x40, value=500, length=0):
            raise RuntimeError('Unexpected power capability payload')
        print('Configured capabilities and power request passed')
        if args.hid:
            configure(2)
            # Detach/claim only this dongle's HID interface. Restoring the
            # configuration below reprobes its original kernel drivers.
            claim = DisconnectClaim(2, 0, b'')
            ioctl(ioctl_number(2, 27, ctypes.sizeof(claim)), claim)
            claimed = True
            report = control(0x81, 6, value=0x2200, index=2, length=4096)
            digest = hashlib.sha256(report).hexdigest()
            if len(report) != 208 or digest != '900fc1696e0d575469facc1b83008e711121c965d477f9a4839364f6ce680103':
                raise RuntimeError('HID report descriptor differs from CatPlay source')
            control(0x21, 0x0a, index=2, length=0)  # SET_IDLE
            if control(0xa1, 0x02, index=2, length=1) != b'\0':  # GET_IDLE
                raise RuntimeError('HID idle request did not complete')
            print('HID report matches all 208 source bytes; SET_IDLE/GET_IDLE passed')
    finally:
        try:
            if claimed:
                ioctl(ioctl_number(2, 16, ctypes.sizeof(ctypes.c_uint)), ctypes.c_uint(2))
            configure(original)
        finally:
            os.close(fd)
    if args.role_switch:
        fd = os.open(path, os.O_RDWR)
        try:
            control(0x40, 0x51, length=0)
            print('Role-switch request acknowledged; expect USB disconnection')
        finally:
            os.close(fd)


if __name__ == '__main__':
    main()
