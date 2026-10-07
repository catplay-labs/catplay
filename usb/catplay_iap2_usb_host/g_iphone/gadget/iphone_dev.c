// SPDX-License-Identifier: GPL-2.0
#include <linux/kernel.h>
#include <linux/module.h>
#include <linux/container_of.h>
#include <linux/usb.h>
#include <linux/sysfs.h>
#include <linux/kmod.h>
#include <linux/slab.h>

#include "iphone_dev.h"
#include "iphone_hid.h"
#include "../scan/iap2_scan.h"
#include "../iap2_char/iap2_transport.h"

/* Host settle time plus rebind debounce; no worker sleeps during this delay. */
#define IPHONE_ROLE_SWITCH_REBIND_DELAY_MS 2000
#define IPHONE_RECOVERY_COMMAND "/usr/bin/carlinkit_otalib usboot"
#define IPHONE_RECOVERY_COMMAND_GADGET "/usr/bin/carlinkit_otalib gadget"

static int iphone_dev_set_otg_role(struct g_iphone *iphone_gadget,
					       enum usb_role role);
static int iphone_dev_start_recovery(struct g_iphone *iphone_gadget);
static int iphone_dev_start_gadget(struct g_iphone *iphone_gadget);
static int iphone_dev_set_otg_role_internal(struct iphone_dev_data *data,
					    enum usb_role role,
					    bool manage_gadget_lifecycle);
static void iphone_dev_status_notify_workfn(struct work_struct *work);
static void iphone_dev_role_switch_failed(struct iphone_dev_data *data);
static void iphone_dev_maintenance_workfn(struct work_struct *work);
static void iphone_dev_disable_runtime(struct iphone_dev_data *data);
static void iphone_dev_recovery_workfn(struct work_struct *work);

static int iphone_dev_launch_recovery_process(const char *command)
{
	char *launcher_command;
	char *argv[] = {
		"/bin/sh",
		"-c",
		NULL,
		NULL
	};
	char *envp[] = {
		"HOME=/",
		"PATH=/usr/sbin:/usr/bin:/sbin:/bin",
		"TERM=linux",
		NULL
	};
	int ret;

	launcher_command = kasprintf(GFP_KERNEL,
				      "(%s </dev/null >/dev/null 2>&1 &)",
				      command);
	if (!launcher_command)
		return -ENOMEM;

	argv[2] = launcher_command;

	pr_info("iPhone: launching detached recovery process (fork/disown style)\n");
	ret = call_usermodehelper(argv[0], argv, envp, UMH_WAIT_PROC);
	if (ret)
		pr_err("iPhone: userspace exec failed: %d\n", ret);
	else
		pr_info("iPhone: detached recovery launcher finished\n");

	kfree(launcher_command);
	return ret;
}

static void iphone_dev_recovery_workfn(struct work_struct *work)
{
	struct iphone_dev_data *data =
		container_of(work, struct iphone_dev_data, recovery_work);
	unsigned long flags;
	const char *command;

	spin_lock_irqsave(&data->request_lock, flags);
	command = data->recovery_command;
	spin_unlock_irqrestore(&data->request_lock, flags);
	if (!READ_ONCE(data->runtime_stopped))
		iphone_dev_launch_recovery_process(command);
	spin_lock_irqsave(&data->request_lock, flags);
	data->recovery_command = NULL;
	spin_unlock_irqrestore(&data->request_lock, flags);
}

static void iphone_dev_notify_status_changed(struct g_iphone *iphone_gadget)
{
	struct iphone_dev_data *data =
		container_of(iphone_gadget, struct iphone_dev_data, g);

	queue_work(data->wq, &data->status_notify_work);
}

static void iphone_dev_notify(struct iphone_dev_data *data)
{
	struct device *owner_dev = READ_ONCE(data->owner_dev);

	if (!owner_dev)
		return;

	/*
	 * Pair with g_iphone_set_status() store-release so poll wakeups happen
	 * after the new value is globally visible to sysfs readers.
	 */
	sysfs_notify(&owner_dev->kobj, NULL, "status");
}

