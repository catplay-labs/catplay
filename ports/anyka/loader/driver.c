// SPDX-License-Identifier: GPL-2.0-only
/* Normal kexec only. Image staging and execution are separate ioctls. */
#include <linux/capability.h>
#include <linux/fs.h>
#include <linux/kallsyms.h>
#include <linux/kexec.h>
#include <linux/miscdevice.h>
#include <linux/module.h>
#include <linux/mutex.h>
#include <linux/reboot.h>
#include <linux/uaccess.h>
#include <linux/syscalls.h>
#include "kexec_internal.h"

#define CP_LOAD 0x43500001
#define CP_EXEC 0x43500002
#define CP_UNLOAD 0x43500003
#define CP_STATUS 0x43500004

struct cp_status { unsigned long entry, dtb, nr_segments; };

struct cp_load {
    unsigned long entry;
    unsigned long nr_segments;
    struct kexec_segment __user *segments;
};

static void (*restart_prepare)(char *);
static void (*shutdown_machine)(void);
static DEFINE_MUTEX(cp_mutex);
int cp_arm_init(void);
void crash_map_reserved_pages(void) {}
void crash_unmap_reserved_pages(void) {}

static long cp_ioctl(struct file *file, unsigned int cmd, unsigned long arg)
{
    struct cp_load request;
    long result = -EINVAL;
    if (!capable(CAP_SYS_BOOT))
        return -EPERM;
    if (mutex_lock_interruptible(&cp_mutex))
        return -ERESTARTSYS;
    switch (cmd) {
    case CP_LOAD:
        if (copy_from_user(&request, (void __user *)arg, sizeof(request))) {
            result = -EFAULT;
            break;
        }
        if (request.nr_segments < 2 || request.nr_segments > 3)
            break;
        /* No crash kernels or preserve-context mode can be requested. */
        result = sys_kexec_load(request.entry, request.nr_segments,
                               request.segments, KEXEC_ARCH_ARM);
        if (!result)
            pr_info("catplay_anyka_kexec: staged %lu segments; entry=%08lx\n",
                    request.nr_segments, request.entry);
        break;
    case CP_UNLOAD:
        result = sys_kexec_load(0, 0, NULL, KEXEC_ARCH_ARM);
        break;
    case CP_STATUS: {
        struct cp_status status = {0};
        if (kexec_image) {
            status.entry = kexec_image->start;
            status.dtb = (unsigned long)kexec_image->image_loader_data;
            status.nr_segments = kexec_image->nr_segments;
        }
        result = copy_to_user((void __user *)arg, &status, sizeof(status)) ? -EFAULT : 0;
        break;
    }
    case CP_EXEC:
        if (arg || !kexec_image)
            break;
        /* Freeze the staged image before shutting down devices. */
        mutex_lock(&kexec_mutex);
        pr_emerg("catplay_anyka_kexec: executing staged RAM image\n");
        restart_prepare(NULL);
        shutdown_machine();
        machine_kexec(kexec_image);
        /* A successful transition never returns. */
        result = -EIO;
        mutex_unlock(&kexec_mutex);
        break;
    }
    mutex_unlock(&cp_mutex);
    return result;
}

static const struct file_operations cp_fops = {
    .owner = THIS_MODULE,
    .unlocked_ioctl = cp_ioctl,
};
static struct miscdevice cp_device = {
    .minor = MISC_DYNAMIC_MINOR,
    .name = "catplay_kexec",
    .fops = &cp_fops,
    .mode = 0600,
};
static int __init cp_init(void)
{
    int error = cp_arm_init();
    if (error)
        return error;
    restart_prepare = (void *)kallsyms_lookup_name("kernel_restart_prepare");
    shutdown_machine = (void *)kallsyms_lookup_name("machine_shutdown");
    if (!restart_prepare || !shutdown_machine)
        return -ENOENT;
    error = misc_register(&cp_device);
    if (!error)
        pr_info("catplay_anyka_kexec: ready; no image loaded\n");
    return error;
}
static void __exit cp_exit(void)
{
    misc_deregister(&cp_device);
    kimage_free(kexec_image);
    kexec_image = NULL;
    pr_info("catplay_anyka_kexec: unloaded\n");
}
module_init(cp_init);
module_exit(cp_exit);
MODULE_LICENSE("GPL");
MODULE_DESCRIPTION("CatPlay AK3918AV130 RAM-only kexec loader");
