#!/usr/bin/env python3
"""Stage CatPlay's iPhone driver with the pinned Linux 4.4 composite core."""
import argparse
from pathlib import Path
import re
import shutil
import subprocess

p = argparse.ArgumentParser(description=__doc__)
p.add_argument('sdk', type=Path)
p.add_argument('output', type=Path)
a = p.parse_args()
if subprocess.check_output(['git', '-C', str(a.sdk), 'rev-parse', 'HEAD'], text=True).strip() != 'a7c6e75c154db2b5c270603d76f85fa66e0a3127':
    p.error('Unexpected SDK revision')
port = Path(__file__).resolve().parent
source = port.parents[1] / 'usb/catplay_iap2_usb_host/g_iphone'
shutil.copytree(source, a.output, dirs_exist_ok=True,
                ignore=shutil.ignore_patterns('dist', '*.o', '*.ko', '*.cmd', '*.mod.c'))
kernel = a.sdk / 'os/kernel'

def replace(text, before, after):
    if text.count(before) != 1:
        raise ValueError(f'Expected exactly one occurrence of {before!r}')
    return text.replace(before, after)

header = (kernel / 'include/linux/usb/composite.h').read_text()
header = replace(header, '\tstruct usb_function_driver *fd;\n',
                 '\tstruct usb_function_driver *fd;\n\tvoid *catplay_vendor_reserved;\n')
# The new callback argument changes no layout; this private core owns calls.
header = replace(header, 'const struct usb_ctrlrequest *);\n\tvoid\t\t\t(*suspend)',
                 'const struct usb_ctrlrequest *, bool config0);\n\tvoid\t\t\t(*suspend)')
header = replace(header, '\tstruct usb_gadget_driver\t\tgadget_driver;\n',
                 '\tstruct usb_gadget_driver\t\tgadget_driver;\n\tchar **udc_name;\n')
header = header.replace('usb_composite_probe', 'c2a_usb_composite_probe')
header += '\n_Static_assert(__builtin_offsetof(struct usb_function, fi) == 96, "stock USB function ABI");\n'
(a.output / 'gadget/libcomposite/composite.h').write_text(header)

core = (kernel / 'drivers/usb/gadget/composite.c').read_text()
core = replace(core, '#include <linux/usb/composite.h>', '#include "composite.h"')
core = re.sub(r'^EXPORT_SYMBOL_GPL\([^\n]+\);$', '', core, flags=re.M)
core = core.replace('usb_composite_probe', 'c2a_usb_composite_probe')
core = core.replace('usb_gadget_set_selfpowered(gadget);', '(void)0; /* CatPlay reports bus power. */')
core = replace(core, 'cpu_to_le16(get_default_bcdDevice());', 'cpu_to_le16(0x1602);')
core = replace(core, 'f->req_match(f, ctrl)', 'f->req_match(f, ctrl, false)')
needle = '\t\t\tf = NULL;\n\t\t}\n\n\t\tswitch (ctrl->bRequestType & USB_RECIP_MASK)'
core = replace(core, needle, '''\t\t} else {
\t\t\tstruct usb_configuration *c;
\t\t\tlist_for_each_entry(c, &cdev->configs, list)
\t\t\t\tlist_for_each_entry(f, &c->functions, list)
\t\t\t\t\tif (f->req_match && f->req_match(f, ctrl, true))
\t\t\t\t\t\tgoto try_fun_setup;
\t\t}
\t\tf = NULL;

\t\tswitch (ctrl->bRequestType & USB_RECIP_MASK)''')
# This unit has one controller; reject names for other devices instead of
# silently accepting an unsupported per-driver UDC selection API.
core = replace(core, '\treturn usb_gadget_probe_driver(gadget_driver);', '''\tif (driver->udc_name && *driver->udc_name &&
\t    strcmp(*driver->udc_name, "20200000.usb"))
\t\treturn -ENODEV;
\treturn usb_gadget_probe_driver(gadget_driver);''')
(a.output / 'gadget/libcomposite/composite.c').write_text(core)
shutil.copyfile(kernel / 'drivers/usb/gadget/u_os_desc.h', a.output / 'gadget/libcomposite/u_os_desc.h')

