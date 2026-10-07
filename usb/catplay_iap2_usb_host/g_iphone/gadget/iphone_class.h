// SPDX-License-Identifier: GPL-2.0
#pragma once

/* iPhone device entry */
struct iphone_dev {
	char name[64];
	struct list_head list;   /* list of devices */
	struct device *dev;      /* sysfs device */
    struct iphone_dev_data *iphone; /* iphone driver (if initialized) */

	/* bind config */
	char udc[64];
	char serial[41];
};

/* List of all iPhone devices*/
static LIST_HEAD(iphone_device_list);
static DEFINE_MUTEX(iphone_device_list_lock);
static DEFINE_MUTEX(iphone_init_lock);

int iphone_add(char *name);
int iphone_remove(char *name);

int iphone_bind(struct device* device);
int iphone_unbind(struct device* device);

int iphone_class_register(void);
void iphone_class_unregister(void);

struct device* iphone_get_device(char *name);
void iphone_put_device(struct device* device);

int iphone_get_serial(struct device* device, char* out, int size);
int iphone_get_udc(struct device* device, char* out, int size);

int iphone_set_serial(struct device* device, char* serial);
int iphone_set_udc(struct device* device, char* udc);