static void iphone_dev_status_notify_workfn(struct work_struct *work)
{
	struct iphone_dev_data *data =
		container_of(work, struct iphone_dev_data, status_notify_work);

	if (!READ_ONCE(data->runtime_stopped))
		iphone_dev_notify(data);
}

static void iphone_dev_remove_accessory_link_locked(struct iphone_dev_data *data)
{
	if (!data->owner_dev || !data->accessory_link_added)
		return;

	sysfs_remove_link(&data->owner_dev->kobj, "iap2_accessory");
	data->accessory_link_added = false;
}

static void iphone_dev_remove_iap2_link(struct iphone_dev_data *data)
{
	if (!data->iap2_link_added)
		return;
	sysfs_remove_link(&data->owner_dev->kobj, "iap2_devnode");
	data->iap2_link_added = false;
	/* The status worker may have notified before this unlink. */
	iphone_dev_notify(data);
}

static int iphone_dev_add_iap2_link(struct iphone_dev_data *data)
{
	int ret;

	iphone_dev_remove_iap2_link(data);
	ret = iap2_session_link_active(data->owner_dev);
	if (!ret)
		data->iap2_link_added = true;
	return ret;
}

static int iphone_dev_add_accessory_link_locked(struct iphone_dev_data *data,
						struct iap2_acc_accessory *acc)
{
	struct device *acc_dev;
	int ret;

	if (!data->owner_dev || !acc)
		return -ENODEV;

	acc_dev = iap2_acc_device_get(acc);
	if (!acc_dev)
		return -ENODEV;

	iphone_dev_remove_accessory_link_locked(data);
	ret = sysfs_create_link(&data->owner_dev->kobj, &acc_dev->kobj,
				"iap2_accessory");
	put_device(acc_dev);
	if (ret)
		return ret;

	data->accessory_link_added = true;
	return 0;
}

static int iphone_dev_unbind_gadget(struct iphone_dev_data *data, int reason)
{
	if (!data->driver_registered)
		return 0;

	pr_info("iPhone: unregistering gadget\n");
	iphone_dev_remove_iap2_link(data);
	/* Latch the reason before disabling endpoints can complete IO with ESHUTDOWN.
	 * Function unbind joins the queued stop before freeing backend resources.
	 */
	iphone_hid_close(data->g.hid, reason);
	usb_composite_unregister(&data->driver->drv);
	data->driver_registered = false;
	if (data->gadget_registered) {
		pr_warn("iPhone: gadget still marked registered after unregister\n");
		return -EBUSY;
	}

	/* After unbind, always leave USB_ROLE_DEVICE */
	int ret = iphone_dev_set_otg_role_internal(data, USB_ROLE_DEVICE, false);
	return ret;
}

static int iphone_dev_bind_gadget(struct iphone_dev_data *data)
{
	int ret;

	if (data->driver_registered)
		return 0;

	/* Before bind, always enter USB_ROLE_DEVICE */
	ret = iphone_dev_set_otg_role_internal(data, USB_ROLE_DEVICE, false);
	if (ret) {
		return ret;
	}

	pr_info("iPhone: registering gadget\n");
	ret = c2a_usb_composite_probe(&data->driver->drv);
	if (ret) {
		pr_warn("iPhone: gadget register failed: %d\n", ret);
		return ret;
	}
	data->driver_registered = true;
	ret = iphone_dev_add_iap2_link(data);
	if (ret) {
		iphone_dev_unbind_gadget(data, -ENODEV);
		return ret;
	}
	/* Setup may already have requested a role switch during probe. */
	if (!READ_ONCE(data->g.role_switch_requested))
		g_iphone_set_status(&data->g, Bind);
	/* Pair the published link and bind state with a post-publication wakeup. */
	iphone_dev_notify(data);

	return 0;
}