# Keep every function compiled against the same private composite layout.
for f in a.output.rglob('*'):
    if f.suffix not in ['.h', '.c'] or 'libcomposite' in f.parts:
        continue
    text = f.read_text()
    text = text.replace('#include <linux/usb/composite.h>', '#include "libcomposite/composite.h"')
    text = text.replace('#include <linux/container_of.h>\n', '')
    text = text.replace('#include <linux/device/driver.h>\n', '')
    text = text.replace('#include <linux/usb/cdc_ncm.h>', 'struct usbnet;\n#include <linux/usb/cdc_ncm.h>')
    text = text.replace('#include <linux/unaligned.h>', '#include <asm/unaligned.h>')
    text = text.replace('#include <linux/usb/role.h>', '#include "anyka-role.h"')
    text = text.replace('const struct class *cls', 'struct class *cls').replace('const struct class_attribute *attr', 'struct class_attribute *attr')
    text = re.sub(r'class_create\("([^"\n]+)"\)', r'class_create(THIS_MODULE, "\1")', text)
    f.write_text(text)

# Networking structures must use the previously audited stock layout.
subprocess.run(['python3', str(port / 'prepare-stock-usb.py'), str(a.sdk), str(a.output / 'abi')], check=True)
shutil.copyfile(port / 'stock-iap2-compat.h', a.output / 'iap2-compat.h')
shutil.copyfile(port / 'stock-iphone-compat.h', a.output / 'compat.h')
shutil.copyfile(port / 'stock-iphone-role.c', a.output / 'gadget/iphone_rs.c')
(a.output / 'gadget/anyka-role.h').write_text('''#pragma once
enum usb_role { USB_ROLE_NONE, USB_ROLE_HOST, USB_ROLE_DEVICE };
''')
# Linux 4.4 cannot disable work items. Serialize the terminal flag with all
# external producers, join the USB callbacks, then drain the queue again.
f = a.output / 'gadget/iphone_dev.c'
text = f.read_text()
text = '#include <linux/delay.h>\n' + text
# ak_udc does not complete zero-length EP0 requests. Allow its status packet
# to reach the host before the role worker removes the controller.
text = replace(text, '\tif (iphone_dev_set_otg_role_internal(data, USB_ROLE_HOST, true)) {',
               '\tmsleep(100);\n\tif (READ_ONCE(data->runtime_stopped))\n\t\tgoto done;\n'
               '\tif (iphone_dev_set_otg_role_internal(data, USB_ROLE_HOST, true)) {')
text = replace(text, '\tdata->dev_desc = iphone_device_desc;', '''\tdata->dev_desc = iphone_device_desc;
\tif (catplay_diagnostic_identity) {
\t\tdata->dev_desc.idVendor = cpu_to_le16(0x0525);
\t\tdata->dev_desc.idProduct = cpu_to_le16(0xa4a1);
\t\tserial = "catplay-anyka-probe";
\t}''')
text = replace(text, '\tqueue_work(data->wq, &data->status_notify_work);', '''\tunsigned long flags;
\tspin_lock_irqsave(&data->request_lock, flags);
\tif (!data->runtime_stopped)
\t\tqueue_work(data->wq, &data->status_notify_work);
\tspin_unlock_irqrestore(&data->request_lock, flags);''')
text = replace(text, 'if (!data->recovery_command) {', 'if (!data->runtime_stopped && !data->recovery_command) {')
text = replace(text, 'if (!data->role_switch_work_scheduled) {', 'if (!data->runtime_stopped && !data->role_switch_work_scheduled) {')
start = text.index('\tif (data->runtime_stopped)\n', text.index('static void iphone_dev_disable_runtime(struct iphone_dev_data *data)\n{'))
end = text.index('\n}', start)
text = text[:start] + '''\tunsigned long flags;
\tspin_lock_irqsave(&data->request_lock, flags);
\tWRITE_ONCE(data->runtime_stopped, true);
\tspin_unlock_irqrestore(&data->request_lock, flags);''' + text[end:]
text = replace(text, '\tiphone_dev_disable_runtime(data);\n\tflush_workqueue(data->wq);',
               '\tiphone_dev_disable_runtime(data);\n\tcancel_delayed_work_sync(&data->maintenance_work);\n\tflush_workqueue(data->wq);')
