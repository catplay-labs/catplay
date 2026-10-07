/* SPDX-License-Identifier: GPL-2.0 */
#ifndef IAP2_TRANSPORT_H
#define IAP2_TRANSPORT_H

#include <linux/types.h>

#define IAP2_MAX_XFER       65535U
#define IAP2_TX_INFLIGHT    3
#define IAP2_RX_BACKLOG_MAX 65535U

struct device;
struct iap2_session;

/* Owned by core; immutable while submitted. One request may span many reports. */
struct iap2_tx_req {
    unsigned int index;
    const u8 *data;
    size_t len;
};

/* All events may be called in atomic context. */
struct iap2_transport_events {
    /* Copies all bytes or none. Retain data on -ENOSPC/-ENOMEM until kick_rx. */
    int (*rx_data)(struct iap2_session *session, const void *data, size_t len);
    void (*rx_error)(struct iap2_session *session, int error);
    /* Exactly once after successful submit_tx, including cancellation. */
    void (*tx_done)(struct iap2_session *session, struct iap2_tx_req *req,
                    int status);
    /* First negative errno wins. Schedules stop; never waits for the backend. */
    void (*closed)(struct iap2_session *session, int reason);
};

/* Core serializes these calls in sleepable context, without holding spinlocks. */
struct iap2_transport_ops {
    int (*start)(void *ctx, const struct iap2_transport_events *events,
                 struct iap2_session *session);
    /* 0: accepted, eventually tx_done (possibly before return). Error: no event. */
    int (*submit_tx)(void *ctx, struct iap2_tx_req *req);
    /* Synchronous: request and its completion callback are quiescent on return. */
    void (*cancel_tx)(void *ctx, struct iap2_tx_req *req);
    void (*kick_rx)(void *ctx);
    /* Also called after a failed start. Complete accepted TX requests, stop all
     * IO/work, and join all callbacks before returning. Must not call unregister.
     */
    void (*stop)(void *ctx, int reason);
};

/* One live backend at a time; numbers are retained until the last session ref.
 * Caller owns ctx until unregister returns. On registration failure,
 * returns ERR_PTR and has already stopped any partially started backend.
 * The ops/ctx need not survive old open FDs after unregister.
 */
struct iap2_session *iap2_session_register(struct device *parent,
                            const struct iap2_transport_ops *ops, void *ctx);
/* Sleepable, exactly once per successful register, even after events->closed.
 * Publishes the terminal errno, then joins stop and all backend callbacks.
 * Call before releasing backend resources. Old FDs retain the terminal errno;
 * it does not imply that a replacement transport is ready.
 */
void iap2_session_unregister(struct iap2_session *session, int reason);
int iap2_session_devnode_path(struct iap2_session *session, char *buf, size_t size);
/* Publish the currently active transport under a g_iphone device. */
int iap2_session_link_active(struct device *owner);

#endif
