# Capability-aware screen keepalive

The transmitter uses the **car receiver's** `/info.keepAliveSendStatsAsBody`
flag. True selects opcode 5 with a 42-byte, encoded empty `bplist00` dictionary;
false or an omitted field retains the byte-identical, empty opcode-2 heartbeat.
The existing public session constructor also defaults to the legacy format.

The opcode-5 header is 128 bytes: `body_size = 42`, opcode 5, and
`params[14] = (0.0f32, 42.0f32)`, with all other fields zero. The static dictionary
avoids serializer failures and per-heartbeat serializer allocations. It contains
no guessed metric keys or values. This is a minimal activity heartbeat candidate,
**not a general implementation of screen statistics telemetry**.

## Evidence and limits

The supplied stock MHI2Q analysis of
`AirPlayReceiverSessionScreen_ProcessFrames` at VA `0x2B428–0x2B49C` identifies
a zero-body rejection before opcode dispatch: badframes increments and local
activity is not refreshed. Nonempty opcodes 2, 4, and 5 reach `0x2B7F8`, updating
the UpTicks activity timestamp without plist inspection. The supplied peer capture
explicitly advertises `keepAliveSendStatsAsBody=true`; the supplied iOS sender
analysis establishes opcode 5, a nonempty binary-plist dictionary, and the header
length float. Optional statistics-key semantics remain unknown.

The existing session-local one-second cadence and EventSleeper are retained,
independent of video traffic. A pending shutdown suppresses the heartbeat, and
backpressure or a failed queue write leaves its deadline unchanged. A
reconciliation that observes backpressure suppresses the heartbeat timer until
the TCP writable wake (or another notification) triggers a writable reconciliation;
frame notifications remain active, avoiding repeated zero-delay timer wakeups.
Successful enqueue advances the deadline; this does not assert delivery to the peer.
Proxy-drop and session-drop behavior is preserved.

Source tests cover header bytes, plist decoding and canonical encoding, legacy
bytes, deterministic due-time scheduling, disabled cadence, backpressure, failed
writes, pending shutdown, and existing drop behavior. These checks do not prove
in-car interoperability, cluster presentation, or resolve USB role-switch/power
behavior. No target observation is implied by the stock analysis or source tests.
