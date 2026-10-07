#include <linux/completion.h>
#include <linux/device.h>
#include <linux/device/driver.h>
#include <linux/kernel.h>
#include <linux/kobject.h>
#include <linux/list.h>
#include <linux/mutex.h>
#include <linux/module.h>
#include <linux/slab.h>
#include <linux/spinlock.h>
#include <linux/sysfs.h>
#include <linux/usb.h>
#include <linux/usb/cdc.h>
#include <linux/usb/cdc_ncm.h>
#include <linux/usb/usbnet.h>

#include "iap2_scan.h"
#include "../iap2_usb_bulk/iap2_usb_bulk.h"

#define IAP2_DRIVER_NAME "iap2_char"
#define NCM_DRIVER_NAME "cdc_ncm"
#define ACCESSORY_PROBE_TIMEOUT_MS 5000
#define ACCESSORY_STR_LEN 64

struct accessory_match {
    struct usb_device *udev;
    int iap2_ifnum;
    int ncm_ifnum;
    int ncm_data_ifnum;
};

struct accessory_wait {
    struct list_head node;
    struct completion done;
    struct accessory_match match;
    bool found;
};

struct bind_wait_group {
    struct completion done;
    unsigned int pending;
};

struct bind_wait {
    struct list_head node;
    struct device *dev;
    const char *driver_name;
    struct bind_wait_group *group;
};

struct iap2_acc_entry {
    struct list_head node;
    struct usb_device *udev;
    struct iap2_acc_accessory *acc;
    struct device *dev;
};

struct iap2_acc_track {
    struct list_head node;
    struct usb_device *udev;
    struct iap2_acc_accessory *acc;
};

static LIST_HEAD(accessory_wait_list);
static DEFINE_SPINLOCK(accessory_wait_lock);

static LIST_HEAD(bind_wait_list);
static DEFINE_SPINLOCK(bind_wait_lock);
static LIST_HEAD(iap2_acc_list);
static LIST_HEAD(iap2_acc_track_list);
static DEFINE_MUTEX(iap2_acc_lock);

static struct kobject *iap2_kobj;
static struct class *iap2_acc_class;

static void iap2_acc_remove_usb(struct usb_device *udev);
static void iap2_acc_untrack_accessory_locked(struct iap2_acc_accessory *acc);
static struct iap2_acc_accessory *iap2_acc_get_locked(struct iap2_acc_accessory *acc);

static bool match_accessory_usb_device(struct usb_device *udev,
                                       struct accessory_match *match)
{
    struct usb_host_config *cfg;
    bool has_iap2 = false;
    bool has_ncm_ctrl = false;
    bool has_ncm_data = false;
    int i;

    if (!udev->actconfig)
        return false;

    cfg = udev->actconfig;
    match->udev = udev;
    match->iap2_ifnum = -1;
    match->ncm_ifnum = -1;
    match->ncm_data_ifnum = -1;

    for (i = 0; i < cfg->desc.bNumInterfaces; i++) {
        struct usb_interface_cache *iface = cfg->intf_cache[i];
        struct usb_interface_descriptor *desc = &iface->altsetting[0].desc;

        if (desc->bInterfaceClass == 0xFF &&
            desc->bInterfaceSubClass == 0xF0 &&
            desc->bInterfaceProtocol == 0x00) {
            has_iap2 = true;
            match->iap2_ifnum = desc->bInterfaceNumber;
        }

        if (desc->bInterfaceClass == 0x02 &&
            desc->bInterfaceSubClass == 0x0D) {
            has_ncm_ctrl = true;
            match->ncm_ifnum = desc->bInterfaceNumber;
        }

        if (desc->bInterfaceClass == 0x0A &&
            desc->bInterfaceSubClass == 0x00) {
            has_ncm_data = true;
            match->ncm_data_ifnum = desc->bInterfaceNumber;
        }
    }

    return has_iap2 && has_ncm_ctrl && has_ncm_data &&
           match->iap2_ifnum >= 0 && match->ncm_ifnum >= 0 &&
           match->ncm_data_ifnum >= 0;
}

