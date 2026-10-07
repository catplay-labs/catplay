// SPDX-License-Identifier: GPL-2.0
#include <linux/kernel.h>
#include <linux/module.h>
#include "libcomposite/composite.h"
#include "iphone_dev.h"
#include "iphone_class.h"

/* Class and default device instance */
static struct class *iphone_class;

/* sysfs - serial */
static ssize_t serial_show(struct device *dev,
								  struct device_attribute *attr, char *buf)
{
	device_lock(dev);
	struct iphone_dev *data = dev_get_drvdata(dev);
	if (!data)
	{
		device_unlock(dev);
		return -ENODEV;
	}

	int ret = sysfs_emit(buf, "%s\n", data->serial);
	device_unlock(dev);
	return ret;
}

#define ATTR_LOAD(buf_out, buf_in, count_in)                          \
	do {                                                              \
		size_t __len = min_t(size_t, (count_in), sizeof(buf_out) - 1);\
		memcpy((buf_out), (buf_in), __len);                           \
		(buf_out)[__len] = '\0';                                      \
		strim(buf_out);                                               \
	} while (0)

static ssize_t serial_store(struct device *dev,
								   struct device_attribute *attr,
								   const char *buf, size_t count)
{
	char serial[64];
	ATTR_LOAD(serial, buf, count);

	int ret = iphone_set_serial(dev, serial);
	if (ret)
		return ret;

	return count;
}

static DEVICE_ATTR_RW(serial);

/* sysfs - udc */
static ssize_t udc_show(struct device *dev,
								  struct device_attribute *attr, char *buf)
{
	device_lock(dev);
	struct iphone_dev *data = dev_get_drvdata(dev);
	if (!data)
	{
		device_unlock(dev);
		return -ENODEV;
	}

	int ret = sysfs_emit(buf, "%s\n", data->udc);
	device_unlock(dev);
	return ret;
}

static ssize_t udc_store(struct device *dev,
								   struct device_attribute *attr,
								   const char *buf, size_t count)
{
	char udc[64];
	ATTR_LOAD(udc, buf, count);

	int ret = iphone_set_udc(dev, udc);
	if (ret)
		return ret;

	return count;
}

static DEVICE_ATTR_RW(udc);

/* sysfs - bind/unbind */
static ssize_t bind_store(struct device *dev,
								   struct device_attribute *attr,
								   const char *buf, size_t count)
{
	if (count == 0)
		return -EINVAL;

	int ret;
	if (buf[0] == '0') { /* unbind */
		ret = iphone_unbind(dev);
		if (ret)
			return ret;
	} else if (buf[0] == '1') { /* bind */
		ret = iphone_bind(dev);
		if (ret)
			return ret;
	} else {
		return -EINVAL;
	}

	return count;
}

static DEVICE_ATTR_WO(bind);

/* sysfs - status */
static ssize_t status_show(struct device *dev,
								  struct device_attribute *attr, char *buf)
{
	device_lock(dev);
	struct iphone_dev *data = dev_get_drvdata(dev);
	if (!data)
	{
		device_unlock(dev);
		return -ENODEV;
	}

	enum GadgetStatus status = Initial /* Unbind */;
	if (data->iphone) {
		status = g_iphone_get_status(&data->iphone->g);
	}

	int ret = sysfs_emit(buf, "%s\n", g_iphone_status_str(status));
	device_unlock(dev);
	return ret;
}

static DEVICE_ATTR_RO(status);

/* sysfs - mkdir */
static ssize_t create_store(const struct class *cls,
                                        const struct class_attribute *attr,
                                        const char *buf, size_t count)
{
    char name[64];
    snprintf(name, sizeof(name), "%.*s", (int)min(count, sizeof(name) - 1), buf);
    strim(name);

    if (!*name)
        return -EINVAL;

    pr_info("iPhone: requested create of device '%s'\n", name);
    int ret = iphone_add(name);
	if (ret)
		return ret;

    return count;
}

static CLASS_ATTR_WO(create);

/* sysfs - rmdir */
static ssize_t remove_store(const struct class *cls,
                                        const struct class_attribute *attr,
                                        const char *buf, size_t count)
{
    char name[64];
    snprintf(name, sizeof(name), "%.*s", (int)min(count, sizeof(name) - 1), buf);
    strim(name);

    if (!*name)
        return -EINVAL;

    pr_info("iPhone: requested rm of device '%s'\n", name);
    int ret = iphone_remove(name);
	if (ret)
		return ret;

    return count;
}

