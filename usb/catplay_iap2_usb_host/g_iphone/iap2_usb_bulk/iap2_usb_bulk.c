// SPDX-License-Identifier: GPL-2.0
#include <linux/module.h>
#include <linux/usb.h>
#include <linux/slab.h>
#include <linux/workqueue.h>
#include <linux/spinlock.h>
#include <linux/list.h>
#include "iap2_usb_bulk.h"
#include "../iap2_char/iap2_transport.h"

#define IAP2_RX_INFLIGHT 3
#define IAP2_RX_RETRY_MS 100

struct iap2_usb;

struct iap2_usb_tx {
    struct iap2_usb *usb;
    struct urb *urb;
    struct iap2_tx_req *req;
};

enum iap2_usb_rx_state {
    IAP2_RX_IDLE,
    IAP2_RX_ACTIVE,
    IAP2_RX_DONE,
};

struct iap2_usb_rx {
    struct iap2_usb *usb;
    struct urb *urb;
    u8 *buf;
    struct list_head node;
    enum iap2_usb_rx_state state;
    int status;
    size_t len;
};

struct iap2_usb {
    struct usb_device *udev;
    struct usb_interface *intf;
    u8 ep_in;
    u8 ep_out;
    struct iap2_session *session;
    const struct iap2_transport_events *events;
    spinlock_t rx_lock;
    bool stopping;
    struct list_head rx_done;
    struct delayed_work rx_work;
    struct work_struct tx_halt_work;
    struct iap2_usb_tx tx[IAP2_TX_INFLIGHT];
    struct iap2_usb_rx rx[IAP2_RX_INFLIGHT];
};

static void iap2_usb_schedule_rx(struct iap2_usb *usb, unsigned long delay)
{
    unsigned long flags;

    spin_lock_irqsave(&usb->rx_lock, flags);
    if (!usb->stopping)
        schedule_delayed_work(&usb->rx_work, delay);
    spin_unlock_irqrestore(&usb->rx_lock, flags);
}

static void iap2_usb_kick_rx(void *ctx)
{
    iap2_usb_schedule_rx(ctx, 0);
}

static void iap2_usb_rx_failed(struct iap2_usb *usb, int error)
{
    if (error == -ENODEV || error == -ESHUTDOWN) {
        usb->events->closed(usb->session, -ENODEV);
        return;
    }
    usb->events->rx_error(usb->session, error);
    iap2_usb_schedule_rx(usb, msecs_to_jiffies(IAP2_RX_RETRY_MS));
}

static void iap2_usb_schedule_tx_halt(struct iap2_usb *usb)
{
    unsigned long flags;

    spin_lock_irqsave(&usb->rx_lock, flags);
    if (!usb->stopping)
        schedule_work(&usb->tx_halt_work);
    spin_unlock_irqrestore(&usb->rx_lock, flags);
}

static void iap2_usb_tx_halt_work(struct work_struct *work)
{
    struct iap2_usb *usb = container_of(work, struct iap2_usb, tx_halt_work);
    int ret;

    if (READ_ONCE(usb->stopping))
        return;
    ret = usb_clear_halt(usb->udev, usb_sndbulkpipe(usb->udev, usb->ep_out));
    if (ret)
        dev_err(&usb->intf->dev, "failed to clear TX halt: %d\n", ret);
}

static void iap2_usb_tx_complete(struct urb *urb)
{
    struct iap2_usb_tx *tx = urb->context;
    struct iap2_usb *usb = tx->usb;
    struct iap2_tx_req *req = tx->req;
    int status = urb->status;

    if (!status && urb->actual_length != req->len)
        status = -EIO;
    if (status == -EPIPE)
        iap2_usb_schedule_tx_halt(usb);
    /* The core can reuse req immediately after this event. */
    usb->events->tx_done(usb->session, req, status);
}

static void iap2_usb_rx_complete(struct urb *urb)
{
    struct iap2_usb_rx *rx = urb->context;
    struct iap2_usb *usb = rx->usb;
    unsigned long flags;

    spin_lock_irqsave(&usb->rx_lock, flags);
    rx->status = urb->status;
    rx->len = urb->actual_length;
    rx->state = IAP2_RX_DONE;
    list_add_tail(&rx->node, &usb->rx_done);
    if (!usb->stopping)
        schedule_delayed_work(&usb->rx_work, 0);
    spin_unlock_irqrestore(&usb->rx_lock, flags);
}