struct accessory_search {
    struct accessory_match match;
    bool found;
};

static int find_accessory_usb_dev(struct usb_device *udev, void *data)
{
    struct accessory_search *search = data;

    if (search->found)
        return 0;

    if (!match_accessory_usb_device(udev, &search->match))
        return 0;

    search->match.udev = usb_get_dev(udev);
    search->found = true;
    return 1;
}

static int iap2_acc_find_existing_accessory(struct accessory_match *match)
{
    struct accessory_search search = {
        .found = false,
    };

    usb_for_each_dev(&search, find_accessory_usb_dev);
    if (!search.found)
        return -ENODEV;

    *match = search.match;
    return 0;
}

static void accessory_wait_add(struct accessory_wait *wait)
{
    unsigned long flags;

    spin_lock_irqsave(&accessory_wait_lock, flags);
    list_add_tail(&wait->node, &accessory_wait_list);
    spin_unlock_irqrestore(&accessory_wait_lock, flags);
}

static void accessory_wait_remove(struct accessory_wait *wait)
{
    unsigned long flags;

    spin_lock_irqsave(&accessory_wait_lock, flags);
    if (!list_empty(&wait->node))
        list_del_init(&wait->node);
    spin_unlock_irqrestore(&accessory_wait_lock, flags);
}

static void bind_wait_add(struct bind_wait *wait)
{
    unsigned long flags;

    spin_lock_irqsave(&bind_wait_lock, flags);
    list_add_tail(&wait->node, &bind_wait_list);
    spin_unlock_irqrestore(&bind_wait_lock, flags);
}

static void bind_wait_remove(struct bind_wait *wait)
{
    unsigned long flags;

    spin_lock_irqsave(&bind_wait_lock, flags);
    if (!list_empty(&wait->node))
        list_del_init(&wait->node);
    spin_unlock_irqrestore(&bind_wait_lock, flags);
}

static int iap2_acc_usb_notifier(struct notifier_block *nb,
                                 unsigned long action,
                                 void *data)
{
    struct usb_device *udev = data;
    struct accessory_wait *wait;
    struct accessory_match match;
    unsigned long flags;

    if (action == USB_DEVICE_REMOVE) {
        iap2_acc_remove_usb(udev);
        return NOTIFY_OK;
    }

    if (action != USB_DEVICE_ADD)
        return NOTIFY_DONE;

    if (!match_accessory_usb_device(udev, &match))
        return NOTIFY_DONE;

    spin_lock_irqsave(&accessory_wait_lock, flags);
    list_for_each_entry(wait, &accessory_wait_list, node) {
        if (wait->found)
            continue;

        wait->match = match;
        wait->match.udev = usb_get_dev(udev);
        wait->found = true;
        complete(&wait->done);
    }
    spin_unlock_irqrestore(&accessory_wait_lock, flags);

    return NOTIFY_OK;
}

static int iap2_acc_bind_bus_notifier(struct notifier_block *nb,
                                      unsigned long action,
                                      void *data)
{
    struct device *dev = data;
    struct bind_wait *wait;
    unsigned long flags;
    bool matched = false;

    if (action != BUS_NOTIFY_BOUND_DRIVER || !dev->driver)
        return NOTIFY_DONE;

    spin_lock_irqsave(&bind_wait_lock, flags);
    list_for_each_entry(wait, &bind_wait_list, node) {
        if (wait->dev != dev)
            continue;
        if (strcmp(wait->driver_name, dev->driver->name))
            continue;

        matched = true;
        if (wait->group->pending)
            wait->group->pending--;
        if (!wait->group->pending)
            complete(&wait->group->done);
    }
    spin_unlock_irqrestore(&bind_wait_lock, flags);

    return matched ? NOTIFY_OK : NOTIFY_DONE;
}

static struct notifier_block accessory_usb_nb = {
    .notifier_call = iap2_acc_usb_notifier,
};