static CLASS_ATTR_WO(remove);

static void iphone_release_cb(struct device *dev)
{
	struct iphone_dev *data = dev_get_drvdata(dev);
	if (data && data->iphone)
	{
		/* If we reach here this is likely a bug anyway as iphone should be freed by unbind/remove/class_unregister */
		iphone_dev_free(data->iphone); /* possibly kick this into background thread if it crashes in atomic context? */
	}

	kfree(data);
	module_put(THIS_MODULE);
    pr_info("iPhone: released device '%s'\n", dev_name(dev));
}

/* ---------------- */

int iphone_add(char *name)
{
    int ret = 0;

	mutex_lock(&iphone_init_lock);
	struct iphone_dev *d = kzalloc(sizeof(*d), GFP_KERNEL);
    if (!d) {
        ret = -ENOMEM;
        goto fail;
    }

	if (!iphone_class) {
		ret = -ENODEV;
		goto fail;
	}

	struct device *dev = device_create(iphone_class, NULL, 0, NULL, "%s", name);
	if (IS_ERR(dev)) {
        ret = PTR_ERR(dev);
		kfree(d);
        goto fail;
    }

	try_module_get(THIS_MODULE); /* disallow rmmod until device is removed */
	dev->release = iphone_release_cb;
	strscpy(d->serial, DEFAULT_IPHONE_SERIAL, sizeof(d->serial));

	d->dev = dev;
	dev_set_drvdata(dev, d);

	/* add to device list */
	mutex_lock(&iphone_device_list_lock);
	list_add_tail(&d->list, &iphone_device_list);
	mutex_unlock(&iphone_device_list_lock);

	/* registers sysfs attrs */
	static struct device_attribute *iphone_attrs[] = {
		&dev_attr_serial,
		&dev_attr_udc,
		&dev_attr_bind,
		&dev_attr_status,
		NULL
	};

	for (struct device_attribute **attr = iphone_attrs; *attr; attr++) {
		ret = device_create_file(d->dev, *attr);
		if (ret) {
			pr_err("iPhone: failed to register sysfs attr '%s': %d",
				(*attr)->attr.name, ret);
			goto fail_unregister;
		}
	}

	pr_info("iPhone: registered device '%s'", name);
    mutex_unlock(&iphone_init_lock);
	return 0;
fail:
    mutex_unlock(&iphone_init_lock);
    return ret;
fail_unregister:
	device_unregister(dev);
    mutex_unlock(&iphone_init_lock);
    return ret;
}

int iphone_bind(struct device* device) 
{
	int ret = -ENODEV;

	device_lock(device);
	pr_info("iPhone: performing device bind\n");
	struct iphone_dev *data = dev_get_drvdata(device);
	if (!data) 
		goto fail;

	if (data->iphone)
	{
		ret = 0;
		goto fail;
	}

	struct iphone_dev_data *iphone = iphone_dev_alloc(device,
							  data->udc[0] ? data->udc : NULL,
							  data->serial);
	if (IS_ERR(iphone)) {
		ret = PTR_ERR(iphone);
		goto fail;
	}

	data->iphone = iphone; 
	device_unlock(device);
	return 0;

fail:
	device_unlock(device);
	return ret;
}

int iphone_unbind(struct device* device) 
{
	device_lock(device);
	pr_info("iPhone: performing device unbind\n");

	struct iphone_dev *data = dev_get_drvdata(device);
	if (data->iphone)
	{
		int ret = iphone_dev_free(data->iphone);
		if (ret) {
			pr_err("iPhone: failed to free device: %d\n", ret);
		}

		data->iphone = NULL;
	}

	device_unlock(device);
	return 0;
} 

