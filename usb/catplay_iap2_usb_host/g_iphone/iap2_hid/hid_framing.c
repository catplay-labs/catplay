// SPDX-License-Identifier: GPL-2.0
#include "hid_framing.h"
#ifdef __KERNEL__
#include <linux/errno.h>
#include <linux/slab.h>
#include <linux/string.h>
#else
#include <errno.h>
#include <stdlib.h>
#include <string.h>
#endif

struct globals { u32 page, id, size, count; };
struct fields { u64 bits[2][256]; bool vendor[2][256]; };

int hid_reports_parse(struct hid_reports *reports, const u8 *desc, size_t len)
{
    struct globals g = {0}, stack[32];
    struct fields *fields;
    unsigned int depth = 0, d, id;
    size_t pos = 0, n, i;
    int ret = -EPROTO;
    bool found[2] = {false, false};
#ifdef __KERNEL__
    fields = kzalloc(sizeof(*fields), GFP_KERNEL);
#else
    fields = calloc(1, sizeof(*fields));
#endif
    if (!fields)
        return -ENOMEM;
    memset(reports, 0, sizeof(*reports));
    while (pos < len) {
        u8 prefix = desc[pos++];
        u32 value = 0;
        if (prefix == 0xfe) {
            if (len - pos < 2 || len - pos - 2 < desc[pos])
                goto out;
            pos += 2 + desc[pos];
            continue;
        }
        n = prefix & 3;
        if (n == 3) n = 4;
        if (n > len - pos) goto out;
        for (i = 0; i < n; i++) value |= (u32)desc[pos++] << (8 * i);
        switch (prefix & 0xfc) {
        case 0x04: g.page = value; break;
        case 0x74: g.size = value; break;
        case 0x84:
            if (!value || value > 255) goto out;
            g.id = value;
            break;
        case 0x94: g.count = value; break;
        case 0xa4:
            if (depth == 32) goto out;
            stack[depth++] = g;
            break;
        case 0xb4:
            if (!depth) goto out;
            g = stack[--depth];
            break;
        case 0x80: case 0x90: {
            u64 bits = (u64)g.size * g.count;
            d = (prefix & 0xfc) == 0x90;
            if (bits > 65534U * 8 || fields->bits[d][g.id] > 65534U * 8 - bits)
                goto out;
            fields->bits[d][g.id] += bits;
            fields->vendor[d][g.id] |= g.page >= 0xff00;
            break;
        }
        default: break;
        }
    }
    if (depth) goto out;
    for (d = 0; d < 2; d++) for (id = 0; id < 256; id++) {
        u64 bits = fields->bits[d][id];
        if (!fields->vendor[d][id]) continue;
        if (!id || bits % 8 || bits < 16 || bits > 65534U * 8) goto out;
        (d ? reports->output : reports->input)[id] = 1 + bits / 8;
        found[d] = true;
    }
    if (found[0] && found[1]) ret = 0;
out:
#ifdef __KERNEL__
    kfree(fields);
#else
    free(fields);
#endif
    if (ret) memset(reports, 0, sizeof(*reports));
    return ret;
}

int hid_report_encode(const u16 sizes[256], const u8 *data, size_t len,
                      size_t *offset, u8 *report, size_t capacity)
{
    unsigned int i, best = 0, largest = 0;
    size_t remaining, n;
    if (!len || *offset >= len) return -EINVAL;
    remaining = len - *offset;
    for (i = 1; i < 256; i++) {
        if (sizes[i] < 3) continue;
        if (sizes[i] >= sizes[largest]) largest = i;
        if (sizes[i] - 2U >= remaining && (!best || sizes[i] < sizes[best])) best = i;
    }
    if (!best) best = largest;
    if (!best) return -EINVAL;
    if (sizes[best] > capacity) return -EMSGSIZE;
    n = remaining < sizes[best] - 2U ? remaining : sizes[best] - 2U;
    memset(report, 0, sizes[best]);
    report[0] = best;
    report[1] = (*offset ? 1 : 0) | (remaining > n ? 2 : 0);
    memcpy(report + 2, data + *offset, n);
    *offset += n;
    return sizes[best];
}

int hid_report_decode(const u16 sizes[256], bool *continuing,
                      const u8 *report, size_t len, struct hid_fragment *fragment)
{
    if (!len || !sizes[report[0]] || sizes[report[0]] != len || len < 3 ||
        (report[1] & ~3) || ((report[1] & 1) && !*continuing)) {
        *continuing = false;
        return -EPROTO;
    }
    fragment->data = report + 2;
    fragment->len = len - 2;
    fragment->starts = !(report[1] & 1);
    fragment->ends = !(report[1] & 2);
    *continuing = !fragment->ends;
    return 0;
}

#ifdef __KERNEL__
#include <linux/module.h>
EXPORT_SYMBOL_GPL(hid_reports_parse);
#endif