static int iap2_find_devnode(struct usb_interface *intf, char *buf, size_t size)
{
    return iap2_usb_bulk_devnode_path(intf, buf, size);
}

static bool driver_is_bound(struct usb_interface *intf, const char *driver_name)
{
    return intf->dev.driver && !strcmp(intf->dev.driver->name, driver_name);
}

static int ensure_driver_bound(struct usb_interface *intf,
                               const char *module_name,
                               const char *driver_name)
{
    struct device *dev = &intf->dev;
    struct device_driver *drv;
    int ret;

    device_lock(dev);
    if (driver_is_bound(intf, driver_name)) {
        device_unlock(dev);
        return 0;
    }
    device_unlock(dev);

    request_module(module_name);

    drv = driver_find(driver_name, dev->bus);
    if (!drv)
        return -ENODEV;

    ret = device_driver_attach(drv, dev);

    if (ret == -EBUSY)
        return 0;
    if (ret < 0)
        return ret;
    return 0;
}

static int wait_for_bound_drivers(struct usb_interface *iap2_intf,
                                  struct usb_interface *ncm_intf)
{
    struct bind_wait_group group;
    struct notifier_block bind_nb = {
        .notifier_call = iap2_acc_bind_bus_notifier,
    };
    struct bind_wait waits[2];
    struct {
        struct usb_interface *intf;
        const char *module_name;
        const char *driver_name;
    } targets[2] = {
        { iap2_intf, IAP2_DRIVER_NAME, IAP2_DRIVER_NAME },
        { ncm_intf, NCM_DRIVER_NAME, NCM_DRIVER_NAME },
    };
    int ret = 0;
    int i;

    init_completion(&group.done);
    group.pending = 0;

    for (i = 0; i < ARRAY_SIZE(targets); i++) {
        INIT_LIST_HEAD(&waits[i].node);
        waits[i].dev = &targets[i].intf->dev;
        waits[i].driver_name = targets[i].driver_name;
        waits[i].group = &group;

        device_lock(&targets[i].intf->dev);
        if (!driver_is_bound(targets[i].intf, targets[i].driver_name)) {
            group.pending++;
            bind_wait_add(&waits[i]);
        }
        device_unlock(&targets[i].intf->dev);
    }

    ret = bus_register_notifier(iap2_intf->dev.bus, &bind_nb);
    if (ret) {
        dev_info(&iap2_intf->dev, "iap2_acc_probe_accessory: bind notifier failed %d\n",
                 ret);
        goto out_remove_waits;
    }

    for (i = 0; i < ARRAY_SIZE(targets); i++) {
        ret = ensure_driver_bound(targets[i].intf,
                                  targets[i].module_name,
                                  targets[i].driver_name);
        if (ret) {
            dev_info(&targets[i].intf->dev,
                     "iap2_acc_probe_accessory: ensure_driver_bound(%s) failed %d\n",
                     targets[i].driver_name, ret);
            break;
        }
    }

    if (!ret && group.pending &&
        !wait_for_completion_timeout(&group.done,
                                     msecs_to_jiffies(ACCESSORY_PROBE_TIMEOUT_MS))) {
        dev_info(&iap2_intf->dev,
                 "iap2_acc_probe_accessory: bind wait timeout pending=%u\n",
                 group.pending);
        ret = -ENODEV;
    }

    if (!ret)
        dev_info(&iap2_intf->dev,
                 "iap2_acc_probe_accessory: drivers bound iap2_if=%d ncm_if=%d\n",
                 iap2_intf->cur_altsetting->desc.bInterfaceNumber,
                 ncm_intf->cur_altsetting->desc.bInterfaceNumber);

    bus_unregister_notifier(iap2_intf->dev.bus, &bind_nb);

out_remove_waits:
    for (i = 0; i < ARRAY_SIZE(waits); i++)
        bind_wait_remove(&waits[i]);

    return ret;
}