int iphone_remove(char *name)
{
	struct iphone_dev *d, *tmp;
	int found = 0;

	mutex_lock(&iphone_init_lock);
	mutex_lock(&iphone_device_list_lock);
	list_for_each_entry_safe(d, tmp, &iphone_device_list, list) {
		if (strcmp(dev_name(d->dev), name) == 0) {
			list_del(&d->list);

			pr_info("iPhone: removing device '%s'\n", name);
			device_lock(d->dev);
			if (d->iphone) {
				int ret = iphone_dev_free(d->iphone);
				if (ret) {
					pr_err("iPhone: failed to free device: %d\n", ret);
				}

				d->iphone = NULL;
			}
			device_unlock(d->dev);
            device_unregister(d->dev);

            found = 1;
			break;
		}
	}
	mutex_unlock(&iphone_device_list_lock);
	mutex_unlock(&iphone_init_lock);

	if (!found) {
		pr_warn("iPhone: device '%s' not found\n", name);
		return -ENOENT;
	}

	return 0;
}

int iphone_class_register()
{
	int ret = 0;

    mutex_lock(&iphone_init_lock);
    iphone_class = class_create("iphone");

    if (IS_ERR(iphone_class)) {
		mutex_unlock(&iphone_init_lock);
        return PTR_ERR(iphone_class);
    }
    
	ret = class_create_file(iphone_class, &class_attr_create);
	if (ret)
		goto fail;

	ret = class_create_file(iphone_class, &class_attr_remove);
	if (ret)
		goto fail;

	mutex_unlock(&iphone_init_lock);
    return 0;
fail:
	mutex_unlock(&iphone_init_lock);
    return ret;
}

void iphone_class_unregister()
{
    mutex_lock(&iphone_init_lock);
    if (!iphone_class) {
        mutex_unlock(&iphone_init_lock);
        return;
    }

    struct iphone_dev *d, *tmp;

	mutex_lock(&iphone_device_list_lock);
	list_for_each_entry_safe(d, tmp, &iphone_device_list, list) {
		list_del(&d->list);
		pr_info("iPhone: removing device '%s'\n", dev_name(d->dev));
		device_lock(d->dev);
		if (d->iphone) {
			int ret = iphone_dev_free(d->iphone);
			if (ret) {
				pr_err("iPhone: failed to free device: %d\n", ret);
			}

			d->iphone = NULL;
		}
		device_unlock(d->dev);
		device_unregister(d->dev);
	}
	mutex_unlock(&iphone_device_list_lock);

	pr_debug("iPhone: all devices removed\n");
    class_destroy(iphone_class);
    iphone_class = NULL;

    mutex_unlock(&iphone_init_lock);
}

struct device* iphone_get_device(char *name) {
	mutex_lock(&iphone_init_lock);
	if (!iphone_class) {
		return NULL;
	}

    struct device* dev = class_find_device_by_name(iphone_class, name);
    if (!dev)
	{
		mutex_unlock(&iphone_init_lock);
        return NULL;
	}
	
	mutex_unlock(&iphone_init_lock);
	return dev;
}

void iphone_put_device(struct device* device) {
	put_device(device);  
}

int iphone_get_serial(struct device* device, char* out, int size) {
	int ret = -ENODEV;

	device_lock(device);
	struct iphone_dev *data = dev_get_drvdata(device);
	if (!data) 
		goto fail;

	strscpy(out, data->serial, size);
	device_unlock(device);
	return 0;

fail:
	device_unlock(device);
	return ret;
}

int iphone_get_udc(struct device* device, char* out, int size) {
	int ret = -ENODEV;

	device_lock(device);
	struct iphone_dev *data = dev_get_drvdata(device);
	if (!data) 
		goto fail;

	strscpy(out, data->udc, size);
	device_unlock(device);
	return 0;

fail:
	device_unlock(device);
	return ret;
}

int iphone_set_serial(struct device* device, char* serial) {
	int ret = -ENODEV;

	device_lock(device);
	struct iphone_dev *data = dev_get_drvdata(device);
	if (!data) 
		goto fail;
	if (data->iphone) 
	{
		ret = -EBUSY;
		goto fail; 
	}

	strscpy(data->serial, serial, sizeof(data->serial));
	device_unlock(device);
	return 0;

fail:
	device_unlock(device);
	return ret;
}

int iphone_set_udc(struct device* device, char* udc) {
	int ret = -ENODEV;

	device_lock(device);
	struct iphone_dev *data = dev_get_drvdata(device);
	if (!data) 
		goto fail;
	if (data->iphone) 
	{
		ret = -EBUSY;
		goto fail; 
	}

	strscpy(data->udc, udc, sizeof(data->udc));
	device_unlock(device);
	return 0;

fail:
	device_unlock(device);
	return ret;
}