static void iap2_usb_rx_work(struct work_struct *work)
{
    struct iap2_usb *usb = container_of(to_delayed_work(work), struct iap2_usb, rx_work);
    struct iap2_usb_rx *rx;
    unsigned long flags;
    int ret, i;

    /* Preserve completion order and retain full buffers while the core queue
     * is full. Never resubmit an URB whose payload has not been accepted.
     */
    for (;;) {
        spin_lock_irqsave(&usb->rx_lock, flags);
        if (usb->stopping || list_empty(&usb->rx_done)) {
            spin_unlock_irqrestore(&usb->rx_lock, flags);
            break;
        }
        rx = list_first_entry(&usb->rx_done, struct iap2_usb_rx, node);
        spin_unlock_irqrestore(&usb->rx_lock, flags);

        if (!rx->status && rx->len) {
            ret = usb->events->rx_data(usb->session, rx->buf, rx->len);
            if (ret) {
                if (ret != -ENOSPC)
                    iap2_usb_rx_failed(usb, ret);
                return;
            }
        } else if (rx->status == -EPIPE) {
            ret = usb_clear_halt(usb->udev,
                                 usb_rcvbulkpipe(usb->udev, usb->ep_in));
            if (ret) {
                iap2_usb_rx_failed(usb, ret);
                return;
            }
        } else if (rx->status) {
            iap2_usb_rx_failed(usb, rx->status);
        }
        spin_lock_irqsave(&usb->rx_lock, flags);
        list_del_init(&rx->node);
        rx->state = IAP2_RX_IDLE;
        spin_unlock_irqrestore(&usb->rx_lock, flags);
    }

    for (i = 0; i < IAP2_RX_INFLIGHT; i++) {
        rx = &usb->rx[i];
        spin_lock_irqsave(&usb->rx_lock, flags);
        if (usb->stopping) {
            spin_unlock_irqrestore(&usb->rx_lock, flags);
            return;
        }
        if (rx->state != IAP2_RX_IDLE) {
            spin_unlock_irqrestore(&usb->rx_lock, flags);
            continue;
        }
        rx->state = IAP2_RX_ACTIVE;
        spin_unlock_irqrestore(&usb->rx_lock, flags);

        ret = usb_submit_urb(rx->urb, GFP_KERNEL);
        if (ret) {
            spin_lock_irqsave(&usb->rx_lock, flags);
            rx->state = IAP2_RX_IDLE;
            spin_unlock_irqrestore(&usb->rx_lock, flags);
            if (ret == -EPIPE) {
                int clear_ret = usb_clear_halt(usb->udev,
                                    usb_rcvbulkpipe(usb->udev, usb->ep_in));

                if (clear_ret)
                    ret = clear_ret;
            }
            iap2_usb_rx_failed(usb, ret);
            return;
        }
    }
}

static int iap2_usb_start(void *ctx, const struct iap2_transport_events *events,
                           struct iap2_session *session)
{
    struct iap2_usb *usb = ctx;

    usb->events = events;
    usb->session = session;
    WRITE_ONCE(usb->stopping, false);
    iap2_usb_kick_rx(usb);
    return 0;
}

static int iap2_usb_submit_tx(void *ctx, struct iap2_tx_req *req)
{
    struct iap2_usb *usb = ctx;
    struct iap2_usb_tx *tx = &usb->tx[req->index];
    int ret;

    tx->req = req;
    usb_fill_bulk_urb(tx->urb, usb->udev,
                      usb_sndbulkpipe(usb->udev, usb->ep_out),
                      (void *)req->data, req->len, iap2_usb_tx_complete, tx);
    /* Preserve bulk framing: intentionally no URB_ZERO_PACKET. */
    tx->urb->transfer_flags = 0;
    ret = usb_submit_urb(tx->urb, GFP_KERNEL);
    if (ret == -EPIPE)
        iap2_usb_schedule_tx_halt(usb);
    return ret;
}

static void iap2_usb_cancel_tx(void *ctx, struct iap2_tx_req *req)
{
    struct iap2_usb *usb = ctx;

    usb_kill_urb(usb->tx[req->index].urb);
}

static void iap2_usb_stop(void *ctx, int reason)
{
    struct iap2_usb *usb = ctx;
    unsigned long flags;
    int i;

    spin_lock_irqsave(&usb->rx_lock, flags);
    WRITE_ONCE(usb->stopping, true);
    spin_unlock_irqrestore(&usb->rx_lock, flags);
    /* Drain submitters before killing URBs: no check/submit race with stop. */
    cancel_delayed_work_sync(&usb->rx_work);
    cancel_work_sync(&usb->tx_halt_work);
    for (i = 0; i < IAP2_TX_INFLIGHT; i++)
        usb_kill_urb(usb->tx[i].urb);
    for (i = 0; i < IAP2_RX_INFLIGHT; i++)
        usb_kill_urb(usb->rx[i].urb);
    dev_dbg(&usb->intf->dev, "session stopped: %d\n", reason);
}

static const struct iap2_transport_ops iap2_usb_ops = {
    .start = iap2_usb_start,
    .submit_tx = iap2_usb_submit_tx,
    .cancel_tx = iap2_usb_cancel_tx,
    .kick_rx = iap2_usb_kick_rx,
    .stop = iap2_usb_stop,
};

