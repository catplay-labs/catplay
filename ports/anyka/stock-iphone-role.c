// SPDX-License-Identifier: GPL-2.0
/* Anyka stock firmware switches roles by exchanging controller modules.
 * The absolute RAM helper path is a module parameter. No shell is invoked.
 */
#include <linux/module.h>
#include <linux/kmod.h>
#include <linux/slab.h>
#include "iphone_rs.h"

static char *role_helper;
module_param(role_helper, charp, 0400);
MODULE_PARM_DESC(role_helper, "Absolute executable that switches Anyka USB role");

static int anyka_set_role(struct iphone_rs *rs, enum usb_role role)
{
	char *argv[] = { role_helper, NULL, NULL };
	char *envp[] = { "PATH=/usr/sbin:/usr/bin:/sbin:/bin", NULL };
	if (role != USB_ROLE_DEVICE && role != USB_ROLE_HOST)
		return -EINVAL;
	if (!role_helper) {
		/* Device-only bring-up leaves the stock UDC running. */
		return role == USB_ROLE_DEVICE ? 0 : -EOPNOTSUPP;
	}
	argv[1] = role == USB_ROLE_HOST ? "host" : "device";
	return call_usermodehelper(role_helper, argv, envp, UMH_WAIT_PROC);
}
static const struct iphone_rs_ops anyka_ops = { .set_role = anyka_set_role };

struct iphone_rs *iphone_rs_sysfs_create(const char *udc, const char *rs_name)
{
	struct iphone_rs *rs;
	if ((udc && udc[0] && strcmp(udc, "20200000.usb")) ||
	    (role_helper && role_helper[0] != '/'))
		return ERR_PTR(-EINVAL);
	rs = kzalloc(sizeof(*rs), GFP_KERNEL);
	if (!rs)
		return ERR_PTR(-ENOMEM);
	rs->ops = &anyka_ops;
	return rs;
}
void iphone_rs_sysfs_destroy(struct iphone_rs *rs)
{
	kfree(rs);
}