static void iphone_dev_clear_accessory_locked(struct iphone_dev_data *data)
{
	struct iap2_acc_accessory *acc = data->acc;

	if (acc)
		iphone_dev_remove_iap2_link(data);
	iphone_dev_remove_accessory_link_locked(data);
	data->acc = NULL;
	if (acc)
		iap2_acc_put_accessory(acc);
}

static void iphone_dev_maintenance_workfn(struct work_struct *work)
{
	struct iphone_dev_data *data =
		container_of(to_delayed_work(work), struct iphone_dev_data,
			     maintenance_work);
	bool connected;
	int ret;

	if (READ_ONCE(data->runtime_stopped))
		return;

	if (data->maintenance == IPHONE_MAINTENANCE_WATCH) {
		mutex_lock(&data->lock);
		connected = data->acc && !iap2_acc_is_gone(data->acc);
		if (!connected)
			iphone_dev_clear_accessory_locked(data);
		mutex_unlock(&data->lock);
		if (connected) {
			queue_delayed_work(data->wq, &data->maintenance_work,
					   msecs_to_jiffies(100));
			return;
		}
		pr_info("iPhone: accessory disconnected, reverting OTG role\n");
	} else if (data->maintenance == IPHONE_MAINTENANCE_REBIND) {
		pr_info("iPhone: rebinding gadget after role-switch failure debounce\n");
	} else {
		return;
	}

	data->maintenance = IPHONE_MAINTENANCE_NONE;
	WRITE_ONCE(data->g.role_switch_requested, false);
	ret = iphone_dev_set_otg_role_internal(data, USB_ROLE_DEVICE, true);
	if (ret)
		pr_warn("iPhone: gadget restore failed: %d\n", ret);
	else if (!READ_ONCE(data->g.role_switch_requested))
		g_iphone_set_status(&data->g, Initial);
}

static int iphone_dev_set_otg_role(struct g_iphone *iphone_gadget,
				       enum usb_role role)
{
	struct iphone_dev_data *data =
		container_of(iphone_gadget, struct iphone_dev_data, g);
	return iphone_dev_set_otg_role_internal(data, role, true);
}

static int iphone_dev_start_command(struct g_iphone *iphone_gadget,
				    const char *command)
{
	struct iphone_dev_data *data =
		container_of(iphone_gadget, struct iphone_dev_data, g);
	unsigned long flags;

	spin_lock_irqsave(&data->request_lock, flags);
	if (!data->recovery_command) {
		data->recovery_command = command;
		if (!queue_work(data->wq, &data->recovery_work))
			data->recovery_command = NULL;
	}
	spin_unlock_irqrestore(&data->request_lock, flags);
	return 0;
}

static int iphone_dev_start_recovery(struct g_iphone *iphone_gadget)
{
	return iphone_dev_start_command(iphone_gadget, IPHONE_RECOVERY_COMMAND);
}

static int iphone_dev_start_gadget(struct g_iphone *iphone_gadget)
{
	return iphone_dev_start_command(iphone_gadget,
					IPHONE_RECOVERY_COMMAND_GADGET);
}


static int iphone_dev_set_otg_role_internal(struct iphone_dev_data *data,
					    enum usb_role role,
					    bool manage_gadget_lifecycle)
{
	int ret;

	if (!data || !data->rs)
		return -ENODEV;
	if (role != USB_ROLE_HOST && role != USB_ROLE_DEVICE)
		return -EINVAL;

	/* Release the UDC and the HID session before the host-side bulk probe. */
	if (role == USB_ROLE_HOST && manage_gadget_lifecycle) {
		ret = iphone_dev_unbind_gadget(data, -ECONNRESET);
		if (ret)
			return ret;
	}

	ret = iphone_rs_set_role(data->rs, role);
	if (ret) {
		pr_warn("iPhone: failed to set OTG role: %d\n", ret);
		if (role == USB_ROLE_HOST && manage_gadget_lifecycle) {
			int restore = iphone_dev_bind_gadget(data);

			if (restore) {
				pr_warn("iPhone: gadget restore failed: %d\n", restore);
			}
		}
		return ret;
	}

	if (role == USB_ROLE_DEVICE && manage_gadget_lifecycle) {
		ret = iphone_dev_bind_gadget(data);
		if (ret)
			return ret;
	}

	return 0;
}

