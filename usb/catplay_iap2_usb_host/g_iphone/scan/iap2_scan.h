#ifndef __IAP2_SCAN_H
#define __IAP2_SCAN_H

#include <linux/completion.h>
#include <linux/if.h>
#include <linux/kref.h>
#include <linux/types.h>

struct usb_device;
struct device;

struct iap2_acc_accessory {
    struct kref kref;
    struct usb_device *udev;
    u16 vendor_id;
    u16 product_id;
    u8 busnum;
    u8 devnum;
    u8 iap2_ifnum;
    char manufacturer[64];
    char product[64];
    char ifname[IFNAMSIZ];
    char iap2_devnode[64];
    struct completion disconnected;
    atomic_t gone;
};

struct iap2_acc_accessory *iap2_acc_probe_accessory(void);
void iap2_acc_put_accessory(struct iap2_acc_accessory *acc);
bool iap2_acc_is_gone(struct iap2_acc_accessory *acc);
struct device *iap2_acc_device_get(struct iap2_acc_accessory *acc);

#endif