static int iap2_acc_wait_for_match(struct accessory_match *match)
{
    struct accessory_wait wait;
    int ret;

    ret = iap2_acc_find_existing_accessory(match);
    if (!ret) {
        pr_info("iap2_acc_probe_accessory: instant match usb %03u:%03u iap2_if=%d ncm_if=%d data_if=%d\n",
                match->udev->bus->busnum, match->udev->devnum,
                match->iap2_ifnum, match->ncm_ifnum, match->ncm_data_ifnum);
        return 0;
    }

    INIT_LIST_HEAD(&wait.node);
    init_completion(&wait.done);
    wait.found = false;
    accessory_wait_add(&wait);

    ret = iap2_acc_find_existing_accessory(match);
    if (!ret) {
        if (wait.found)
            usb_put_dev(wait.match.udev);
        accessory_wait_remove(&wait);
        pr_info("iap2_acc_probe_accessory: late instant match usb %03u:%03u iap2_if=%d ncm_if=%d data_if=%d\n",
                match->udev->bus->busnum, match->udev->devnum,
                match->iap2_ifnum, match->ncm_ifnum, match->ncm_data_ifnum);
        return 0;
    }

    if (!wait_for_completion_timeout(&wait.done,
                                     msecs_to_jiffies(ACCESSORY_PROBE_TIMEOUT_MS))) {
        pr_info("iap2_acc_probe_accessory: accessory wait timeout\n");
        accessory_wait_remove(&wait);
        return -ENODEV;
    }

    *match = wait.match;
    accessory_wait_remove(&wait);
    pr_info("iap2_acc_probe_accessory: notified match usb %03u:%03u iap2_if=%d ncm_if=%d data_if=%d\n",
            match->udev->bus->busnum, match->udev->devnum,
            match->iap2_ifnum, match->ncm_ifnum, match->ncm_data_ifnum);
    return 0;
}

static struct usbnet *find_ncm_usbnet(struct accessory_match *match)
{
    struct usb_interface *ncm_intf;
    struct usb_interface *ncm_data_intf;
    struct usbnet *usbnet;

    ncm_intf = usb_ifnum_to_if(match->udev, match->ncm_ifnum);
    if (ncm_intf) {
        usbnet = usb_get_intfdata(ncm_intf);
        if (usbnet && usbnet->net) {
            dev_info(&ncm_intf->dev,
                     "iap2_acc_probe_accessory: usbnet on NCM ctrl if -> %s\n",
                     netdev_name(usbnet->net));
            return usbnet;
        }

        dev_info(&ncm_intf->dev,
                 "iap2_acc_probe_accessory: no usbnet on NCM ctrl if, intfdata=%px\n",
                 usbnet);
    }

    ncm_data_intf = usb_ifnum_to_if(match->udev, match->ncm_data_ifnum);
    if (!ncm_data_intf) {
        pr_info("iap2_acc_probe_accessory: missing NCM data interface %d\n",
                match->ncm_data_ifnum);
        return NULL;
    }

    usbnet = usb_get_intfdata(ncm_data_intf);
    if (!usbnet || !usbnet->net) {
        dev_info(&ncm_data_intf->dev,
                 "iap2_acc_probe_accessory: no usbnet on NCM data if, intfdata=%px\n",
                 usbnet);
        return NULL;
    }

    dev_info(&ncm_data_intf->dev,
             "iap2_acc_probe_accessory: usbnet on NCM data if -> %s\n",
             netdev_name(usbnet->net));

    return usbnet;
}

static struct iap2_acc_accessory *iap2_acc_build_accessory(struct accessory_match *match,
                                                           struct usb_interface *iap2_intf)
{
    struct usbnet *usbnet;
    struct iap2_acc_accessory *acc;
    int len;
    int ret;

    usbnet = find_ncm_usbnet(match);
    if (!usbnet) {
        pr_info("iap2_acc_probe_accessory: build_accessory missing usbnet\n");
        return ERR_PTR(-ENODEV);
    }

    acc = kzalloc(sizeof(*acc), GFP_KERNEL);
    if (!acc)
        return ERR_PTR(-ENOMEM);

