// SPDX-License-Identifier: GPL-2.0
#include <linux/err.h>
#include <linux/fs.h>
#include <linux/fs_struct.h>
#include <linux/slab.h>

#include "iphone_rs.h"

struct iphone_rs_sysfs {
	struct iphone_rs rs;
	struct path root;
	char role_path[256];
	enum usb_role cached_role;
};

static const char *iphone_rs_sysfs_name(const char *udc_name)
{
	if (!strcmp(udc_name, "2184200.usb"))
		return "ci_hdrc.1";
	/* V821 exposes MUSB through a child UDC of the USB glue device. */
	if (!strcmp(udc_name, "musb-hdrc.1.auto"))
		return "44100000.usb";

	return udc_name;
}

static int iphone_rs_sysfs_set_role(struct iphone_rs *rs, enum usb_role role)
{
	struct iphone_rs_sysfs *sysfs = container_of(rs, struct iphone_rs_sysfs, rs);
	struct file *file;
	const char *role_str;
	size_t role_len;
	ssize_t written;
	loff_t pos = 0;

	switch (role) {
	case USB_ROLE_HOST:
		role_str = "host";
		break;
	case USB_ROLE_DEVICE:
		role_str = "device";
		break;
	default:
		return -EINVAL;
	}
	if (sysfs->cached_role == role)
		return 0;

	file = file_open_root(&sysfs->root, sysfs->role_path + 1, O_WRONLY, 0);
	if (IS_ERR(file)) {
		pr_warn("iPhone: cannot open role switch %s: %ld\n",
			sysfs->role_path, PTR_ERR(file));
		return PTR_ERR(file);
	}
	role_len = strlen(role_str);
	written = kernel_write(file, role_str, role_len, &pos);
	filp_close(file, NULL);
	if (written < 0)
		return written;
	if (written != role_len)
		return -EIO;

	sysfs->cached_role = role;
	pr_info("iPhone: setting usb role: %s -> %s\n", sysfs->role_path, role_str);
	return 0;
}

static const struct iphone_rs_ops iphone_rs_sysfs_ops = {
	.set_role = iphone_rs_sysfs_set_role,
};

struct iphone_rs *iphone_rs_sysfs_create(const char *udc_name)
{
	struct iphone_rs_sysfs *sysfs;
	int len;

	if (!udc_name || !udc_name[0] || !current->fs)
		return ERR_PTR(-ENODEV);
	sysfs = kzalloc(sizeof(*sysfs), GFP_KERNEL);
	if (!sysfs)
		return ERR_PTR(-ENOMEM);
	len = snprintf(sysfs->role_path, sizeof(sysfs->role_path),
		       "/sys/class/usb_role/%s-role-switch/role",
		       iphone_rs_sysfs_name(udc_name));
	if (len < 0 || len >= sizeof(sysfs->role_path)) {
		kfree(sysfs);
		return ERR_PTR(-ENAMETOOLONG);
	}
	get_fs_root(current->fs, &sysfs->root);
	sysfs->cached_role = USB_ROLE_NONE;
	sysfs->rs.ops = &iphone_rs_sysfs_ops;
	return &sysfs->rs;
}

void iphone_rs_sysfs_destroy(struct iphone_rs *rs)
{
	struct iphone_rs_sysfs *sysfs;

	if (!rs)
		return;
	sysfs = container_of(rs, struct iphone_rs_sysfs, rs);
	path_put(&sysfs->root);
	kfree(sysfs);
}
