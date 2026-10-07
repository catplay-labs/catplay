// SPDX-License-Identifier: GPL-2.0
#include <linux/module.h>
#include <linux/miscdevice.h>
#include <linux/fs.h>
#include <linux/uaccess.h>
#include <linux/slab.h>
#include <linux/poll.h>
#include <linux/completion.h>
#include <linux/kref.h>
#include <linux/list.h>
#include <linux/mutex.h>
#include <linux/workqueue.h>
#include <linux/idr.h>
#include "iap2_transport.h"

struct iap2_rx_record {
    struct list_head node;
    size_t len;
    size_t off;
    u8 data[];
};

struct iap2_tx_slot {
    struct iap2_tx_req req;
    struct completion done;
    int status;
    bool in_use;
    bool blocking;
};

struct iap2_session {
    struct kref kref;
    struct miscdevice miscdev;
    char name[24];
    int id;
    struct device *parent;
    const struct iap2_transport_ops *ops;
    void *ctx;
    struct mutex io_mutex;
    struct mutex read_mutex;
    spinlock_t lock;
    int terminal_error;
    int tx_error;
    int rx_error;
    bool started;
    bool registered;
    bool link_added;
    struct work_struct close_work;
    wait_queue_head_t tx_wq;
    wait_queue_head_t rx_wq;
    struct iap2_tx_slot tx[IAP2_TX_INFLIGHT];
    struct list_head rx_records;
    size_t rx_bytes;
};

/* Keep session numbers distinct while old open files still reference them. */
static DEFINE_MUTEX(iap2_registry_mutex);
static DEFINE_IDA(iap2_ids);
static struct iap2_session *iap2_active_session;

static void iap2_session_release(struct kref *ref)
{
    struct iap2_session *s = container_of(ref, struct iap2_session, kref);
    struct iap2_rx_record *rec, *tmp;
    int i;

    list_for_each_entry_safe(rec, tmp, &s->rx_records, node) {
        list_del(&rec->node);
        kfree(rec);
    }
    for (i = 0; i < IAP2_TX_INFLIGHT; i++)
        kfree(s->tx[i].req.data);
    if (s->id >= 0)
        ida_free(&iap2_ids, s->id);
    put_device(s->parent);
    kfree(s);
}

static int iap2_terminal_error(struct iap2_session *s)
{
    return READ_ONCE(s->terminal_error);
}

static void iap2_closed(struct iap2_session *s, int reason)
{
    unsigned long flags;

    if (WARN_ON_ONCE(reason >= 0))
        reason = -ENODEV;
    spin_lock_irqsave(&s->lock, flags);
    if (!s->terminal_error) {
        WRITE_ONCE(s->terminal_error, reason);
        /* Queue under the lock: unregister must not miss a concurrent close. */
        schedule_work(&s->close_work);
    }
    spin_unlock_irqrestore(&s->lock, flags);
    wake_up_interruptible_all(&s->rx_wq);
    wake_up_interruptible_all(&s->tx_wq);
}

static void iap2_close_work(struct work_struct *work)
{
    struct iap2_session *s = container_of(work, struct iap2_session, close_work);

    mutex_lock(&iap2_registry_mutex);
    if (s->link_added) {
        sysfs_remove_link(&s->parent->kobj, "iap2_devnode");
        s->link_added = false;
    }
    if (s->registered) {
        misc_deregister(&s->miscdev);
        s->registered = false;
    }
    mutex_lock(&s->io_mutex);
    if (s->started) {
        s->ops->stop(s->ctx, iap2_terminal_error(s));
        s->started = false;
    }
    mutex_unlock(&s->io_mutex);
    if (iap2_active_session == s)
        iap2_active_session = NULL;
    mutex_unlock(&iap2_registry_mutex);
}

static int iap2_rx_data(struct iap2_session *s, const void *data, size_t len)
{
    struct iap2_rx_record *rec;
    unsigned long flags;
    int ret;

    if (len > IAP2_RX_BACKLOG_MAX)
        return -EMSGSIZE;
    if (!len)
        return iap2_terminal_error(s);
    rec = kmalloc(struct_size(rec, data, len), GFP_ATOMIC);
    if (!rec)
        return -ENOMEM;
    rec->len = len;
    rec->off = 0;
    memcpy(rec->data, data, len);

    spin_lock_irqsave(&s->lock, flags);
    ret = s->terminal_error;
    if (!ret && s->rx_bytes + len > IAP2_RX_BACKLOG_MAX)
        ret = -ENOSPC;
    if (!ret) {
        s->rx_bytes += len;
        s->rx_error = 0;
        list_add_tail(&rec->node, &s->rx_records);
    }
    spin_unlock_irqrestore(&s->lock, flags);
    if (ret)
        kfree(rec);
    else
        wake_up_interruptible(&s->rx_wq);
    return ret;
}

