// SPDX-License-Identifier: GPL-2.0
/* g_iphone's UDC adapter; HID report framing lives in iap2_hid. */
#include "libcomposite/composite.h"
#include "iphone_hid.h"
#include "hid.h"
#include "../iap2_hid/iap2_hid.h"
#include "../iap2_hid/hid_framing.h"
#include <linux/slab.h>
#include <linux/spinlock.h>
#include <linux/wait.h>
#include <linux/ctype.h>
#include <linux/hid.h>
#include <linux/printk.h>

#define IPHONE_HID_VERBOSE 0

struct iphone_hid {
    struct usb_function *func;
    struct usb_ep *in;
    struct usb_request *in_req, *ctrl_req;
    struct iap2_hid *backend;
    struct iap2_hid_report *active;
    struct hid_reports reports;
    const u8 *report_desc;
    size_t report_len;
    u8 *last[256];
    spinlock_t lock;
    wait_queue_head_t idle;
    unsigned int callbacks;
    bool enabled, stopped, ctrl_pending;
    u8 output_id, idle_rate;
};

static void in_complete(struct usb_ep *ep, struct usb_request *req)
{
    struct iphone_hid *hid = req->context;
    struct iap2_hid_report *report;
    unsigned long flags;
    int status = req->status;

    spin_lock_irqsave(&hid->lock, flags);
    hid->callbacks++;
    report = hid->active;
    hid->active = NULL;
    spin_unlock_irqrestore(&hid->lock, flags);
    if (!status && req->actual != req->length) status = -EIO;
#if IPHONE_HID_VERBOSE
    if (!status)
        print_hex_dump(KERN_WARNING, "iPhone HID IN: ", DUMP_PREFIX_OFFSET,
                       16, 1, req->buf, req->actual, true);
    else
        pr_warn("iPhone HID IN failed: status=%d actual=%u expected=%u\n",
                status, req->actual, req->length);
#endif
    if (report) iap2_hid_report_complete(report, status);
    spin_lock_irqsave(&hid->lock, flags);
    hid->callbacks--;
    wake_up_all(&hid->idle);
    spin_unlock_irqrestore(&hid->lock, flags);
}

static int submit_report(void *ctx, struct iap2_hid_report *report)
{
    struct iphone_hid *hid = ctx;
    unsigned long flags;
    int ret;

    spin_lock_irqsave(&hid->lock, flags);
    if (hid->stopped || !hid->enabled || hid->active) {
        ret = hid->stopped ? -ENODEV : -EAGAIN;
        spin_unlock_irqrestore(&hid->lock, flags);
        return ret;
    }
    if (report->len != hid->reports.input[report->report_id]) {
        spin_unlock_irqrestore(&hid->lock, flags);
        return -EINVAL;
    }
    memcpy(hid->last[report->report_id], report->data, report->len);
    memcpy(hid->in_req->buf, report->data, report->len);
    hid->in_req->length = report->len;
    hid->in_req->zero = 0;
    hid->active = report;
    ret = usb_ep_queue(hid->in, hid->in_req, GFP_ATOMIC);
    if (ret) hid->active = NULL;
    spin_unlock_irqrestore(&hid->lock, flags);
    return ret;
}

static bool output_idle(struct iphone_hid *hid)
{
    unsigned long flags;
    bool idle;
    spin_lock_irqsave(&hid->lock, flags);
    idle = !hid->active && !hid->callbacks;
    spin_unlock_irqrestore(&hid->lock, flags);
    return idle;
}

static bool report_idle(struct iphone_hid *hid,
                        struct iap2_hid_report *report)
{
    unsigned long flags;
    bool idle;

    spin_lock_irqsave(&hid->lock, flags);
    /* active is cleared before the completion callback finishes. */
    idle = hid->active != report && !hid->callbacks;
    spin_unlock_irqrestore(&hid->lock, flags);
    return idle;
}

static void cancel_report(void *ctx, struct iap2_hid_report *report)
{
    struct iphone_hid *hid = ctx;
    unsigned long flags;
    bool pending;
    spin_lock_irqsave(&hid->lock, flags);
    pending = hid->active == report;
    spin_unlock_irqrestore(&hid->lock, flags);
    if (pending) usb_ep_dequeue(hid->in, hid->in_req);
    wait_event(hid->idle, report_idle(hid, report));
}

static void disable_endpoint(struct iphone_hid *hid)
{
    unsigned long flags;
    bool enabled;
    spin_lock_irqsave(&hid->lock, flags);
    enabled = hid->enabled;
    hid->enabled = false;
    spin_unlock_irqrestore(&hid->lock, flags);
    if (enabled) usb_ep_disable(hid->in);
}

static void stop(void *ctx, int reason)
{
    struct iphone_hid *hid = ctx;
    unsigned long flags;
    bool pending;
    spin_lock_irqsave(&hid->lock, flags);
    hid->stopped = true;
    pending = hid->ctrl_pending;
    spin_unlock_irqrestore(&hid->lock, flags);
    disable_endpoint(hid);
    if (pending) usb_ep_dequeue(hid->func->config->cdev->gadget->ep0, hid->ctrl_req);
    wait_event(hid->idle, output_idle(hid) && !READ_ONCE(hid->ctrl_pending));
}