    acc->udev = usb_get_dev(match->udev);
    acc->vendor_id = le16_to_cpu(match->udev->descriptor.idVendor);
    acc->product_id = le16_to_cpu(match->udev->descriptor.idProduct);
    acc->busnum = match->udev->bus->busnum;
    acc->devnum = match->udev->devnum;
    acc->iap2_ifnum = match->iap2_ifnum;
    len = usb_string(match->udev, match->udev->descriptor.iManufacturer,
                     acc->manufacturer, ACCESSORY_STR_LEN);
    if (len < 0)
        acc->manufacturer[0] = '\0';

    len = usb_string(match->udev, match->udev->descriptor.iProduct,
                     acc->product, ACCESSORY_STR_LEN);
    if (len < 0)
        acc->product[0] = '\0';

    strscpy(acc->ifname, netdev_name(usbnet->net), sizeof(acc->ifname));
    ret = iap2_find_devnode(iap2_intf, acc->iap2_devnode,
                            sizeof(acc->iap2_devnode));
    if (ret) {
        pr_info("iap2_acc_probe_accessory: failed to find iAP2 devnode: %d\n",
                ret);
        kfree(acc);
        return ERR_PTR(ret);
    }
    kref_init(&acc->kref);
    init_completion(&acc->disconnected);
    atomic_set(&acc->gone, 0);

    return acc;
}

static void iap2_acc_release(struct kref *kref)
{
    struct iap2_acc_accessory *acc =
        container_of(kref, struct iap2_acc_accessory, kref);

    usb_put_dev(acc->udev);
    kfree(acc);
}

static struct iap2_acc_accessory *iap2_acc_get_locked(struct iap2_acc_accessory *acc)
{
    kref_get(&acc->kref);
    return acc;
}

void iap2_acc_put_accessory(struct iap2_acc_accessory *acc)
{
    if (!acc)
        return;

    mutex_lock(&iap2_acc_lock);
    iap2_acc_untrack_accessory_locked(acc);
    mutex_unlock(&iap2_acc_lock);

    kref_put(&acc->kref, iap2_acc_release);
}

bool iap2_acc_is_gone(struct iap2_acc_accessory *acc)
{
    return acc && (atomic_read(&acc->gone) ||
                   completion_done(&acc->disconnected));
}

struct device *iap2_acc_device_get(struct iap2_acc_accessory *acc)
{
    struct iap2_acc_entry *entry;
    struct device *dev = NULL;

    if (!acc)
        return NULL;

    mutex_lock(&iap2_acc_lock);
    list_for_each_entry(entry, &iap2_acc_list, node) {
        if (entry->acc != acc)
            continue;

        dev = get_device(entry->dev);
        break;
    }
    mutex_unlock(&iap2_acc_lock);

    return dev;
}

static ssize_t vendor_id_show(struct device *dev,
                              struct device_attribute *attr, char *buf)
{
    struct iap2_acc_entry *entry = dev_get_drvdata(dev);

    return sysfs_emit(buf, "%04x\n", entry->acc->vendor_id);
}

static ssize_t product_id_show(struct device *dev,
                               struct device_attribute *attr, char *buf)
{
    struct iap2_acc_entry *entry = dev_get_drvdata(dev);

    return sysfs_emit(buf, "%04x\n", entry->acc->product_id);
}

static ssize_t manufacturer_show(struct device *dev,
                                 struct device_attribute *attr, char *buf)
{
    struct iap2_acc_entry *entry = dev_get_drvdata(dev);

    return sysfs_emit(buf, "%s\n", entry->acc->manufacturer);
}

static ssize_t product_show(struct device *dev,
                            struct device_attribute *attr, char *buf)
{
    struct iap2_acc_entry *entry = dev_get_drvdata(dev);

    return sysfs_emit(buf, "%s\n", entry->acc->product);
}

static ssize_t ifname_show(struct device *dev,
                           struct device_attribute *attr, char *buf)
{
    struct iap2_acc_entry *entry = dev_get_drvdata(dev);

