// SPDX-License-Identifier: GPL-2.0
#include "f_iphone.h"
#include "interfaces.h"
#include "hid.h"
#include "iphone_hid.h"

#define IPHONE_REQ_USBOOT 0x88
#define IPHONE_REQ_GADGET 0x99

static struct g_iphone *g_iphone_from_func(struct usb_function *f)
{
	struct f_iphone *iphone_func = container_of(f, struct f_iphone, func);
	return iphone_func->g;
}

static struct g_iphone *g_iphone_from_usb_config(struct usb_configuration *f)
{
	struct f_iphone_usb_config *iphone_func = container_of(f, struct f_iphone_usb_config, cfg);
	return iphone_func->g;
}

/* ----------------- Bind/Unbind ----------------- */
static int f_iphone_bind(struct usb_configuration *c, struct usb_function *f)
{
	int config = c->bConfigurationValue;
	int ret;
	pr_debug("iPhone: binding config %u", config);

	struct g_iphone *iphone_gadget = g_iphone_from_func(f);
	if (!iphone_gadget)
		return -ENODEV;

	if (config < 1 || config > 4)
	{
		pr_err("iPhone: no descriptors for config %u", config);
		return -EINVAL;
	}

	struct usb_descriptor_header **d;
	d = iphone_descs[config];
	int i = 0;
	for (; d && *d; d++)
	{
		pr_debug("  desc: type=0x%02x len=%u", (*d)->bDescriptorType, (*d)->bLength);
		if (i++ > 30)
			break;
	}

	int intfs = iphone_intf_counts[config];
	pr_debug("iPhone: adding %u intfs at config %u", intfs, config);

	for (int i = 0; i < intfs; i++)
	{
		int id = usb_interface_id(c, f);
		if (id < 0)
		{
			return id;
		}
	}

	ret = usb_assign_descriptors(f,
								  iphone_descs[config],
								  iphone_descs[config],
								  NULL, NULL);
	if (ret)
		return ret;

	if (config == 2) {
		struct iphone_hid *hid;

		if (iphone_gadget->hid) {
			usb_free_all_descriptors(f);
			return -EBUSY;
		}
		hid = iphone_hid_bind(f, &ep5, report_2_2_bin_1,
				      report_2_2_bin_1_len);
		if (IS_ERR(hid)) {
			usb_free_all_descriptors(f);
			return PTR_ERR(hid);
		}
		iphone_gadget->hid = hid;
	}
	return 0;
}

static void f_iphone_unbind(struct usb_configuration *c, struct usb_function *f)
{
	struct g_iphone *g = g_iphone_from_func(f);

	pr_debug("iPhone: unbind()");
	if (c->bConfigurationValue == 2 && g->hid) {
		iphone_hid_unbind(g->hid, -ENODEV);
		g->hid = NULL;
	}
	usb_free_all_descriptors(f);

	struct f_iphone *iphone_func = container_of(f, struct f_iphone, func);
	kfree(iphone_func);
}

static int f_iphone_set_alt(struct usb_function *f, unsigned intf, unsigned alt)
{
	int ret;
	pr_info("iPhone: set_alt(intf=%u, alt=%u)", intf, alt);

	struct g_iphone *iphone_gadget = g_iphone_from_func(f);
	if (!iphone_gadget)
		return -ENODEV;
	if (f->config->bConfigurationValue == 2 && intf == 2) {
		if (alt || !iphone_gadget->hid)
			return -EINVAL;
		ret = iphone_hid_enable(iphone_gadget->hid);
		if (ret)
			return ret;
	}

	if (!READ_ONCE(iphone_gadget->role_switch_requested))
		g_iphone_set_status(iphone_gadget, Enabled);
	return 0;
}

static void f_iphone_disable(struct usb_function *f)
{
	pr_debug("iPhone: disable()");

	struct g_iphone *iphone_gadget = g_iphone_from_func(f);
	if (!iphone_gadget)
		return;
	if (f->config->bConfigurationValue == 2)
		iphone_hid_disable(iphone_gadget->hid);

	if (!READ_ONCE(iphone_gadget->role_switch_requested))
		g_iphone_set_status(iphone_gadget, Disabled);
}

static void f_iphone_suspend(struct usb_function *f)
{
	pr_debug("iPhone: suspend()");

	struct g_iphone *iphone_gadget = g_iphone_from_func(f);
	if (!iphone_gadget)
		return;

	if (!READ_ONCE(iphone_gadget->role_switch_requested))
		g_iphone_set_status(iphone_gadget, Suspended);
}

static void f_iphone_resume(struct usb_function *f)
{
	pr_debug("iPhone: resume()");

	struct g_iphone *iphone_gadget = g_iphone_from_func(f);
	if (!iphone_gadget)
		return;

	if (!READ_ONCE(iphone_gadget->role_switch_requested))
		g_iphone_set_status(iphone_gadget, Enabled);
}

