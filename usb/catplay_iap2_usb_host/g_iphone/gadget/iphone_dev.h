// SPDX-License-Identifier: GPL-2.0
#pragma once

#include <linux/kernel.h>
#include <linux/module.h>
#include <linux/mutex.h>
#include <linux/spinlock.h>
#include <linux/usb/role.h>
#include <linux/workqueue.h>

#include "libcomposite/composite.h"

#include "f_iphone.h"
#include "g_iphone.h"
#include "iphone_rs.h"

enum iphone_maintenance {
	IPHONE_MAINTENANCE_NONE,
	IPHONE_MAINTENANCE_REBIND,
	IPHONE_MAINTENANCE_WATCH,
};

struct iphone_dev_data
{
    struct device *owner_dev;
	char serial[41];
	char serial_r[64];

	char udc[64]; /* or NULL for auto */
	struct iphone_rs *rs;
	char *udc_name_vec[2];
	char driver_name[64];
	bool udc_auto;

	struct g_iphone g;

	struct usb_gadget_strings gadget_strings;
	struct usb_gadget_strings *gadget_strings_array[2];

	struct usb_string *stringtab;
	struct iphone_dev_driver *driver; /* container_of: usb_composite_driver */
	struct f_iphone_usb_config usb_configs[4]; 
	struct usb_device_descriptor dev_desc;
	struct usb_composite_dev *cdev;
	/* Control work is serialized; USB callbacks only enqueue requests. */
	struct workqueue_struct *wq;
	spinlock_t request_lock;
	bool runtime_stopped;
	struct work_struct status_notify_work;
	struct work_struct role_switch_work;
	struct work_struct recovery_work;
	struct delayed_work maintenance_work;
	enum iphone_maintenance maintenance;
	struct mutex lock;
	struct iap2_acc_accessory *acc;
	bool accessory_link_added;
	bool iap2_link_added;
	bool role_switch_work_scheduled;
	const char *recovery_command;
	bool driver_registered;
	bool gadget_registered;
};

struct iphone_dev_driver { /* container_of: usb_composite_driver */
	struct usb_composite_driver drv;
	struct iphone_dev_data *data;
};

struct iphone_dev_data *iphone_dev_alloc(struct device *owner_dev, char *udc_name, char* serial);
int iphone_dev_free(struct iphone_dev_data *data);