    return sysfs_emit(buf, "%s\n", entry->acc->ifname);
}

static ssize_t iap2_devnode_show(struct device *dev,
                                 struct device_attribute *attr, char *buf)
{
    struct iap2_acc_entry *entry = dev_get_drvdata(dev);

    return sysfs_emit(buf, "%s\n", entry->acc->iap2_devnode);
}

static ssize_t busnum_show(struct device *dev,
                           struct device_attribute *attr, char *buf)
{
    struct iap2_acc_entry *entry = dev_get_drvdata(dev);

    return sysfs_emit(buf, "%u\n", entry->acc->busnum);
}

static ssize_t devnum_show(struct device *dev,
                           struct device_attribute *attr, char *buf)
{
    struct iap2_acc_entry *entry = dev_get_drvdata(dev);

    return sysfs_emit(buf, "%u\n", entry->acc->devnum);
}

static DEVICE_ATTR_RO(vendor_id);
static DEVICE_ATTR_RO(product_id);
static DEVICE_ATTR_RO(manufacturer);
static DEVICE_ATTR_RO(product);
static DEVICE_ATTR_RO(ifname);
static DEVICE_ATTR_RO(iap2_devnode);
static DEVICE_ATTR_RO(busnum);
static DEVICE_ATTR_RO(devnum);

static struct attribute *iap2_acc_attrs[] = {
    &dev_attr_vendor_id.attr,
    &dev_attr_product_id.attr,
    &dev_attr_manufacturer.attr,
    &dev_attr_product.attr,
    &dev_attr_ifname.attr,
    &dev_attr_iap2_devnode.attr,
    &dev_attr_busnum.attr,
    &dev_attr_devnum.attr,
    NULL,
};

ATTRIBUTE_GROUPS(iap2_acc);

static void iap2_acc_dev_release(struct device *dev)
{
    struct iap2_acc_entry *entry = dev_get_drvdata(dev);

    if (!entry)
        return;

    iap2_acc_put_accessory(entry->acc);
    usb_put_dev(entry->udev);
    kfree(entry);
}

static struct iap2_acc_entry *iap2_acc_find_locked(struct usb_device *udev)
{
    struct iap2_acc_entry *entry;

    list_for_each_entry(entry, &iap2_acc_list, node) {
        if (entry->udev == udev)
            return entry;
    }

    return NULL;
}

static struct iap2_acc_track *iap2_acc_find_track_locked(struct iap2_acc_accessory *acc)
{
    struct iap2_acc_track *track;

    list_for_each_entry(track, &iap2_acc_track_list, node) {
        if (track->acc == acc)
            return track;
    }

    return NULL;
}

static struct iap2_acc_track *iap2_acc_find_track_by_udev_locked(struct usb_device *udev)
{
    struct iap2_acc_track *track;

    list_for_each_entry(track, &iap2_acc_track_list, node) {
        if (track->udev == udev)
            return track;
    }

    return NULL;
}

static int iap2_acc_track_accessory(struct iap2_acc_accessory *acc)
{
    struct iap2_acc_track *track;

    track = kzalloc(sizeof(*track), GFP_KERNEL);
    if (!track)
        return -ENOMEM;

    INIT_LIST_HEAD(&track->node);
    track->udev = acc->udev;
    track->acc = acc;

    mutex_lock(&iap2_acc_lock);
    if (iap2_acc_find_track_by_udev_locked(acc->udev)) {
        mutex_unlock(&iap2_acc_lock);
        kfree(track);
        return -EEXIST;
    }
    list_add_tail(&track->node, &iap2_acc_track_list);
    mutex_unlock(&iap2_acc_lock);

    return 0;
}

static void iap2_acc_untrack_accessory_locked(struct iap2_acc_accessory *acc)
{
    struct iap2_acc_track *track = iap2_acc_find_track_locked(acc);

    if (!track)
        return;

    list_del(&track->node);
    kfree(track);
}