static const struct iap2_hid_io_ops io_ops = {
    .submit_report = submit_report, .cancel_report = cancel_report, .stop = stop,
};

static void control_complete(struct usb_ep *ep, struct usb_request *req)
{
    struct iphone_hid *hid = req->context;
    unsigned long flags;
    bool receive;
    int ret = 0;

    spin_lock_irqsave(&hid->lock, flags);
    hid->callbacks++;
    receive = !hid->stopped && hid->output_id;
    spin_unlock_irqrestore(&hid->lock, flags);
    if (receive && !req->status) {
#if IPHONE_HID_VERBOSE
        print_hex_dump(KERN_WARNING, "iPhone HID OUT: ", DUMP_PREFIX_OFFSET,
                       16, 1, req->buf, req->actual, true);
#endif
        if (req->actual != req->length) ret = -EPROTO;
        else ret = iap2_hid_set_report(hid->backend, IAP2_HID_OUTPUT_REPORT,
                                      hid->output_id, req->buf, req->actual);
#if IPHONE_HID_VERBOSE
        if (ret)
            pr_warn("iPhone HID OUT rejected: status=%d actual=%u expected=%u\n",
                    ret, req->actual, req->length);
#endif
        if (ret) {
            usb_ep_set_halt(ep);
            /* A dropped acknowledged fragment would corrupt the byte stream. */
            iap2_hid_closed(hid->backend, ret);
        }
    }
#if IPHONE_HID_VERBOSE
    else if (receive && req->status)
        pr_warn("iPhone HID OUT failed: status=%d actual=%u expected=%u\n",
                req->status, req->actual, req->length);
#endif
    spin_lock_irqsave(&hid->lock, flags);
    hid->ctrl_pending = false;
    hid->callbacks--;
    wake_up_all(&hid->idle);
    spin_unlock_irqrestore(&hid->lock, flags);
}

int iphone_hid_setup(struct iphone_hid *hid, const struct usb_ctrlrequest *ctrl)
{
    struct usb_request *req = hid->ctrl_req;
    unsigned long flags;
    unsigned int value = le16_to_cpu(ctrl->wValue), len = le16_to_cpu(ctrl->wLength);
    unsigned int id = value & 255;
    int ret = -EOPNOTSUPP;

    spin_lock_irqsave(&hid->lock, flags);
    if (hid->stopped || hid->ctrl_pending) goto out;
    hid->output_id = 0;
    switch (ctrl->bRequestType) {
    case USB_DIR_OUT | USB_TYPE_CLASS | USB_RECIP_INTERFACE:
        if (!hid->enabled) goto out;
        switch (ctrl->bRequest) {
        case HID_REQ_SET_REPORT:
            if ((value >> 8) != 2 || !hid->reports.output[id] ||
                len != hid->reports.output[id])
                goto out;
            hid->output_id = id;
            req->length = len;
            break;
        case HID_REQ_SET_IDLE:
            if (len || (value >> 8) ||
                (id && !hid->reports.input[id]))
                goto out;
            hid->idle_rate = 0;
            req->length = 0;
            break;
        default:
            goto out;
        }
        break;
    case USB_DIR_IN | USB_TYPE_CLASS | USB_RECIP_INTERFACE:
        if (!hid->enabled) goto out;
        switch (ctrl->bRequest) {
        case HID_REQ_GET_REPORT:
            if ((value >> 8) != 1 || !hid->reports.input[id])
                goto out;
            req->length = min_t(unsigned int, len, hid->reports.input[id]);
            memcpy(req->buf, hid->last[id], req->length);
            break;
        case HID_REQ_GET_IDLE:
            if (len != 1)
                goto out;
            *(u8 *)req->buf = hid->idle_rate;
            req->length = 1;
            break;
        default:
            goto out;
        }
        break;
    case USB_DIR_IN | USB_TYPE_STANDARD | USB_RECIP_INTERFACE:
        if (ctrl->bRequest != USB_REQ_GET_DESCRIPTOR) goto out;
        switch (value >> 8) {
        case HID_DT_HID:
            req->length = min_t(unsigned int, len, sizeof(hid_2_2_bin));
            memcpy(req->buf, hid_2_2_bin, req->length);
            break;
        case HID_DT_REPORT:
            req->length = min_t(size_t, len, hid->report_len);
            memcpy(req->buf, hid->report_desc, req->length);
            break;
        default:
            goto out;
        }
        break;
    default:
        goto out;
    }
    req->zero = req->length < len;
    hid->ctrl_pending = true;
    ret = usb_ep_queue(hid->func->config->cdev->gadget->ep0, req, GFP_ATOMIC);
    if (ret) hid->ctrl_pending = false;
out:
    spin_unlock_irqrestore(&hid->lock, flags);
    return ret;
}

