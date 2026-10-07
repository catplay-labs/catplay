/* SPDX-License-Identifier: GPL-2.0 */
#ifndef CATPLAY_HID_FRAMING_H
#define CATPLAY_HID_FRAMING_H
#ifdef __KERNEL__
#include <linux/types.h>
#else
#include <stdint.h>
#include <stdbool.h>
#include <stddef.h>
typedef uint8_t u8;
typedef uint16_t u16;
typedef uint32_t u32;
typedef uint64_t u64;
#endif

/* Device descriptor directions: device transmits input, receives output. */
struct hid_reports { u16 input[256]; u16 output[256]; };
struct hid_fragment { const u8 *data; size_t len; bool starts, ends; };
int hid_reports_parse(struct hid_reports *reports, const u8 *desc, size_t len);
int hid_report_encode(const u16 sizes[256], const u8 *data, size_t len,
                      size_t *offset, u8 *report, size_t capacity);
int hid_report_decode(const u16 sizes[256], bool *continuing,
                      const u8 *report, size_t len, struct hid_fragment *fragment);
#endif
