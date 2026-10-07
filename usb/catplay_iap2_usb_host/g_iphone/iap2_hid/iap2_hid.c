// SPDX-License-Identifier: GPL-2.0
#include <linux/module.h>
#include <linux/slab.h>
#include <linux/spinlock.h>
#include <linux/list.h>
#include <linux/workqueue.h>
#include "../iap2_char/iap2_transport.h"
#include "iap2_hid.h"
#include "hid_framing.h"
#include <linux/completion.h>
#include <linux/mutex.h>

#define IAP2_HID_RX_RECORDS 64
#define IAP2_HID_RX_RETRY_MS 100

struct iap2_hid_tx {
    struct iap2_hid_report report;
    struct iap2_hid *hid;
    struct iap2_tx_req *req;
    struct work_struct work;
    struct completion done;
    u8 *wire;
    int status;
    bool cancelled;
};

struct iap2_hid_rx {
    struct list_head node;
    size_t len;
    u8 *data;
};

struct iap2_hid {
    struct iap2_session *session;
    const struct iap2_transport_events *events;
    const struct iap2_hid_io_ops *io;
    void *io_ctx;
    size_t max_input_report_len;
    size_t max_output_report_len;
    spinlock_t lock;
    int terminal_error;
    struct iap2_hid_tx tx[IAP2_TX_INFLIGHT];
    struct list_head rx_queue;
    size_t rx_bytes;
    unsigned int rx_records;
    struct delayed_work rx_work;
    struct hid_reports reports;
    bool rx_continuing;
    struct workqueue_struct *tx_wq;
    struct mutex out_mutex;
};

static void iap2_hid_free_rx(struct iap2_hid_rx *rx)
{
    kfree(rx->data);
    kfree(rx);
}

static void iap2_hid_schedule_rx(struct iap2_hid *hid, unsigned long delay)
{
    unsigned long flags;

    spin_lock_irqsave(&hid->lock, flags);
    if (!hid->terminal_error)
        schedule_delayed_work(&hid->rx_work, delay);
    spin_unlock_irqrestore(&hid->lock, flags);
}

static int iap2_hid_receive_report(struct iap2_hid *hid, struct iap2_hid_rx *rx)
{
    struct hid_fragment fragment;
    bool continuing = hid->rx_continuing;
    int ret;

    ret = hid_report_decode(hid->reports.output, &continuing,
                            rx->data, rx->len, &fragment);
    if (ret)
        return ret;
    /* Include HID padding. Only userspace knows where the iAP2 packet ends. */
    ret = hid->events->rx_data(hid->session, fragment.data, fragment.len);
    if (!ret)
        hid->rx_continuing = continuing;
    return ret;
}

static void iap2_hid_rx_work(struct work_struct *work)
{
    struct iap2_hid *hid = container_of(to_delayed_work(work), struct iap2_hid, rx_work);
    struct iap2_hid_rx *rx;
    unsigned long flags;
    int ret;

    for (;;) {
        spin_lock_irqsave(&hid->lock, flags);
        if (hid->terminal_error || list_empty(&hid->rx_queue)) {
            spin_unlock_irqrestore(&hid->lock, flags);
            return;
        }
        rx = list_first_entry(&hid->rx_queue, struct iap2_hid_rx, node);
        spin_unlock_irqrestore(&hid->lock, flags);

        ret = iap2_hid_receive_report(hid, rx);
        if (ret == -ENOSPC)
            return;
        if (ret == -ENOMEM) {
            iap2_hid_schedule_rx(hid, msecs_to_jiffies(IAP2_HID_RX_RETRY_MS));
            return;
        }
        if (ret) {
            iap2_hid_closed(hid, ret);
            return;
        }
        spin_lock_irqsave(&hid->lock, flags);
        list_del(&rx->node);
        hid->rx_bytes -= rx->len;
        hid->rx_records--;
        spin_unlock_irqrestore(&hid->lock, flags);
        iap2_hid_free_rx(rx);
    }
}

int iap2_hid_set_report(struct iap2_hid *hid, u8 report_type, u8 report_id,
                        const void *data, size_t len)
{
    struct iap2_hid_rx *rx;
    unsigned long flags;
    int ret;

    spin_lock_irqsave(&hid->lock, flags);
    ret = hid->terminal_error;
    if (!ret && report_type != IAP2_HID_OUTPUT_REPORT)
        ret = -EOPNOTSUPP;
    if (!ret && len > hid->max_output_report_len)
        ret = -EMSGSIZE;
    if (!ret && (!len || !data ||
        ((const u8 *)data)[0] != report_id || !hid->reports.output[report_id] ||
        len != hid->reports.output[report_id]))
        ret = -EPROTO;
    if (!ret && len && !data)
        ret = -EINVAL;
    if (!ret && (hid->rx_records >= IAP2_HID_RX_RECORDS ||
                 hid->rx_bytes + len > IAP2_RX_BACKLOG_MAX))
        ret = -ENOSPC;
    if (!ret && len) {
        /* Reserve before allocating, including concurrent ingress calls. */
        hid->rx_records++;
        hid->rx_bytes += len;
    }
    spin_unlock_irqrestore(&hid->lock, flags);
    if (ret || !len)
        return ret;

    rx = kzalloc(sizeof(*rx), GFP_ATOMIC);
    if (rx)
        rx->data = kmemdup(data, len, GFP_ATOMIC);
    ret = rx && rx->data ? 0 : -ENOMEM;
    spin_lock_irqsave(&hid->lock, flags);
    ret = hid->terminal_error ?: ret;
    if (!ret) {
        rx->len = len;
        list_add_tail(&rx->node, &hid->rx_queue);
        schedule_delayed_work(&hid->rx_work, 0);
    } else {
        hid->rx_records--;
        hid->rx_bytes -= len;
    }
    spin_unlock_irqrestore(&hid->lock, flags);
    if (ret && rx)
        iap2_hid_free_rx(rx);
    return ret;
}
EXPORT_SYMBOL_GPL(iap2_hid_set_report);

