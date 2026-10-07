// SPDX-License-Identifier: GPL-2.0
#include <linux/kernel.h>
#include <linux/module.h>
#include "libcomposite/composite.h"
#include "iphone_dev.h"
#include "iphone_class.h"
#include "iphone_mod.h"

static int __init iphone_init(void)
{
    int ret;
    ret = iphone_class_register();
	if (ret) {
		return ret;
	}

    if (device_name && device_name[0]) {
        ret = iphone_add(device_name);
        if (ret) {
            pr_err("iPhone: failed to create default device: %d\n", ret);
            iphone_class_unregister();
            return ret;
        }
        struct device* iphone = iphone_get_device(device_name);
        if (!iphone) {
            pr_err("iPhone: failed to create default device\n");
            iphone_class_unregister();
            return -ENODEV;
        }
        
        ret = iphone_set_serial(iphone, iphone_serial);
        if (ret) {
            pr_err("iPhone: failed to override serial\n");
            iphone_put_device(iphone);
            iphone_class_unregister();
            return ret;
        }

        if (udc_name && udc_name[0]) {
            ret = iphone_set_udc(iphone, udc_name);
            if (ret) {
                pr_err("iPhone: failed to override UDC\n");
                iphone_put_device(iphone);
                iphone_class_unregister();
                return ret;
            }
        }

        ret = iphone_bind(iphone);
        if (ret) {
            pr_err("iPhone: failed to bind default device: %d\n", ret);
            iphone_put_device(iphone);
            iphone_class_unregister();
            return ret;
        }
        iphone_put_device(iphone);

    }

	pr_info("iPhone: kernel module loaded\n");
    return 0;
}

static void __exit iphone_exit(void)
{
    iphone_class_unregister();
    pr_info("iPhone: kernel module unloaded\n");
}

module_init(iphone_init);
module_exit(iphone_exit);

MODULE_AUTHOR("CatPlay");
MODULE_DESCRIPTION("iPhone role-switch gadget that works with CarPlay headunits that perform extremely deep USB heuristics");
MODULE_LICENSE("GPL");