int iphone_hid_enable(struct iphone_hid *hid)
{
    unsigned long flags;
    int ret;

    disable_endpoint(hid);
    spin_lock_irqsave(&hid->lock, flags);
    if (hid->stopped) {
        ret = -ENODEV;
        goto out;
    }
    ret = config_ep_by_speed(hid->func->config->cdev->gadget, hid->func, hid->in);
    if (ret) goto out;
    ret = usb_ep_enable(hid->in);
    if (!ret) hid->enabled = true;
out:
    spin_unlock_irqrestore(&hid->lock, flags);
    return ret;
}

void iphone_hid_close(struct iphone_hid *hid, int reason)
{
    if (hid && hid->backend) iap2_hid_closed(hid->backend, reason);
}

void iphone_hid_disable(struct iphone_hid *hid)
{
    if (!hid) return;
    disable_endpoint(hid);
    /* A configuration change can enable this function again. Physical unbind
     * and role switch supply the terminal session reason explicitly.
     */
}

void iphone_hid_unbind(struct iphone_hid *hid, int reason)
{
    int i;
    if (!hid) return;
    if (hid->backend)
        iap2_hid_destroy(hid->backend, reason);
    if (hid->in_req) {
        kfree(hid->in_req->buf);
        usb_ep_free_request(hid->in, hid->in_req);
    }
    if (hid->ctrl_req) {
        kfree(hid->ctrl_req->buf);
        usb_ep_free_request(hid->func->config->cdev->gadget->ep0, hid->ctrl_req);
    }
    for (i = 0; i < 256; i++) kfree(hid->last[i]);
    if (hid->in) usb_ep_autoconfig_release(hid->in);
    kfree(hid);
}

/* The descriptor layout mirrors an iPhone and is not negotiable. Fixed
 * hardware endpoints must have exactly the requested number; configurable
 * endpoints may adopt that number. Never rewrite the descriptor to fit a UDC.
 */
static struct usb_ep *claim_fixed_endpoint(struct usb_gadget *gadget,
                                          const struct usb_endpoint_descriptor *desc)
{
    struct usb_ep *ep;
    list_for_each_entry(ep, &gadget->ep_list, ep_list) {
        struct usb_endpoint_descriptor candidate = *desc;
        if (strlen(ep->name) > 2 && isdigit(ep->name[2]) &&
            simple_strtoul(ep->name + 2, NULL, 10) != usb_endpoint_num(desc))
            continue;
        if (!usb_gadget_ep_match_desc(gadget, ep, &candidate, NULL))
            continue;
        ep->address = desc->bEndpointAddress;
        ep->desc = NULL;
        ep->comp_desc = NULL;
        ep->claimed = true;
        return ep;
    }
    return NULL;
}

struct iphone_hid *iphone_hid_bind(struct usb_function *f, const struct usb_endpoint_descriptor *desc, const u8 *report_desc, size_t report_len)
{
    struct iphone_hid *hid = kzalloc(sizeof(*hid), GFP_KERNEL);
    struct iap2_hid_config config = {
        .report_descriptor = report_desc,
        .report_descriptor_len = report_len,
    };
    int i, ret;
    size_t max_len = 0;
    if (!hid) return ERR_PTR(-ENOMEM);
    hid->func = f;
    hid->report_desc = report_desc;
    hid->report_len = report_len;
    spin_lock_init(&hid->lock);
    init_waitqueue_head(&hid->idle);
    ret = hid_reports_parse(&hid->reports, config.report_descriptor, config.report_descriptor_len);
    if (ret) goto fail;
    hid->in = claim_fixed_endpoint(f->config->cdev->gadget, desc);
    if (!hid->in) { ret = -ENODEV; goto fail; }
    for (i = 1; i < 256; i++) {
        size_t len = hid->reports.input[i];
        max_len = max(max_len, max_t(size_t, len, hid->reports.output[i]));
        if (!len) continue;
        hid->last[i] = kzalloc(len, GFP_KERNEL);
        if (!hid->last[i]) { ret = -ENOMEM; goto fail; }
        hid->last[i][0] = i;
    }
    hid->in_req = usb_ep_alloc_request(hid->in, GFP_KERNEL);
    hid->ctrl_req = usb_ep_alloc_request(f->config->cdev->gadget->ep0, GFP_KERNEL);
    if (!hid->in_req || !hid->ctrl_req) { ret = -ENOMEM; goto fail; }
    hid->in_req->buf = kmalloc(max_len, GFP_KERNEL);
    hid->ctrl_req->buf = kmalloc(max(max_len, report_len), GFP_KERNEL);
    if (!hid->in_req->buf || !hid->ctrl_req->buf) { ret = -ENOMEM; goto fail; }
    hid->in_req->context = hid;
    hid->in_req->complete = in_complete;
    hid->ctrl_req->context = hid;
    hid->ctrl_req->complete = control_complete;
    hid->backend = iap2_hid_create(&f->config->cdev->gadget->dev, &config, &io_ops, hid);
    if (IS_ERR(hid->backend)) { ret = PTR_ERR(hid->backend); hid->backend = NULL; goto fail; }
    return hid;
fail:
    iphone_hid_unbind(hid, -ENODEV);
    return ERR_PTR(ret);
}