/* Caller holds hid->lock. Core callbacks never call transport ops inline. */
static void iap2_hid_finish_tx(struct iap2_hid_tx *tx, int status)
{
    struct iap2_tx_req *req = tx->req;

    if (!req)
        return;
    tx->req = NULL;
    tx->hid->events->tx_done(tx->hid->session, req, status);
}

void iap2_hid_report_complete(struct iap2_hid_report *report, int status)
{
    struct iap2_hid_tx *tx = container_of(report, struct iap2_hid_tx, report);
    struct iap2_hid *hid = tx->hid;
    unsigned long flags;

    spin_lock_irqsave(&hid->lock, flags);
    tx->status = status;
    complete(&tx->done);
    spin_unlock_irqrestore(&hid->lock, flags);
}
EXPORT_SYMBOL_GPL(iap2_hid_report_complete);

void iap2_hid_closed(struct iap2_hid *hid, int reason)
{
    unsigned long flags;

    if (WARN_ON_ONCE(reason >= 0))
        reason = -ENODEV;
    spin_lock_irqsave(&hid->lock, flags);
    if (!hid->terminal_error) {
        hid->terminal_error = reason;
        hid->events->closed(hid->session, reason);
    }
    spin_unlock_irqrestore(&hid->lock, flags);
}
EXPORT_SYMBOL_GPL(iap2_hid_closed);

static int iap2_hid_start(void *ctx, const struct iap2_transport_events *events,
                         struct iap2_session *session)
{
    struct iap2_hid *hid = ctx;

    hid->events = events;
    hid->session = session;
    return 0;
}

static void iap2_hid_tx_work(struct work_struct *work)
{
    struct iap2_hid_tx *tx = container_of(work, struct iap2_hid_tx, work);
    struct iap2_hid *hid = tx->hid;
    struct iap2_tx_req *req = tx->req;
    size_t offset = 0;
    unsigned long flags;
    int ret = 0;

    while (offset < req->len) {
        mutex_lock(&hid->out_mutex);
        if (READ_ONCE(hid->terminal_error) || tx->cancelled) {
            ret = READ_ONCE(hid->terminal_error) ?: -ECANCELED;
            mutex_unlock(&hid->out_mutex);
            break;
        }
        ret = hid_report_encode(hid->reports.input, req->data, req->len,
                                &offset, tx->wire, hid->max_input_report_len);
        if (ret < 0) { mutex_unlock(&hid->out_mutex); break; }
        tx->report.data = tx->wire;
        tx->report.len = ret;
        tx->report.report_id = tx->wire[0];
        reinit_completion(&tx->done);
        ret = hid->io->submit_report(hid->io_ctx, &tx->report);
        mutex_unlock(&hid->out_mutex);
        if (ret) break;
        wait_for_completion(&tx->done);
        ret = tx->status;
        if (ret) break;
    }
    spin_lock_irqsave(&hid->lock, flags);
    iap2_hid_finish_tx(tx, hid->terminal_error ?: ret);
    spin_unlock_irqrestore(&hid->lock, flags);
}

static int iap2_hid_submit_tx(void *ctx, struct iap2_tx_req *req)
{
    struct iap2_hid *hid = ctx;
    struct iap2_hid_tx *tx = &hid->tx[req->index];
    unsigned long flags;
    int ret;

    spin_lock_irqsave(&hid->lock, flags);
    ret = hid->terminal_error;
    if (!ret) {
        tx->req = req;
    }
    spin_unlock_irqrestore(&hid->lock, flags);
    if (ret)
        return ret;
    tx->cancelled = false;
    queue_work(hid->tx_wq, &tx->work);
    return 0;
}

static void iap2_hid_cancel_tx(void *ctx, struct iap2_tx_req *req)
{
    struct iap2_hid *hid = ctx;
    struct iap2_hid_tx *tx = &hid->tx[req->index];
    unsigned long flags;

    mutex_lock(&hid->out_mutex);
    tx->cancelled = true;
    hid->io->cancel_report(hid->io_ctx, &tx->report);
    tx->status = -ECANCELED;
    complete(&tx->done);
    mutex_unlock(&hid->out_mutex);
    cancel_work_sync(&tx->work);
    spin_lock_irqsave(&hid->lock, flags);
    iap2_hid_finish_tx(tx, -ECANCELED);
    spin_unlock_irqrestore(&hid->lock, flags);
}