static void iap2_rx_error(struct iap2_session *s, int error)
{
    unsigned long flags;

    spin_lock_irqsave(&s->lock, flags);
    if (!s->terminal_error)
        s->rx_error = error;
    spin_unlock_irqrestore(&s->lock, flags);
    wake_up_interruptible(&s->rx_wq);
}

static void iap2_tx_done(struct iap2_session *s, struct iap2_tx_req *req, int status)
{
    struct iap2_tx_slot *slot = container_of(req, struct iap2_tx_slot, req);
    unsigned long flags;

    spin_lock_irqsave(&s->lock, flags);
    slot->status = status;
    if (status && !slot->blocking && !s->terminal_error)
        s->tx_error = status;
    complete(&slot->done);
    /* A blocking writer owns its slot until it consumes the completion. */
    if (!slot->blocking)
        slot->in_use = false;
    spin_unlock_irqrestore(&s->lock, flags);
    wake_up_interruptible(&s->tx_wq);
}

static const struct iap2_transport_events iap2_events = {
    .rx_data = iap2_rx_data,
    .rx_error = iap2_rx_error,
    .tx_done = iap2_tx_done,
    .closed = iap2_closed,
};

static int iap2_open(struct inode *inode, struct file *file)
{
    struct miscdevice *misc = file->private_data;
    struct iap2_session *s = container_of(misc, struct iap2_session, miscdev);
    int ret = iap2_terminal_error(s);

    /* misc core serializes open with deregistration; the registration reference
     * remains held until after deregistration and stop have finished.
     */
    if (ret)
        return ret;
    kref_get(&s->kref);
    file->private_data = s;
    return 0;
}

static int iap2_release_file(struct inode *inode, struct file *file)
{
    struct iap2_session *s = file->private_data;

    kref_put(&s->kref, iap2_session_release);
    return 0;
}

static bool iap2_tx_free_locked(struct iap2_session *s)
{
    int i;

    for (i = 0; i < IAP2_TX_INFLIGHT; i++)
        if (!s->tx[i].in_use)
            return true;
    return false;
}

static bool iap2_tx_ready(struct iap2_session *s)
{
    unsigned long flags;
    bool ready;

    spin_lock_irqsave(&s->lock, flags);
    ready = s->terminal_error || s->tx_error || iap2_tx_free_locked(s);
    spin_unlock_irqrestore(&s->lock, flags);
    return ready;
}

static bool iap2_rx_ready(struct iap2_session *s)
{
    unsigned long flags;
    bool ready;

    spin_lock_irqsave(&s->lock, flags);
    ready = s->terminal_error || s->rx_error || !list_empty(&s->rx_records);
    spin_unlock_irqrestore(&s->lock, flags);
    return ready;
}

static __poll_t iap2_poll(struct file *file, poll_table *wait)
{
    struct iap2_session *s = file->private_data;
    unsigned long flags;
    __poll_t mask = 0;

    poll_wait(file, &s->tx_wq, wait);
    poll_wait(file, &s->rx_wq, wait);
    spin_lock_irqsave(&s->lock, flags);
    if (s->terminal_error) {
        mask = EPOLLERR | EPOLLHUP;
    } else {
        if (iap2_tx_free_locked(s))
            mask |= EPOLLOUT | EPOLLWRNORM;
        if (!list_empty(&s->rx_records))
            mask |= EPOLLIN | EPOLLRDNORM;
        if (s->rx_error || s->tx_error)
            mask |= EPOLLERR;
    }
    spin_unlock_irqrestore(&s->lock, flags);
    return mask;
}

static void iap2_release_slot(struct iap2_session *s, struct iap2_tx_slot *slot)
{
    unsigned long flags;

    spin_lock_irqsave(&s->lock, flags);
    slot->in_use = false;
    spin_unlock_irqrestore(&s->lock, flags);
    wake_up_interruptible(&s->tx_wq);
}

