/* SPDX-License-Identifier: GPL-2.0 */
#ifndef IAP2_HID_H
#define IAP2_HID_H

#include <linux/types.h>

struct device;
struct iap2_hid;

/* USB HID SET_REPORT wValue report type, not Linux hid_report_type indexes. */
#define IAP2_HID_OUTPUT_REPORT 2

/* One complete Input report to be sent on the interrupt IN path by the arbiter.
 * Data includes Report ID, LCB and padding, and is borrowed until completion.
 */
struct iap2_hid_report {
    const u8 *data;
    size_t len;
    u8 report_id;
};

struct iap2_hid_config {
    /* Required descriptor determines report IDs and lengths. */
    const u8 *report_descriptor;
    size_t report_descriptor_len;
};

/* All calls run in sleepable context. The arbiter owns UDC/endpoint operations.
 * Preserve submit order. Never destroy the HID instance inside these callbacks.
 */
struct iap2_hid_io_ops {
    /* Queue promptly. 0 requires one report_complete, possibly inline;
     * negative errno means no completion. No access to report after completion.
     */
    int (*submit_report)(void *ctx, struct iap2_hid_report *report);
    /* Join this report AND its completion callback, even if already completed.
     * May complete it while cancelling; otherwise HID supplies -ECANCELED.
     */
    void (*cancel_report)(void *ctx, struct iap2_hid_report *report);
    /* Quiesce all ingress calls (set_report/closed), queued reports and their
     * callbacks before return. May complete pending reports; HID finishes any
     * remaining ones. Also safe before create returns on a registration error.
     */
    void (*stop)(void *ctx, int reason);
};

/* Creates a fresh numbered session and publishes its /dev/iap2-X node.
 * A valid report descriptor is required. Returns ERR_PTR on failure.
 * ops/ctx must survive until destroy returns. Start ingress only after success.
 */
struct iap2_hid *iap2_hid_create(struct device *parent,
                                const struct iap2_hid_config *config,
                                const struct iap2_hid_io_ops *ops, void *ctx);
/* Sleepable; exactly once, including after closed. Old FDs retain reason. */
void iap2_hid_destroy(struct iap2_hid *hid, int reason);

/* Atomic-safe ingress after the SET_REPORT data stage. Only Output reports are
 * supported; Feature/Input return -EOPNOTSUPP. Validate ID and report length.
 * The HID header is removed; payload, including padding, reaches core RX.
 * 0 accepts the whole report; -ENOSPC/-ENOMEM accepts nothing (arbiter retries
 * or rejects the control transfer). No SETUP decoding or UDC acknowledgement.
 * The arbiter serializes these calls in wire order (normally the EP0 path).
 */
int iap2_hid_set_report(struct iap2_hid *hid, u8 report_type, u8 report_id,
                        const void *data, size_t len);
/* Atomic-safe interrupt completion. Success means the whole report was sent.
 * Stop/cancel must join this function before freeing or reusing the report.
 */
void iap2_hid_report_complete(struct iap2_hid_report *report, int status);
/* Atomic-safe external disconnect/reset notification. First errno wins;
 * schedules teardown. The owner must still destroy the instance afterwards.
 */
void iap2_hid_closed(struct iap2_hid *hid, int reason);

#endif
