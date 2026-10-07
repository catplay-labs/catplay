// SPDX-License-Identifier: GPL-2.0
#pragma once

#include <linux/usb/role.h>

struct iphone_rs;

struct iphone_rs_ops {
	/* May sleep. Caller serializes calls; only HOST and DEVICE are valid. */
	int (*set_role)(struct iphone_rs *rs, enum usb_role role);
};

struct iphone_rs {
	const struct iphone_rs_ops *ops;
};

static inline int iphone_rs_set_role(struct iphone_rs *rs, enum usb_role role)
{
	return rs->ops->set_role(rs, role);
}

/* Create in the caller's filesystem context; returns ERR_PTR on failure. */
struct iphone_rs *iphone_rs_sysfs_create(const char *udc_name);
/* Paired with sysfs_create; accepts NULL. All set_role calls must be joined. */
void iphone_rs_sysfs_destroy(struct iphone_rs *rs);
