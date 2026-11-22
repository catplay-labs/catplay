#!/usr/bin/env python3
"""Prepare private SDK copies with USB/network layouts audited against this unit."""
import argparse
from pathlib import Path
import subprocess

p = argparse.ArgumentParser(description=__doc__)
p.add_argument('sdk', type=Path)
p.add_argument('output', type=Path)
a = p.parse_args()
expected = 'a7c6e75c154db2b5c270603d76f85fa66e0a3127'
if subprocess.check_output(['git', '-C', str(a.sdk), 'rev-parse', 'HEAD'], text=True).strip() != expected:
    p.error('Unexpected SDK revision')
a.output.mkdir(parents=True, exist_ok=True)
kernel = a.sdk / 'os/kernel'

composite = (kernel / 'include/linux/usb/composite.h').read_text()
needle = '\tstruct usb_function_driver *fd;\n'
if composite.count(needle) != 1:
    p.error('Unexpected usb_function_instance declaration')
composite = composite.replace(needle, needle + '\tvoid *catplay_vendor_reserved; /* Stock slot at offset 80. */\n')
for member, offset in [('set_inst_name', 84), ('free_func_inst', 88)]:
    composite += f'\n_Static_assert(__builtin_offsetof(struct usb_function_instance, {member}) == {offset}, "stock USB instance ABI");\n'
composite += '\n_Static_assert(__builtin_offsetof(struct usb_function, fi) == 96, "stock USB function ABI");\n'
(a.output / 'stock-composite.h').write_text(composite)

netdevice = (kernel / 'include/linux/netdevice.h').read_text()
needle = ('#ifdef CONFIG_WIRELESS_EXT\n'
          '\tconst struct iw_handler_def *\twireless_handlers;\n'
          '\tstruct iw_public_data *\twireless_data;\n#endif\n')
if netdevice.count(needle) != 1:
    p.error('Unexpected net_device wireless handler declaration')
netdevice = netdevice.replace(needle, '/* No legacy wireless handler slots in the audited stock layout. */\n')
# These offsets come from the captured stock kernel's networking functions.
for member, offset in [('stats', 192), ('netdev_ops', 292), ('ethtool_ops', 296),
                       ('header_ops', 300), ('flags', 304), ('priv_flags', 308),
                       ('mtu', 320), ('type', 324), ('hard_header_len', 326),
                       ('addr_len', 367), ('dev_addr', 456), ('broadcast', 496),
                       ('tx_queue_len', 552), ('pcpu_refcnt', 592), ('dev', 632)]:
    netdevice += f'\n_Static_assert(__builtin_offsetof(struct net_device, {member}) == {offset}, "stock net_device {member} ABI");\n'
netdevice += '\n_Static_assert(((sizeof(struct net_device) + 31) & ~31) == 992, "stock net_device private area");\n'
(a.output / 'stock-netdevice.h').write_text(netdevice)
for source in ['u_ether', 'f_ncm']:
    (a.output / f'{source}.c').write_bytes((kernel / f'drivers/usb/gadget/function/{source}.c').read_bytes())
(a.output / 'Makefile').write_text('''obj-m := u_ether.o usb_f_ncm.o
usb_f_ncm-y := f_ncm.o
ccflags-y := -include $(src)/stock-netdevice.h -include $(src)/stock-composite.h
ccflags-y += -I$(srctree)/drivers/usb/gadget/ -I$(srctree)/drivers/usb/gadget/function/ -I$(srctree)/drivers/usb/gadget/udc/
''')
print(a.output)