static void iap2_hid_kick_rx(void *ctx)
{
    iap2_hid_schedule_rx(ctx, 0);
}

static void iap2_hid_stop(void *ctx, int reason)
{
    struct iap2_hid *hid = ctx;
    struct iap2_hid_rx *rx, *tmp;
    unsigned long flags;
    int i;

    spin_lock_irqsave(&hid->lock, flags);
    /* Core arbitrates races between an owner reset and external closure. */
    hid->terminal_error = reason;
    spin_unlock_irqrestore(&hid->lock, flags);
    /* The arbiter must join all external callers before destroy can free hid. */
    mutex_lock(&hid->out_mutex);
    hid->io->stop(hid->io_ctx, reason);
    for (i = 0; i < IAP2_TX_INFLIGHT; i++) {
        hid->tx[i].status = reason;
        complete(&hid->tx[i].done);
    }
    mutex_unlock(&hid->out_mutex);
    for (i = 0; i < IAP2_TX_INFLIGHT; i++)
        cancel_work_sync(&hid->tx[i].work);
    cancel_delayed_work_sync(&hid->rx_work);
    spin_lock_irqsave(&hid->lock, flags);
    for (i = 0; i < IAP2_TX_INFLIGHT; i++)
        iap2_hid_finish_tx(&hid->tx[i], reason);
    spin_unlock_irqrestore(&hid->lock, flags);
    list_for_each_entry_safe(rx, tmp, &hid->rx_queue, node) {
        list_del(&rx->node);
        iap2_hid_free_rx(rx);
    }
    hid->rx_bytes = 0;
    hid->rx_records = 0;
}

static const struct iap2_transport_ops iap2_hid_transport_ops = {
    .start = iap2_hid_start,
    .submit_tx = iap2_hid_submit_tx,
    .cancel_tx = iap2_hid_cancel_tx,
    .kick_rx = iap2_hid_kick_rx,
    .stop = iap2_hid_stop,
};

struct iap2_hid *iap2_hid_create(struct device *parent,
                                const struct iap2_hid_config *config,
                                const struct iap2_hid_io_ops *ops, void *ctx)
{
    struct iap2_session *session;
    struct iap2_hid *hid;
    int i, ret;

    if (!config || !config->report_descriptor || !config->report_descriptor_len ||
        !ops || !ops->submit_report || !ops->cancel_report || !ops->stop)
        return ERR_PTR(-EINVAL);
    hid = kzalloc(sizeof(*hid), GFP_KERNEL);
    if (!hid)
        return ERR_PTR(-ENOMEM);
    hid->io = ops;
    hid->io_ctx = ctx;
    mutex_init(&hid->out_mutex);
    ret = hid_reports_parse(&hid->reports, config->report_descriptor,
                            config->report_descriptor_len);
    if (ret) goto fail;
    for (i = 1; i < 256; i++) {
        hid->max_input_report_len = max_t(size_t, hid->max_input_report_len, hid->reports.input[i]);
        hid->max_output_report_len = max_t(size_t, hid->max_output_report_len, hid->reports.output[i]);
    }
    hid->tx_wq = alloc_ordered_workqueue("iap2_hid_tx", WQ_MEM_RECLAIM);
    if (!hid->tx_wq) { ret = -ENOMEM; goto fail; }
    spin_lock_init(&hid->lock);
    INIT_LIST_HEAD(&hid->rx_queue);
    INIT_DELAYED_WORK(&hid->rx_work, iap2_hid_rx_work);
    for (i = 0; i < IAP2_TX_INFLIGHT; i++) {
        hid->tx[i].hid = hid;
        init_completion(&hid->tx[i].done);
        INIT_WORK(&hid->tx[i].work, iap2_hid_tx_work);
        hid->tx[i].wire = kmalloc(hid->max_input_report_len, GFP_KERNEL);
        if (!hid->tx[i].wire) { ret = -ENOMEM; goto fail; }
    }
    session = iap2_session_register(parent, &iap2_hid_transport_ops, hid);
    if (IS_ERR(session)) {
        ret = PTR_ERR(session);
        goto fail;
    }
    return hid;
fail:
    if (hid->tx_wq) destroy_workqueue(hid->tx_wq);
    for (i = 0; i < IAP2_TX_INFLIGHT; i++) kfree(hid->tx[i].wire);
    kfree(hid);
    return ERR_PTR(ret);
}
EXPORT_SYMBOL_GPL(iap2_hid_create);

void iap2_hid_destroy(struct iap2_hid *hid, int reason)
{
    int i;
    iap2_session_unregister(hid->session, reason);
    if (hid->tx_wq) destroy_workqueue(hid->tx_wq);
    for (i = 0; i < IAP2_TX_INFLIGHT; i++) kfree(hid->tx[i].wire);
    kfree(hid);
}
EXPORT_SYMBOL_GPL(iap2_hid_destroy);

MODULE_AUTHOR("CatPlay");
MODULE_DESCRIPTION("iAP2 HID transport");
MODULE_LICENSE("GPL");