static void iphone_dev_role_switch_failed(struct iphone_dev_data *data)
{
	WRITE_ONCE(data->g.role_switch_requested, false);
	g_iphone_set_status(&data->g, RoleSwitchFailed);
	data->maintenance = IPHONE_MAINTENANCE_REBIND;
	mod_delayed_work(data->wq, &data->maintenance_work,
			 msecs_to_jiffies(IPHONE_ROLE_SWITCH_REBIND_DELAY_MS));
}

static void iphone_dev_role_switch_work(struct work_struct *work)
{
	struct iphone_dev_data *data =
		container_of(work, struct iphone_dev_data, role_switch_work);
	struct iap2_acc_accessory *acc;
	unsigned long flags;
	int ret;

	if (READ_ONCE(data->runtime_stopped) ||
	    !READ_ONCE(data->g.role_switch_requested))
		goto done;
	/* A status update during bind must not discard an accepted USB request. */
	g_iphone_set_status(&data->g, RoleSwitch);

	/* Both callbacks run on our ordered queue; maintenance cannot be running. */
	cancel_delayed_work(&data->maintenance_work);
	data->maintenance = IPHONE_MAINTENANCE_NONE;

	if (iphone_dev_set_otg_role_internal(data, USB_ROLE_HOST, true)) {
		pr_warn("iPhone: role-switch host transition failed\n");
		WRITE_ONCE(data->g.role_switch_requested, false);
		g_iphone_set_status(&data->g, Initial);
		goto done;
	}

	acc = iap2_acc_probe_accessory();
	if (READ_ONCE(data->runtime_stopped)) {
		if (!IS_ERR(acc))
			iap2_acc_put_accessory(acc);
		goto done;
	}
	if (IS_ERR(acc)) {
		pr_warn("iPhone: role-switch accessory probe failed: %ld\n",
			PTR_ERR(acc));
		iphone_dev_role_switch_failed(data);
		goto done;
	}

	pr_info("iPhone: role-switch accessory probe succeeded for if=%s\n",
		acc->ifname);

	mutex_lock(&data->lock);
	iphone_dev_clear_accessory_locked(data);
	data->acc = acc;
	ret = iphone_dev_add_accessory_link_locked(data, acc);
	if (!ret)
		ret = iphone_dev_add_iap2_link(data);
	if (ret)
		iphone_dev_clear_accessory_locked(data);
	mutex_unlock(&data->lock);
	if (ret) {
		pr_warn("iPhone: failed to publish bulk iAP2 links: %d\n", ret);
		iphone_dev_role_switch_failed(data);
		goto done;
	}

	g_iphone_set_status(&data->g, Accessory);
	/* A previously queued status worker can run before the bulk link exists. */
	iphone_dev_notify(data);
	data->maintenance = IPHONE_MAINTENANCE_WATCH;
	mod_delayed_work(data->wq, &data->maintenance_work,
			 msecs_to_jiffies(100));
done:
	spin_lock_irqsave(&data->request_lock, flags);
	data->role_switch_work_scheduled = false;
	spin_unlock_irqrestore(&data->request_lock, flags);
}

static int iphone_dev_start_role_switch_probe(struct g_iphone *iphone_gadget)
{
	struct iphone_dev_data *data =
		container_of(iphone_gadget, struct iphone_dev_data, g);
	unsigned long flags;

	spin_lock_irqsave(&data->request_lock, flags);
	if (!data->role_switch_work_scheduled) {
		data->role_switch_work_scheduled = true;
		if (!queue_work(data->wq, &data->role_switch_work))
			data->role_switch_work_scheduled = false;
	}
	spin_unlock_irqrestore(&data->request_lock, flags);
	return 0;
}

