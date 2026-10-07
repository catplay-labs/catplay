/* SPDX-License-Identifier: GPL-2.0 */
#ifndef IPHONE_HID_H
#define IPHONE_HID_H
#include <linux/types.h>
struct usb_function;
struct usb_endpoint_descriptor;
struct usb_ctrlrequest;
struct iphone_hid;
struct iphone_hid *iphone_hid_bind(struct usb_function *f, const struct usb_endpoint_descriptor *desc, const u8 *report_desc, size_t report_len);
int iphone_hid_enable(struct iphone_hid *hid);
void iphone_hid_disable(struct iphone_hid *hid);
int iphone_hid_setup(struct iphone_hid *hid, const struct usb_ctrlrequest *ctrl);
/* Publish the first terminal reason and schedule stop; unbind joins teardown. */
void iphone_hid_close(struct iphone_hid *hid, int reason);
void iphone_hid_unbind(struct iphone_hid *hid, int reason);
#endif