text = replace(text, '''\t/* Keep wq and data alive while unregister joins the USB producers.
\t * Their late queue attempts are harmless: all work stays disabled.
\t */''', '''\t/* Keep data alive until unregister joins the USB producers. */''')
text = replace(text, '\tdestroy_workqueue(data->wq);', '''\t/* An already-running worker may have scheduled its delayed follow-up. */
\tcancel_delayed_work_sync(&data->maintenance_work);
\tflush_workqueue(data->wq);
\tdestroy_workqueue(data->wq);''')
f.write_text(text)
f = a.output / 'gadget/iphone_mod.c'
text = f.read_text()
text += '''
/* Use the existing development identity to avoid the host Apple charger
 * driver while exercising configuration and HID requests. */
bool catplay_diagnostic_identity;
module_param_named(diagnostic_identity, catplay_diagnostic_identity, bool, 0400);
MODULE_PARM_DESC(diagnostic_identity, "Use the Anyka development VID/PID/serial");
'''
f.write_text(text)
f = a.output / 'gadget/f_iphone.c'
text = f.read_text()
text = replace(text, 'NULL, NULL);', 'NULL);')
f.write_text(text)
f = a.output / 'scan/iap2_scan.c'
text = f.read_text()
start = text.index('    drv = driver_find(driver_name, dev->bus);')
end = text.index('\n}', start)
text = text[:start] + '''    /* device_attach holds the device lock while matching/probing drivers. */
    ret = device_attach(dev);
    if (ret < 0)
        return ret;
    device_lock(dev);
    ret = driver_is_bound(intf, driver_name) ? 0 : -ENODEV;
    device_unlock(dev);
    return ret;''' + text[end:]
text = text.replace('    struct device_driver *drv;\n', '')
f.write_text(text)
f = a.output / 'gadget/iphone_hid.c'
text = f.read_text()
text = replace(text, '    hid->ctrl_pending = true;\n    ret = usb_ep_queue', '''    /* ak_udc acknowledges EP0 ZLPs without completing their request. Use
     * the composite-owned request so this HID request stays available. */
    if (!req->length) {
        struct usb_request *ack = hid->func->config->cdev->req;
        ack->length = 0;
        ack->zero = 0;
        ret = usb_ep_queue(hid->func->config->cdev->gadget->ep0, ack, GFP_ATOMIC);
        goto out;
    }
    hid->ctrl_pending = true;
    ret = usb_ep_queue''')
f.write_text(text)
f = a.output / 'gadget/iphone_audio.c'
text = f.read_text()
helper = '''
/* This controller supports full/high speed. Select the endpoint within the
 * requested alternate setting; never reuse an endpoint from another alt. */
static int config_ep_by_speed_and_alt(struct usb_gadget *g,
                                     struct usb_function *f,
                                     struct usb_ep *ep, u8 alt)
{
    struct usb_descriptor_header **d;
    bool selected = false;
    if (g->speed != USB_SPEED_HIGH && g->speed != USB_SPEED_FULL)
        return -EOPNOTSUPP;
    d = g->speed == USB_SPEED_HIGH ? f->hs_descriptors : f->fs_descriptors;
    if (!d)
        return -EINVAL;
    for (; *d; d++) {
        if ((*d)->bDescriptorType == USB_DT_INTERFACE)
            selected = ((struct usb_interface_descriptor *)*d)->bAlternateSetting == alt;
        else if (selected && (*d)->bDescriptorType == USB_DT_ENDPOINT) {
            struct usb_endpoint_descriptor *desc = (void *)*d;
            if (desc->bEndpointAddress != ep->address)
                continue;
            ep->desc = desc;
            ep->maxpacket = usb_endpoint_maxp(desc) & 0x7ff;
            ep->mult = g->speed == USB_SPEED_HIGH ?
                ((usb_endpoint_maxp(desc) >> 11) & 3) + 1 : 1;
            ep->maxburst = 0;
            ep->comp_desc = NULL;
            return 0;
        }
    }
    return -EINVAL;
}
'''
text = replace(text, '#define AUDIO_TX_REQUESTS 8', helper + '\n#define AUDIO_TX_REQUESTS 8')
f.write_text(text)
makefile = (a.output / 'Makefile').read_text()
makefile += '\nccflags-y := -std=gnu11 -Wno-declaration-after-statement -include $(src)/compat.h\n'
makefile += 'ccflags-y += -include $(src)/abi/stock-netdevice.h -include linux/mii.h -include linux/ethtool.h\n'
makefile += 'ccflags-y += -I$(srctree)/drivers/usb/gadget/\n'
(a.output / 'Makefile').write_text(makefile)
print(a.output)