static int iphone_dev_driver_bind(struct usb_composite_dev *cdev)
{
	struct iphone_dev_driver *ipdrv =
		container_of(cdev->driver, struct iphone_dev_driver, drv);
	struct iphone_dev_data *data = ipdrv->data;
	data->cdev = cdev;
	data->gadget_registered = true;

	pr_info("iPhone: driver binding 4 configurations\n");

	int ret;

	for (int i = 0; i < 4; i++)
	{
		ret = usb_add_config(cdev, &data->usb_configs[i].cfg, iphone_do_config);
		if (ret)
			return ret;
	}

	return 0;
}

static int iphone_dev_driver_unbind(struct usb_composite_dev *cdev)
{
	struct iphone_dev_driver *ipdrv =
		container_of(cdev->driver, struct iphone_dev_driver, drv);
	struct iphone_dev_data *data = ipdrv->data;

	pr_info("iPhone: driver unbind");

	iphone_dev_remove_iap2_link(data);
	mutex_lock(&data->lock);
	iphone_dev_clear_accessory_locked(data);
	mutex_unlock(&data->lock);
	data->cdev = NULL;
	data->gadget_registered = false;
	
	return 0;
}

struct iphone_initial_bind {
	struct work_struct work;
	struct iphone_dev_data *data;
	int result;
};

static void iphone_dev_initial_bind_workfn(struct work_struct *work)
{
	struct iphone_initial_bind *bind =
		container_of(work, struct iphone_initial_bind, work);

	bind->result = iphone_dev_bind_gadget(bind->data);
	if (bind->result)
		iphone_dev_disable_runtime(bind->data);
}

struct iphone_dev_data *iphone_dev_alloc(struct device *owner_dev, char *udc_name, char* serial)
{
	int ret = -ENOMEM;
	struct iphone_dev_data *data = NULL;
	struct iphone_initial_bind bind;
	size_t i;

	if (!serial)
		serial = DEFAULT_IPHONE_SERIAL;

	data = kzalloc(sizeof(*data), GFP_KERNEL);
	if (!data)
		return ERR_PTR(-ENOMEM);
	mutex_init(&data->lock);
	spin_lock_init(&data->request_lock);

	/* copy device descriptor */
	data->dev_desc = iphone_device_desc;
	data->owner_dev = owner_dev;

	/* copy strings table */
	data->stringtab = kmemdup(iphone_strings,
	                           sizeof(iphone_strings),
	                           GFP_KERNEL);
	if (!data->stringtab)
		goto fail;

	for (i = 0; data->stringtab[i].s; i++) {
		if (data->stringtab[i].id == 3)
			data->stringtab[i].s = data->serial;
		else if (data->stringtab[i].id == 4)
			data->stringtab[i].s = data->serial_r;
	}

	/* copy serial and serial_r */
	strscpy(data->g.serial, serial, sizeof(data->g.serial));
	strscpy(data->serial, serial, sizeof(data->serial));
	snprintf(data->serial_r, sizeof(data->serial_r), "%s-R", serial);

	/* init gadget_strings */
	data->gadget_strings.language = 0x0409;
	data->gadget_strings.strings  = data->stringtab;

	/* init gadget_strings_array */
	data->gadget_strings_array[0] = &data->gadget_strings;
	data->gadget_strings_array[1] = NULL;

	/* init usb_composite_driver */
	data->driver = kzalloc(sizeof(*data->driver), GFP_KERNEL);
	if (!data->driver)
		goto fail;

	data->driver->drv = (struct usb_composite_driver) {
		.name       = data->driver_name,
		.dev        = &data->dev_desc,
		.strings    = data->gadget_strings_array,
		.max_speed  = USB_SPEED_SUPER,
		.bind       = iphone_dev_driver_bind,
		.unbind     = iphone_dev_driver_unbind,
		.udc_name   = data->udc_name_vec,
	};
	data->driver->data = data;
	data->udc_name_vec[0] = data->udc;
	data->udc_name_vec[1] = NULL;