static ssize_t iap2_write(struct file *file, const char __user *buf,
                          size_t count, loff_t *ppos)
{
    struct iap2_session *s = file->private_data;
    struct iap2_tx_slot *slot = NULL;
    bool blocking = !(file->f_flags & O_NONBLOCK);
    unsigned long flags;
    int ret, i;

    ret = iap2_terminal_error(s);
    if (ret)
        return ret;
    if (!count)
        return 0;
    if (count > IAP2_MAX_XFER)
        return -EMSGSIZE;

    for (;;) {
        spin_lock_irqsave(&s->lock, flags);
        ret = s->terminal_error ?: s->tx_error;
        if (!s->terminal_error)
            s->tx_error = 0;
        if (!ret) {
            for (i = 0; i < IAP2_TX_INFLIGHT; i++) {
                if (!s->tx[i].in_use) {
                    slot = &s->tx[i];
                    slot->in_use = true;
                    slot->blocking = blocking;
                    slot->status = -EINPROGRESS;
                    reinit_completion(&slot->done);
                    break;
                }
            }
        }
        spin_unlock_irqrestore(&s->lock, flags);
        if (ret)
            return ret;
        if (slot)
            break;
        if (!blocking)
            return -EAGAIN;
        ret = wait_event_interruptible(s->tx_wq, iap2_tx_ready(s));
        if (ret)
            return iap2_terminal_error(s) ?: ret;
    }

    if (copy_from_user((void *)slot->req.data, buf, count)) {
        ret = -EFAULT;
        goto release;
    }
    slot->req.len = count;
    ret = mutex_lock_interruptible(&s->io_mutex);
    if (ret)
        goto release;
    ret = iap2_terminal_error(s);
    if (!ret && s->started)
        ret = s->ops->submit_tx(s->ctx, &slot->req);
    mutex_unlock(&s->io_mutex);
    if (ret)
        goto release;
    if (!blocking)
        return count;

    ret = wait_event_interruptible(s->tx_wq,
                iap2_terminal_error(s) || completion_done(&slot->done));
    if (ret || iap2_terminal_error(s)) {
        /* stop and cancellation share the submission gate. Do not release a
         * buffer while a backend may still be accessing it.
         */
        mutex_lock(&s->io_mutex);
        if (s->started)
            s->ops->cancel_tx(s->ctx, &slot->req);
        mutex_unlock(&s->io_mutex);
    } else {
        ret = slot->status;
    }
release:
    ret = iap2_terminal_error(s) ?: ret;
    iap2_release_slot(s, slot);
    return ret ?: count;
}

static ssize_t iap2_read(struct file *file, char __user *buf,
                         size_t count, loff_t *ppos)
{
    struct iap2_session *s = file->private_data;
    struct iap2_rx_record *rec;
    unsigned long flags;
    size_t len;
    ssize_t ret;
    bool freed = false;

    ret = iap2_terminal_error(s);
    if (ret || !count)
        return ret;
    for (;;) {
        /* Serialize access across copy_to_user, but never hold the reader
         * mutex while waiting for data from the transport.
         */
        if (file->f_flags & O_NONBLOCK) {
            if (!mutex_trylock(&s->read_mutex))
                return iap2_terminal_error(s) ?: -EAGAIN;
        } else {
            ret = mutex_lock_interruptible(&s->read_mutex);
            if (ret)
                return iap2_terminal_error(s) ?: ret;
        }
        spin_lock_irqsave(&s->lock, flags);
        ret = s->terminal_error;
        if (!ret && !list_empty(&s->rx_records))
            break;
        if (!ret)
            ret = s->rx_error;
        spin_unlock_irqrestore(&s->lock, flags);
        mutex_unlock(&s->read_mutex);
        if (ret)
            return ret;
        if (file->f_flags & O_NONBLOCK)
            return -EAGAIN;
        ret = wait_event_interruptible(s->rx_wq, iap2_rx_ready(s));
        if (ret)
            return iap2_terminal_error(s) ?: ret;
    }
    rec = list_first_entry(&s->rx_records, struct iap2_rx_record, node);
    len = min(count, rec->len - rec->off);
    spin_unlock_irqrestore(&s->lock, flags);
    ret = -EFAULT;
    if (!copy_to_user(buf, rec->data + rec->off, len)) {
        spin_lock_irqsave(&s->lock, flags);
        rec->off += len;
        if (rec->off == rec->len) {
            list_del(&rec->node);
            s->rx_bytes -= rec->len;
            freed = true;
        }
        spin_unlock_irqrestore(&s->lock, flags);
        ret = len;
    }
    if (freed)
        kfree(rec);
    mutex_unlock(&s->read_mutex);
    if (freed) {
        mutex_lock(&s->io_mutex);
        if (s->started && !iap2_terminal_error(s))
            s->ops->kick_rx(s->ctx);
        mutex_unlock(&s->io_mutex);
    }
    return iap2_terminal_error(s) ?: ret;
}

