// SPDX-License-Identifier: GPL-2.0-only
/* Read-only compatibility check for the stock AK3918AV130 kernel. */
#include <linux/init.h>
#include <linux/kallsyms.h>
#include <linux/module.h>
#include <linux/utsname.h>

static int __init catplay_anyka_probe_init(void)
{
    unsigned long kexec_load = kallsyms_lookup_name("sys_kexec_load");
    unsigned long ni_syscall = kallsyms_lookup_name("sys_ni_syscall");

    pr_info("catplay_anyka_probe: loaded on %s; module_size=%zu\n",
            utsname()->release, sizeof(struct module));
    pr_info("catplay_anyka_probe: kexec_syscall=%s soft_restart=%s lookup=%s\n",
            kexec_load && ni_syscall && kexec_load != ni_syscall ? "available" : "unavailable",
            kallsyms_lookup_name("soft_restart") ? "present" : "absent",
            kallsyms_lookup_name("kernel_restart_prepare") ? "present" : "absent");
    return 0;
}

static void __exit catplay_anyka_probe_exit(void)
{
    pr_info("catplay_anyka_probe: unloaded\n");
}

module_init(catplay_anyka_probe_init);
module_exit(catplay_anyka_probe_exit);
MODULE_LICENSE("GPL");
MODULE_DESCRIPTION("Read-only CatPlay Anyka stock-kernel compatibility probe");