static int ep0_send(struct usb_composite_dev *cdev, const void *src, u16 wLength, size_t real_len)
{
	size_t len;
	if (!cdev || !cdev->req || !cdev->gadget || !cdev->gadget->ep0 || !cdev->req->buf)
		return -ENODEV;

	len = min_t(size_t, real_len, wLength);

	memcpy(cdev->req->buf, src, len);
	cdev->req->length = len;
	cdev->req->zero = len < wLength;
	pr_info("iPhone: ep0_send -> len %zu", len);

	return usb_ep_queue(cdev->gadget->ep0, cdev->req, GFP_ATOMIC);
}
static int ep0_zlp(struct usb_composite_dev *cdev)
{
	cdev->req->length = 0;
	cdev->req->zero = 0;
	return usb_ep_queue(cdev->gadget->ep0, cdev->req, GFP_ATOMIC);
}

/* ----------------- Control requests ----------------- */
static int f_iphone_setup(struct usb_function *f,
						const struct usb_ctrlrequest *ctrl)
{

	u16 wIndex = le16_to_cpu(ctrl->wIndex);
	u16 wValue = le16_to_cpu(ctrl->wValue);
	u16 wLength = le16_to_cpu(ctrl->wLength);

	pr_info("iPhone: setup bmReq=0x%02x bReq=0x%02x wValue=0x%04x wIndex=0x%04x wLen=%u",
			ctrl->bRequestType, ctrl->bRequest, wValue, wIndex, wLength);

	struct usb_composite_dev *cdev = f->config->cdev;
	if (!cdev)
		return -ENODEV;

	struct g_iphone *iphone_gadget = g_iphone_from_func(f);
	if (!iphone_gadget)
		return -ENODEV;

	/* iAP2 over HID */
	if (f->config->bConfigurationValue == 2 && wIndex == 2)
		return iphone_gadget->hid ?
			iphone_hid_setup(iphone_gadget->hid, ctrl) : -ENODEV;

	/* Vendor-specific requests (device-to-host) */
	if ((ctrl->bRequestType & USB_TYPE_MASK) == USB_TYPE_VENDOR)
	{

		switch (ctrl->bRequest)
		{
		case 0x40: /* Power Capability */
		{
			if (!cdev->req || !cdev->gadget->ep0)
				return -ENODEV;

			pr_info("iPhone: Power Capability offered: %umA", wValue + 500);
			return ep0_zlp(cdev);
		}
		case 0x51: /* Role Switch */
			pr_info("iPhone: Role Switch requested");
			if (!cdev->req || !cdev->gadget->ep0)
				return -ENODEV;

			WRITE_ONCE(iphone_gadget->role_switch_requested, true);
			g_iphone_set_status(iphone_gadget, RoleSwitch);

			return ep0_zlp(cdev);

		case 0x53: /* Capabilities */
		{
			pr_info("iPhone: Capabilities requested");
			static const u8 caps[4] = {0x01, 0x00, 0x00, 0x00};
			return ep0_send(cdev, caps, wLength, sizeof(caps));
		}
		case IPHONE_REQ_USBOOT: /* Launch userspace OTA usboot flow */
		{
			int ret;

			if (!cdev->req || !cdev->gadget->ep0)
				return -ENODEV;

			ret = g_iphone_start_recovery(iphone_gadget);
			if (ret)
				pr_warn("iPhone: recovery request rejected: %d\n", ret);

			return ep0_zlp(cdev);
		}
		case IPHONE_REQ_GADGET: /* Launch userspace gadget flow */
		{
			int ret;

			if (!cdev->req || !cdev->gadget->ep0)
				return -ENODEV;

			ret = g_iphone_start_gadget(iphone_gadget);
			if (ret)
				pr_warn("iPhone: gadget request rejected: %d\n", ret);

			return ep0_zlp(cdev);
		}
		}
	}
	return -EOPNOTSUPP;
}

static bool f_iphone_req_match(struct usb_function *f,
							 const struct usb_ctrlrequest *ctrl,
							 bool config0)
{
	/* req_match is important to be able to capture this request before any interface(1-4) is selected
	   some headunits may select the interface before role switch, some may not */

	if ((ctrl->bRequestType & USB_TYPE_MASK) == USB_TYPE_VENDOR)
	{
		switch (ctrl->bRequest)
		{
		case 0x40: /* Power Capability */
		case 0x51: /* Role Switch */
		case 0x53: /* Capabilities */
		case IPHONE_REQ_USBOOT: /* Launch userspace OTA usboot flow */
		case IPHONE_REQ_GADGET: /* Launch userspace gadget flow */
			return true;
		}
	}
	return false;
}

int iphone_do_config(struct usb_configuration *c)
{
	struct g_iphone *g = g_iphone_from_usb_config(c);

	struct f_iphone *iphone = kzalloc(sizeof(*iphone), GFP_KERNEL);
	if (!iphone)
		return -ENOMEM;

	iphone->config = c->bConfigurationValue;
	iphone->g = g;

	struct usb_function *f = &iphone->func;
	f->name = "iphone";
	f->bind = f_iphone_bind;
	f->unbind = f_iphone_unbind;
	f->set_alt = f_iphone_set_alt;
	f->disable = f_iphone_disable;
	f->setup = f_iphone_setup;
	f->resume = f_iphone_resume;
	f->suspend = f_iphone_suspend;
	f->req_match = f_iphone_req_match;

	return usb_add_function(c, f);
}
