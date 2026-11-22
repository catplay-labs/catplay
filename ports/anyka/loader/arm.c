// SPDX-License-Identifier: GPL-2.0-only
/* ARMv5 handoff using the SDK's standard kexec relocation assembly. */
#include <linux/kallsyms.h>
#include <linux/kexec.h>
#include <linux/mm.h>
#include <linux/of_fdt.h>
#include <linux/uaccess.h>
#include <asm/cacheflush.h>

extern void relocate_new_kernel(void);
extern const unsigned int relocate_new_kernel_size;
extern unsigned long kexec_start_address, kexec_indirection_page;
extern unsigned long kexec_mach_type, kexec_boot_atags;
static int (*region_is_memory)(phys_addr_t, phys_addr_t);
static void (*restart_cpu)(unsigned long);

int cp_arm_init(void)
{
    region_is_memory = (void *)kallsyms_lookup_name("memblock_is_region_memory");
    restart_cpu = (void *)kallsyms_lookup_name("soft_restart");
    return region_is_memory && restart_cpu ? 0 : -ENOENT;
}

int machine_kexec_prepare(struct kimage *image)
{
    unsigned int i, dtb_count = 0;
    bool entry_found = false;
    unsigned long total = 0, dtb_address = 0;
    for (i = 0; i < image->nr_segments; i++) {
        struct kexec_segment *s = &image->segment[i];
        __be32 magic;
        /* First bring-up only: use the verified lower 60 MiB of RAM. */
        if (s->bufsz < 4 || s->mem < 0x80008000UL || s->mem >= 0x83c00000UL ||
            !s->memsz || s->memsz > 0x83c00000UL - s->mem ||
            s->bufsz > s->memsz || !region_is_memory(s->mem, s->memsz))
            return -EINVAL;
        total += s->memsz;
        if (total > 16 * 1024 * 1024UL)
            return -E2BIG;
        if (get_user(magic, (__be32 __user *)s->buf))
            return -EFAULT;
        if (be32_to_cpu(magic) == OF_DT_HEADER) {
            __be32 size;
            if (s->bufsz < 40 || get_user(size, (__be32 __user *)s->buf + 1))
                return -EINVAL;
            if (be32_to_cpu(size) < 40 || be32_to_cpu(size) > s->bufsz)
                return -EINVAL;
            dtb_address = s->mem;
            dtb_count++;
        } else if (image->start == s->mem) {
            /* ARM zImage magic is at offset 0x24, in little endian. */
            u32 zmagic;
            if (s->bufsz < 0x30 ||
                get_user(zmagic, (u32 __user *)((char __user *)s->buf + 0x24)))
                return -EINVAL;
            if (zmagic != 0x016f2818)
                return -ENOEXEC;
            entry_found = true;
        }
    }
    if (!entry_found || dtb_count != 1)
        return -EINVAL;
    image->image_loader_data = (void *)dtb_address;
    return 0;
}

void machine_kexec_cleanup(struct kimage *image) {}

static void put_reloc_value(void *buffer, unsigned long *symbol, unsigned long value)
{
    unsigned long offset = (char *)symbol - (char *)relocate_new_kernel;
    *(unsigned long *)((char *)buffer + offset) = value;
}

void machine_kexec(struct kimage *image)
{
    void *buffer = page_address(image->control_code_page);
    unsigned long physical = page_to_pfn(image->control_code_page) << PAGE_SHIFT;
    BUG_ON(relocate_new_kernel_size > PAGE_SIZE);
    memcpy(buffer, (void *)relocate_new_kernel, relocate_new_kernel_size);
    put_reloc_value(buffer, &kexec_start_address, image->start);
    put_reloc_value(buffer, &kexec_indirection_page, image->head & PAGE_MASK);
    put_reloc_value(buffer, &kexec_mach_type, ~0UL);
    put_reloc_value(buffer, &kexec_boot_atags, (unsigned long)image->image_loader_data);
    flush_icache_range((unsigned long)buffer,
                       (unsigned long)buffer + relocate_new_kernel_size);
    restart_cpu(physical);
}