static void iap2_acc_unregister_locked(struct iap2_acc_entry *entry)
{
    list_del_init(&entry->node);
    device_unregister(entry->dev);
}

static int iap2_acc_publish(struct iap2_acc_accessory *acc)
{
    struct iap2_acc_entry *entry;
    struct iap2_acc_entry *old;
    int ret;

    if (!iap2_acc_class)
        return -ENODEV;

    entry = kzalloc(sizeof(*entry), GFP_KERNEL);
    if (!entry)
        return -ENOMEM;

    mutex_lock(&iap2_acc_lock);
    old = iap2_acc_find_locked(acc->udev);
    if (old)
        iap2_acc_unregister_locked(old);
    mutex_unlock(&iap2_acc_lock);

    INIT_LIST_HEAD(&entry->node);
    entry->acc = iap2_acc_get_locked(acc);
    entry->udev = usb_get_dev(acc->udev);
    entry->dev = device_create_with_groups(iap2_acc_class, NULL, MKDEV(0, 0),
                                           entry, NULL,
                                           "usb-%03u-%03u",
                                           acc->busnum, acc->devnum);
    if (IS_ERR(entry->dev)) {
        ret = PTR_ERR(entry->dev);
        usb_put_dev(entry->udev);
        kfree(entry);
        return ret;
    }

    mutex_lock(&iap2_acc_lock);
    list_add_tail(&entry->node, &iap2_acc_list);
    mutex_unlock(&iap2_acc_lock);

    return 0;
}

static void iap2_acc_remove_usb(struct usb_device *udev)
{
    struct iap2_acc_entry *entry;
    struct iap2_acc_track *track;

    mutex_lock(&iap2_acc_lock);
    list_for_each_entry(track, &iap2_acc_track_list, node) {
        if (track->acc->udev != udev)
            continue;
        if (atomic_read(&track->acc->gone))
            continue;

        atomic_set(&track->acc->gone, 1);
        complete_all(&track->acc->disconnected);
    }

    entry = iap2_acc_find_locked(udev);
    if (entry)
        iap2_acc_unregister_locked(entry);
    mutex_unlock(&iap2_acc_lock);
}

struct iap2_acc_accessory *iap2_acc_probe_accessory(void)
{
    struct accessory_match match;
    struct iap2_acc_track *track;
    struct usb_interface *iap2_intf;
    struct usb_interface *ncm_intf;
    struct iap2_acc_accessory *acc;
    int ret;

    ret = iap2_acc_wait_for_match(&match);
    if (ret) {
        pr_info("iap2_acc_probe_accessory: wait_for_accessory_match failed %d\n", ret);
        return ERR_PTR(-ENODEV);
    }

    iap2_intf = usb_ifnum_to_if(match.udev, match.iap2_ifnum);
    ncm_intf = usb_ifnum_to_if(match.udev, match.ncm_ifnum);
    if (!iap2_intf || !ncm_intf) {
        pr_info("iap2_acc_probe_accessory: missing interfaces iap2=%px ncm=%px\n",
                iap2_intf, ncm_intf);
        usb_put_dev(match.udev);
        return ERR_PTR(-ENODEV);
    }

    ret = wait_for_bound_drivers(iap2_intf, ncm_intf);
    if (ret) {
        pr_info("iap2_acc_probe_accessory: wait_for_bound_drivers failed %d\n", ret);
        usb_put_dev(match.udev);
        return ERR_PTR(-ENODEV);
    }

    mutex_lock(&iap2_acc_lock);
    track = iap2_acc_find_track_by_udev_locked(match.udev);
    if (track) {
        acc = iap2_acc_get_locked(track->acc);
        mutex_unlock(&iap2_acc_lock);
        usb_put_dev(match.udev);
        return acc;
    }
    mutex_unlock(&iap2_acc_lock);