static const struct file_operations iap2_fops = {
    .owner = THIS_MODULE,
    .open = iap2_open,
    .release = iap2_release_file,
    .read = iap2_read,
    .write = iap2_write,
    .poll = iap2_poll,
    .llseek = noop_llseek,
};

struct iap2_session *iap2_session_register(struct device *parent,
                            const struct iap2_transport_ops *ops, void *ctx)
{
    struct iap2_session *s;
    int ret, i;

    if (!parent || !ops || !ops->start || !ops->stop || !ops->submit_tx ||
        !ops->cancel_tx || !ops->kick_rx)
        return ERR_PTR(-EINVAL);
    s = kzalloc(sizeof(*s), GFP_KERNEL);
    if (!s)
        return ERR_PTR(-ENOMEM);
    kref_init(&s->kref);
    s->id = -1;
    s->parent = get_device(parent);
    s->ops = ops;
    s->ctx = ctx;
    mutex_init(&s->io_mutex);
    mutex_init(&s->read_mutex);
    spin_lock_init(&s->lock);
    INIT_WORK(&s->close_work, iap2_close_work);
    INIT_LIST_HEAD(&s->rx_records);
    init_waitqueue_head(&s->rx_wq);
    init_waitqueue_head(&s->tx_wq);
    for (i = 0; i < IAP2_TX_INFLIGHT; i++) {
        init_completion(&s->tx[i].done);
        s->tx[i].req.index = i;
        s->tx[i].req.data = kmalloc(IAP2_MAX_XFER, GFP_KERNEL);
        if (!s->tx[i].req.data) {
            ret = -ENOMEM;
            goto put;
        }
    }
    s->miscdev.minor = MISC_DYNAMIC_MINOR;
    s->miscdev.name = s->name;
    s->miscdev.fops = &iap2_fops;
    s->miscdev.parent = parent;
    s->miscdev.mode = 0600;

    mutex_lock(&iap2_registry_mutex);
    if (iap2_active_session) {
        mutex_unlock(&iap2_registry_mutex);
        ret = -EBUSY;
        goto put;
    }
    ret = ida_alloc(&iap2_ids, GFP_KERNEL);
    if (ret < 0) {
        mutex_unlock(&iap2_registry_mutex);
        goto put;
    }
    s->id = ret;
    snprintf(s->name, sizeof(s->name), "iap2-%d", s->id);
    iap2_active_session = s;
    mutex_lock(&s->io_mutex);
    s->started = true;
    ret = ops->start(ctx, &iap2_events, s);
    mutex_unlock(&s->io_mutex);
    if (ret)
        goto unlock_close;
    ret = iap2_terminal_error(s);
    if (ret)
        goto unlock_close;
    ret = misc_register(&s->miscdev);
    if (ret)
        goto unlock_close;
    s->registered = true;
    ret = sysfs_create_link(&parent->kobj, &s->miscdev.this_device->kobj,
                            "iap2_devnode");
    if (ret)
        goto unlock_close;
    s->link_added = true;
    mutex_unlock(&iap2_registry_mutex);
    return s;

unlock_close:
    mutex_unlock(&iap2_registry_mutex);
    iap2_session_unregister(s, ret);
    return ERR_PTR(ret);
put:
    kref_put(&s->kref, iap2_session_release);
    return ERR_PTR(ret);
}
EXPORT_SYMBOL_GPL(iap2_session_register);

void iap2_session_unregister(struct iap2_session *s, int reason)
{
    iap2_closed(s, reason);
    flush_work(&s->close_work);
    kref_put(&s->kref, iap2_session_release);
}
EXPORT_SYMBOL_GPL(iap2_session_unregister);

int iap2_session_devnode_path(struct iap2_session *s, char *buf, size_t size)
{
    int ret = iap2_terminal_error(s);

    if (ret)
        return ret;
    return snprintf(buf, size, "/dev/%s", s->name) >= size ? -ENAMETOOLONG : 0;
}
EXPORT_SYMBOL_GPL(iap2_session_devnode_path);

int iap2_session_link_active(struct device *owner)
{
    struct iap2_session *s;
    int ret;

    mutex_lock(&iap2_registry_mutex);
    s = iap2_active_session;
    if (!s || !s->registered || iap2_terminal_error(s))
        ret = -ENODEV;
    else
        ret = sysfs_create_link(&owner->kobj, &s->miscdev.this_device->kobj,
                                "iap2_devnode");
    mutex_unlock(&iap2_registry_mutex);
    return ret;
}
EXPORT_SYMBOL_GPL(iap2_session_link_active);

MODULE_AUTHOR("CatPlay");
MODULE_DESCRIPTION("iAP2 character device core");
MODULE_LICENSE("GPL");
