/* SPDX-License-Identifier: GPL-2.0 */
#ifndef IAP2_USB_BULK_H
#define IAP2_USB_BULK_H

#include <linux/types.h>

struct usb_interface;
struct device;

int iap2_usb_bulk_devnode_path(struct usb_interface *intf, char *buf, size_t size);
int iap2_usb_bulk_link_session(struct usb_interface *intf, struct device *owner);

#endif