    acc = iap2_acc_build_accessory(&match, iap2_intf);
    usb_put_dev(match.udev);
    if (!IS_ERR(acc)) {
        ret = iap2_acc_track_accessory(acc);
        if (ret) {
            if (ret == -EEXIST) {
                mutex_lock(&iap2_acc_lock);
                track = iap2_acc_find_track_by_udev_locked(acc->udev);
                if (track) {
                    struct iap2_acc_accessory *shared = iap2_acc_get_locked(track->acc);

                    mutex_unlock(&iap2_acc_lock);
                    iap2_acc_put_accessory(acc);
                    return shared;
                }
                mutex_unlock(&iap2_acc_lock);
            }
            iap2_acc_put_accessory(acc);
            return ERR_PTR(ret);
        }
    }

    ret = iap2_acc_publish(acc);
    if (ret) {
        pr_info("iap2_acc_probe_accessory: class publish failed %d\n", ret);
        iap2_acc_put_accessory(acc);
        return ERR_PTR(ret);
    }

    return acc;
}

static ssize_t scan_store(struct kobject *kobj,
                          struct kobj_attribute *attr,
                          const char *buf,
                          size_t count)
{
    struct iap2_acc_accessory *acc = iap2_acc_probe_accessory();

    if (IS_ERR(acc)) {
        pr_info("iap2_acc_probe_accessory failed: %ld\n", PTR_ERR(acc));
        return count;
    }

    pr_info("iap2_acc_probe_accessory: usb %03u:%03u vid=%04x pid=%04x mfg=\"%s\" product=\"%s\" if=%s iap2=%s\n",
            acc->busnum, acc->devnum, acc->vendor_id, acc->product_id,
            acc->manufacturer, acc->product, acc->ifname, acc->iap2_devnode);

    return count;
}

static struct kobj_attribute scan_attr =
    __ATTR(scan, 0200, NULL, scan_store);

static int __init iap2_scan_init(void)
{
    int ret;

    iap2_acc_class = class_create("iap2_accessory");
    if (IS_ERR(iap2_acc_class))
        return PTR_ERR(iap2_acc_class);
    iap2_acc_class->dev_groups = iap2_acc_groups;
    iap2_acc_class->dev_release = iap2_acc_dev_release;

    iap2_kobj = kobject_create_and_add("iap2_scan", kernel_kobj);
    if (!iap2_kobj) {
        class_destroy(iap2_acc_class);
        return -ENOMEM;
    }

    usb_register_notify(&accessory_usb_nb);

    ret = sysfs_create_file(iap2_kobj, &scan_attr.attr);
    if (ret) {
        usb_unregister_notify(&accessory_usb_nb);
        kobject_put(iap2_kobj);
        class_destroy(iap2_acc_class);
        return ret;
    }

    return 0;
}

static void __exit iap2_scan_exit(void)
{
    struct iap2_acc_entry *entry, *tmp;
    struct iap2_acc_track *track, *track_tmp;

    sysfs_remove_file(iap2_kobj, &scan_attr.attr);
    usb_unregister_notify(&accessory_usb_nb);
    kobject_put(iap2_kobj);

    mutex_lock(&iap2_acc_lock);
    list_for_each_entry_safe(entry, tmp, &iap2_acc_list, node)
        iap2_acc_unregister_locked(entry);
    list_for_each_entry_safe(track, track_tmp, &iap2_acc_track_list, node) {
        list_del(&track->node);
        kfree(track);
    }
    mutex_unlock(&iap2_acc_lock);

    class_destroy(iap2_acc_class);
}

module_init(iap2_scan_init);
module_exit(iap2_scan_exit);

EXPORT_SYMBOL_GPL(iap2_acc_probe_accessory);
EXPORT_SYMBOL_GPL(iap2_acc_put_accessory);
EXPORT_SYMBOL_GPL(iap2_acc_is_gone);
EXPORT_SYMBOL_GPL(iap2_acc_device_get);

MODULE_DESCRIPTION("Combo driver for iAP2 accessories (NCM+iAP2)");
MODULE_AUTHOR("CatPlay");
MODULE_LICENSE("GPL");
