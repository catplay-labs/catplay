// SPDX-License-Identifier: GPL-2.0
#pragma once

#include <linux/usb/role.h>
#include <linux/compiler.h>

enum GadgetStatus
{
	Initial,
	Bind,
	Enabled,
	Disabled,
	Suspended,
	RoleSwitch,
	RoleSwitchFailed,
	Accessory,
	Unbind
};

struct iphone_hid;

struct g_iphone
{
	char serial[41];

	int active_config;
	struct iphone_hid *hid;
	bool role_switch_requested;
	enum GadgetStatus status;
	int (*set_otg_role)(struct g_iphone *iphone_gadget, enum usb_role role);
	int (*start_role_switch_probe)(struct g_iphone *iphone_gadget);
	int (*start_recovery)(struct g_iphone *iphone_gadget);
	int (*start_gadget)(struct g_iphone *iphone_gadget);
	void (*notify_status_changed)(struct g_iphone *iphone_gadget);
};

static inline enum GadgetStatus g_iphone_get_status(struct g_iphone *iphone_gadget)
{
	return smp_load_acquire(&iphone_gadget->status);
}

void g_iphone_set_status(struct g_iphone *iphone_gadget, enum GadgetStatus status);
int g_iphone_set_otg_role(struct g_iphone *iphone_gadget, enum usb_role role);
int g_iphone_start_role_switch_probe(struct g_iphone *iphone_gadget);
int g_iphone_start_recovery(struct g_iphone *iphone_gadget);
int g_iphone_start_gadget(struct g_iphone *iphone_gadget);
char *g_iphone_status_str(enum GadgetStatus status);