	/* copy UDC name if provided */
	if (!udc_name) {
		data->driver->drv.udc_name = NULL;
		data->udc_auto = true;
	} else {
		strscpy(data->udc, udc_name, sizeof(data->udc));
	}

	/* create runtime-unique driver name */
	if (!udc_name) {
		snprintf(data->driver_name, sizeof(data->driver_name), "iphone");
	} else {
		snprintf(data->driver_name, sizeof(data->driver_name), "iphone-%s", udc_name);
	}

	/* init usb configs */
	for (int i = 0; i < ARRAY_SIZE(iphone_configs) && i < 4; i++) {
		data->usb_configs[i].g = &data->g;
		data->usb_configs[i].cfg = iphone_configs[i];
	}

	data->driver_registered = false;
	data->gadget_registered = false;
	data->rs = iphone_rs_sysfs_create(udc_name);
	if (IS_ERR(data->rs)) {
		ret = PTR_ERR(data->rs);
		data->rs = NULL;
		goto fail;
	}
	data->g.set_otg_role = iphone_dev_set_otg_role;
	data->g.start_role_switch_probe = iphone_dev_start_role_switch_probe;
	data->g.start_recovery = iphone_dev_start_recovery;
	data->g.start_gadget = iphone_dev_start_gadget;
	data->g.notify_status_changed = iphone_dev_notify_status_changed;
	INIT_WORK(&data->status_notify_work, iphone_dev_status_notify_workfn);
	INIT_WORK(&data->role_switch_work, iphone_dev_role_switch_work);
	INIT_WORK(&data->recovery_work, iphone_dev_recovery_workfn);
	INIT_DELAYED_WORK(&data->maintenance_work, iphone_dev_maintenance_workfn);
	data->wq = alloc_ordered_workqueue("iphone-%s", 0, dev_name(owner_dev));
	if (!data->wq)
		goto fail;

	pr_info("iPhone: initial gadget bind with udc_name '%s'\n", udc_name);

	/* Early USB requests queue behind this bind, never alongside it. */
	bind.data = data;
	INIT_WORK_ONSTACK(&bind.work, iphone_dev_initial_bind_workfn);
	queue_work(data->wq, &bind.work);
	flush_work(&bind.work);
	destroy_work_on_stack(&bind.work);
	ret = bind.result;
	if (ret) {
		pr_warn("iPhone: initial gadget bind failed: %d\n", ret);
		goto fail;
	}

	pr_debug("iPhone: initial gadget bind complete\n");
	return data;

fail:
	iphone_dev_free(data);
	return ERR_PTR(ret);
}

/* Called by initial-bind failure or the external owner, never concurrently. */
static void iphone_dev_disable_runtime(struct iphone_dev_data *data)
{
	if (data->runtime_stopped)
		return;
	WRITE_ONCE(data->runtime_stopped, true);
	disable_work(&data->role_switch_work);
	disable_delayed_work(&data->maintenance_work);
	disable_work(&data->recovery_work);
	disable_work(&data->status_notify_work);
}

static void iphone_dev_stop_runtime(struct iphone_dev_data *data)
{
	if (!data->wq)
		return;

	iphone_dev_disable_runtime(data);
	flush_workqueue(data->wq);
	/* Keep wq and data alive while unregister joins the USB producers.
	 * Their late queue attempts are harmless: all work stays disabled.
	 */
	iphone_dev_unbind_gadget(data, -ENODEV);
	destroy_workqueue(data->wq);

	mutex_lock(&data->lock);
	iphone_dev_clear_accessory_locked(data);
	mutex_unlock(&data->lock);
}

int iphone_dev_free(struct iphone_dev_data *data)
{
	if (!data)
		return -ENODEV;

	iphone_dev_stop_runtime(data);
	iphone_rs_sysfs_destroy(data->rs);
	kfree(data->stringtab);
	kfree(data->driver);
	kfree(data);
	return 0;
}
