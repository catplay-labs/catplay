// SPDX-License-Identifier: GPL-2.0-only
/* Recover from a driver oops without waiting for potentially locked drivers. */
#include <linux/kallsyms.h>
#include <linux/module.h>

static bool restart;
module_param(restart, bool, 0400);
MODULE_PARM_DESC(restart, "Set to 1 only after userspace sync; immediately restarts the stock system");

static int __init catplay_restart_init(void)
{
    void (*restart_system)(void);
    if (!restart)
        return -EINVAL;
    restart_system = (void *)kallsyms_lookup_name("emergency_restart");
    if (!restart_system)
        return -ENOENT;
    pr_emerg("catplay_anyka_restart: immediate restart requested\n");
    restart_system();
    return -EIO;
}

module_init(catplay_restart_init);
MODULE_LICENSE("GPL");
MODULE_DESCRIPTION("Explicit recovery restart for stock Anyka development");
