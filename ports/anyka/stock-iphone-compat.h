/* SPDX-License-Identifier: GPL-2.0 */
#include "iap2-compat.h"
#include <linux/string.h>
#define sysfs_emit(buf, fmt, ...) scnprintf(buf, PAGE_SIZE, fmt, ##__VA_ARGS__)
/* Callers only copy bounded, kernel-owned strings and check no return value. */
#define strscpy(dst, src, len) strlcpy(dst, src, len)
#define CLASS_ATTR_WO(name) CLASS_ATTR(name, 0200, NULL, name##_store)
static inline int catplay_match_name(struct device *dev, const void *name)
{
	return !strcmp(dev_name(dev), name);
}
static inline struct device *class_find_device_by_name(struct class *class, const char *name)
{
	return class_find_device(class, NULL, name, catplay_match_name);
}
static inline u32 get_unaligned_le24(const void *buf)
{
	const u8 *p = buf;
	return p[0] | ((u32)p[1] << 8) | ((u32)p[2] << 16);
}
extern bool catplay_diagnostic_identity;
