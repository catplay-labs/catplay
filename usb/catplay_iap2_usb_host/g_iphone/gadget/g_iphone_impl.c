// SPDX-License-Identifier: GPL-2.0
#include <linux/kernel.h>
#include <linux/module.h>
#include "g_iphone.h"

void g_iphone_set_status(struct g_iphone *iphone_gadget, enum GadgetStatus status)
{
	if (g_iphone_get_status(iphone_gadget) != status)
	{
		smp_store_release(&iphone_gadget->status, status);
		pr_info("iPhone: status -> %s", g_iphone_status_str(status));
		if (iphone_gadget->notify_status_changed)
			iphone_gadget->notify_status_changed(iphone_gadget);
		if (status == RoleSwitch && iphone_gadget->start_role_switch_probe) {
			int ret = iphone_gadget->start_role_switch_probe(iphone_gadget);

			if (ret)
				pr_warn("iPhone: async accessory probe start failed: %d\n",
					ret);
		}
	}
}

int g_iphone_set_otg_role(struct g_iphone *iphone_gadget, enum usb_role role)
{
	if (!iphone_gadget || !iphone_gadget->set_otg_role)
		return -EOPNOTSUPP;

	return iphone_gadget->set_otg_role(iphone_gadget, role);
}

int g_iphone_start_role_switch_probe(struct g_iphone *iphone_gadget)
{
	if (!iphone_gadget || !iphone_gadget->start_role_switch_probe)
		return -EOPNOTSUPP;

	return iphone_gadget->start_role_switch_probe(iphone_gadget);
}

int g_iphone_start_recovery(struct g_iphone *iphone_gadget)
{
	if (!iphone_gadget || !iphone_gadget->start_recovery)
		return -EOPNOTSUPP;

	return iphone_gadget->start_recovery(iphone_gadget);
}

int g_iphone_start_gadget(struct g_iphone *iphone_gadget)
{
	if (!iphone_gadget || !iphone_gadget->start_gadget)
		return -EOPNOTSUPP;

	return iphone_gadget->start_gadget(iphone_gadget);
}

char *g_iphone_status_str(enum GadgetStatus status)
{
    switch (status)
    {
    case Initial:
        return "initial";
    case Bind:
        return "bind";
    case Enabled:
        return "enabled";
    case Disabled:
        return "disabled";
    case Suspended:
        return "suspended";
    case RoleSwitch:
        return "roleswitch";
    case RoleSwitchFailed:
        return "roleswitchfailed";
    case Accessory:
        return "accessory";
    case Unbind:
        return "unbind";
    default:
        return "unknown";
    }
}
