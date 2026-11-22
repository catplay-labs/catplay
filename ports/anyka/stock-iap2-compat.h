/* SPDX-License-Identifier: GPL-2.0 */
/* API adapters for the pinned Anyka Linux 4.4 SDK. */
#include <linux/device.h>
#include <linux/idr.h>
#include <linux/poll.h>
#include <linux/slab.h>

typedef unsigned int __poll_t;
#define EPOLLIN POLLIN
#define EPOLLOUT POLLOUT
#define EPOLLERR POLLERR
#define EPOLLHUP POLLHUP
#define EPOLLRDNORM POLLRDNORM
#define EPOLLWRNORM POLLWRNORM
#define ida_alloc(ida, flags) ida_simple_get((ida), 0, 0, (flags))
#define ida_free(ida, id) ida_simple_remove((ida), (id))

/* Saturate on overflow, as modern struct_size() does. */
static inline size_t catplay_struct_size(size_t base, size_t element, size_t count)
{
	if (count > (SIZE_MAX - base) / element)
		return SIZE_MAX;
	return base + element * count;
}
#define struct_size(ptr, member, count) \
	catplay_struct_size(sizeof(*(ptr)), sizeof((ptr)->member[0]), (count))