static void iap2_usb_free(struct iap2_usb *usb)
{
    int i;

    for (i = 0; i < IAP2_TX_INFLIGHT; i++)
        usb_free_urb(usb->tx[i].urb);
    for (i = 0; i < IAP2_RX_INFLIGHT; i++) {
        usb_free_urb(usb->rx[i].urb);
        kfree(usb->rx[i].buf);
    }
    usb_put_dev(usb->udev);
    kfree(usb);
}

/* Compatibility adapter for iap2_scan; USB private data stays in the backend. */
int iap2_usb_bulk_devnode_path(struct usb_interface *intf, char *buf, size_t size)
{
    struct iap2_usb *usb = usb_get_intfdata(intf);

    if (!usb)
        return -ENODEV;
    return iap2_session_devnode_path(usb->session, buf, size);
}
EXPORT_SYMBOL_GPL(iap2_usb_bulk_devnode_path);

static int iap2_usb_probe(struct usb_interface *intf, const struct usb_device_id *id)
{
    struct usb_host_interface *alts = intf->cur_altsetting;
    struct iap2_usb *usb;
    struct iap2_session *session;
    int ret, i;

    usb = kzalloc(sizeof(*usb), GFP_KERNEL);
    if (!usb)
        return -ENOMEM;
    usb->udev = usb_get_dev(interface_to_usbdev(intf));
    usb->intf = intf;
    usb->stopping = true;
    spin_lock_init(&usb->rx_lock);
    INIT_LIST_HEAD(&usb->rx_done);
    INIT_DELAYED_WORK(&usb->rx_work, iap2_usb_rx_work);
    INIT_WORK(&usb->tx_halt_work, iap2_usb_tx_halt_work);
    for (i = 0; i < alts->desc.bNumEndpoints; i++) {
        struct usb_endpoint_descriptor *ep = &alts->endpoint[i].desc;

        if (usb_endpoint_is_bulk_in(ep) && !usb->ep_in)
            usb->ep_in = ep->bEndpointAddress;
        else if (usb_endpoint_is_bulk_out(ep) && !usb->ep_out)
            usb->ep_out = ep->bEndpointAddress;
    }
    if (!usb->ep_in || !usb->ep_out) {
        ret = -ENODEV;
        goto free;
    }
    for (i = 0; i < IAP2_TX_INFLIGHT; i++) {
        usb->tx[i].usb = usb;
        usb->tx[i].urb = usb_alloc_urb(0, GFP_KERNEL);
        if (!usb->tx[i].urb) {
            ret = -ENOMEM;
            goto free;
        }
    }
    for (i = 0; i < IAP2_RX_INFLIGHT; i++) {
        struct iap2_usb_rx *rx = &usb->rx[i];

        rx->usb = usb;
        INIT_LIST_HEAD(&rx->node);
        rx->buf = kmalloc(IAP2_MAX_XFER, GFP_KERNEL);
        rx->urb = usb_alloc_urb(0, GFP_KERNEL);
        if (!rx->buf || !rx->urb) {
            ret = -ENOMEM;
            goto free;
        }
        usb_fill_bulk_urb(rx->urb, usb->udev,
                          usb_rcvbulkpipe(usb->udev, usb->ep_in),
                          rx->buf, IAP2_MAX_XFER, iap2_usb_rx_complete, rx);
    }
    session = iap2_session_register(&intf->dev, &iap2_usb_ops, usb);
    if (IS_ERR(session)) {
        ret = PTR_ERR(session);
        goto free;
    }
    usb_set_intfdata(intf, usb);
    dev_info(&intf->dev, "bound iAP2 bulk interface\n");
    return 0;
free:
    iap2_usb_free(usb);
    return ret;
}

static void iap2_usb_disconnect(struct usb_interface *intf)
{
    struct iap2_usb *usb = usb_get_intfdata(intf);

    if (!usb)
        return;
    usb_set_intfdata(intf, NULL);
    iap2_session_unregister(usb->session, -ENODEV);
    iap2_usb_free(usb);
    dev_info(&intf->dev, "disconnected\n");
}

static const struct usb_device_id iap2_id_table[] = {
    { USB_INTERFACE_INFO(0xFF, 0xF0, 0x00) },
    { }
};
MODULE_DEVICE_TABLE(usb, iap2_id_table);

static struct usb_driver iap2_usb_driver = {
    .name = "iap2_char",
    .probe = iap2_usb_probe,
    .disconnect = iap2_usb_disconnect,
    .id_table = iap2_id_table,
};
module_usb_driver(iap2_usb_driver);

MODULE_AUTHOR("CatPlay");
MODULE_DESCRIPTION("iAP2 USB bulk transport");
MODULE_LICENSE("GPL");
