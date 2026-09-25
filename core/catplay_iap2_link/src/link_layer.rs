use alloc::{boxed::Box, collections::BTreeMap, vec::Vec};
use catplay_util::{ModSeq, ModSeq8};
use core::{cmp::Ordering, time::Duration};
use log::{debug, trace};

#[cfg(test)]
#[path = "link_layer_test.rs"]
mod tests;

use crate::{
    EAKPayload, LSPPayload, LinkSession, PayloadDecodable, PayloadSession, SessionType,
    clock::{Clock, ClockInstant},
    files::{FileSession, FileTransferEvent},
    negotiate::{LSPNegotiator, Negotiation, NegotiationTx, SimpleLSPNegotiator},
    packet::*,
};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum LinkStatus {
    Detecting,
    Negotiating,
    Writable,
    Unwritable,
    Recovery,

    Error(LinkError),
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum LinkError {
    Reset,
    RetransmissionsExceeded,
    LinkNegotiationRoundsExceeded,
    Eof,
    RecvMaxLenViolation { received: usize, limit: usize },
    TransmitMaxLenViolation { transmitting: usize, limit: usize },
}

/// Represents instructions that should be processed by external transport(socket) layer
/// in response to processed requests/inputs.
#[derive(Debug)]
pub enum LinkEvent {
    /// A new CSM message was decoded.
    ReadCsm(Vec<u8>),
    /// A packet should be forwarded to transport.
    Write(PacketOrDetect),
    /// Status has changed.
    Status(LinkStatus),
    /// File transfer event.
    FileTransfer(u8, FileTransferEvent),
}

pub struct LinkLayer {
    /// We are either a server(host) or an accessory.
    server: bool,
    status: LinkStatus,
    callback: Box<dyn Fn(LinkEvent) + Send>,
    lsp_negotiator: Box<dyn LSPNegotiator>,
    negotiation: Negotiation,
    local_lsp: LSPPayload,
    peer_lsp: LSPPayload,
    peer_seq: Option<ModSeq8>,
    own_seq: ModSeq8,
    peer_ack: Option<ModSeq8>,

    /// Injectable system clock (for easier testing)
    clock: Box<dyn Clock>,

    logger: &'static str,

    /// Retransmission control
    out_of_order: BTreeMap<u8, Packet>,
    retransmit_map: BTreeMap<u8, RetransmitEntry>,
    eak: EakState,

    /// Retransmission control for SYN(client),
    syn_ack_received: bool,
    syn_transmitted: Option<ClockInstant>,
    detect_last_transmit: Option<ClockInstant>,

    // Sessions,
    files: FileSession,
    csm: PayloadSession,
    ea: PayloadSession,

    pending_ack: PendingAck,

    #[cfg(feature = "tracing")]
    tracer: catplay_tracing::tracer::SessionTracer,
}

#[derive(Default, Debug)]
struct PendingAck {
    unacked_packets: usize,
    deadline: Option<ClockInstant>,
    out_of_order_ids: Vec<u8>,
    needs_detect_acknowledge: bool,
    force: bool,
}

impl PendingAck {
    pub fn reset(&mut self) {
        *self = Self::default()
    }

    pub fn add_out_of_order_seq(&mut self, seq: u8) {
        if !self.out_of_order_ids.contains(&seq) {
            self.out_of_order_ids.push(seq);
        }
    }
}

#[derive(Default, Debug)]
struct EakState {
    missing_psns: Vec<u8>,
}

impl EakState {
    fn is_active(&self) -> bool {
        !self.missing_psns.is_empty()
    }

    fn begin_recovery(&mut self, missing_psns: &[u8]) {
        self.missing_psns.clear();
        self.missing_psns.extend_from_slice(missing_psns);
    }

    fn clear(&mut self) {
        self.missing_psns.clear();
    }

    fn next_missing_psn(&self) -> Option<u8> {
        self.missing_psns.first().copied()
    }
}
#[derive(Debug)]
pub(crate) struct RetransmitEntry {
    pub packet: Packet,
    pub _sent_at: ClockInstant,
    pub retry_count: u8,
    pub timeout_deadline: ClockInstant,
}

impl LinkLayer {
    const DETECT_RETRANSMIT: Duration = Duration::from_millis(200);

    pub fn files(&mut self) -> &mut FileSession {
        &mut self.files
    }

    pub fn ea(&mut self) -> &mut PayloadSession {
        &mut self.ea
    }

    pub fn csm(&mut self) -> &mut PayloadSession {
        &mut self.csm
    }

    pub fn status(&self) -> &LinkStatus {
        &self.status
    }

    fn send_event(&mut self, ev: LinkEvent) {
        if let LinkEvent::Write(p) = &ev {
            self.trace_packet(true, p);
        }

        #[cfg(debug_assertions)]
        trace!(target: self.logger, "Event <- {:?}", &ev);
        (self.callback)(ev)
    }

    fn trace_packet(&mut self, outgoing: bool, packet: &PacketOrDetect) {
        #[cfg(feature = "tracing")]
        {
            let packet = packet.clone();

            if outgoing {
                catplay_tracing::strace!(self.tracer, "Link -> {}", packet);
            } else {
                catplay_tracing::strace!(self.tracer, "Link <- {}", packet);
            }
        }

        #[cfg(debug_assertions)]
        {
            if outgoing {
                debug!(target: self.logger, "Link -> {}", packet);
            } else {
                debug!(target: self.logger, "Link <- {}", packet);
            }
        }
    }

    fn change_state(&mut self, state: LinkStatus) {
        self.status = state;
        (self.callback)(LinkEvent::Status(state));
        debug!(target: self.logger, "Changing state to {state:?}");
    }

    fn refresh_data_status(&mut self) {
        if !matches!(self.status, LinkStatus::Recovery | LinkStatus::Writable | LinkStatus::Unwritable) {
            return;
        }

        let next = if self.eak.is_active() {
            LinkStatus::Recovery
        } else if self.is_window_writable() {
            LinkStatus::Writable
        } else {
            LinkStatus::Unwritable
        };

        if self.status != next {
            self.change_state(next);
        }
    }

    #[allow(dead_code)]
    pub fn with_clock_and_lsp_negotiator(
        server: bool,
        clock: Box<dyn Clock>,
        lsp_negotiator: Box<dyn LSPNegotiator>,
        callback: impl Fn(LinkEvent) + Send + 'static,
    ) -> Self {
        let negotiation = Negotiation::new(lsp_negotiator.start());
        let mut me = Self {
            server,
            lsp_negotiator,
            negotiation,
            local_lsp: LSPPayload::negotiating(),
            peer_lsp: LSPPayload::negotiating(),
            peer_seq: None,
            peer_ack: None,

            own_seq: match server {
                true => ModSeq(0x10),
                false => ModSeq(0x20),
            },
            logger: match server {
                true => "catplay_iap2_link::LinkLayer[server]",
                false => "catplay_iap2_link::LinkLayer[client]",
            },
            clock,
            status: LinkStatus::Detecting,

            out_of_order: BTreeMap::new(),
            retransmit_map: BTreeMap::new(),
            eak: EakState::default(),
            syn_ack_received: false,
            syn_transmitted: None,
            detect_last_transmit: None,

            callback: Box::new(callback),

            files: FileSession::new(server),
            ea: PayloadSession::new(),
            csm: PayloadSession::new(),
            pending_ack: PendingAck::default(),
            #[cfg(feature = "tracing")]
            tracer: catplay_tracing::tracer::SessionTracer::new("iap2_trace"),
        };
        me.reconcile();
        me
    }

    #[allow(dead_code)]
    pub fn with_clock(server: bool, clock: Box<dyn Clock>, callback: impl Fn(LinkEvent) + Send + 'static) -> Self {
        // TODO: unhardcode choice of LSPPayload
        Self::with_clock_and_lsp_negotiator(server, clock, Box::new(SimpleLSPNegotiator::new(LSPPayload::bluetooth())), callback)
    }

    #[cfg(feature = "std")]
    pub fn new(server: bool, callback: impl Fn(LinkEvent) + Send + 'static) -> Self {
        use crate::clock::SystemClock;
        Self::with_clock(server, Box::new(SystemClock), callback)
    }

    #[cfg(feature = "std")]
    pub fn new_with_lsp_negotiator(
        server: bool,
        lsp_negotiator: Box<dyn LSPNegotiator>,
        callback: impl Fn(LinkEvent) + Send + 'static,
    ) -> Self {
        use crate::clock::SystemClock;
        Self::with_clock_and_lsp_negotiator(server, Box::new(SystemClock), lsp_negotiator, callback)
    }

    fn is_window_writable(&self) -> bool {
        let Some(peer_ack) = self.peer_ack else { return true }; /* when sending SYN-ACK */

        let window = self.peer_lsp.max_outgoing;
        let outstanding = self.own_seq - peer_ack;

        trace!(
            target: self.logger,
            "Writable check: own_seq={} peer_ack={} outstanding={} max_outgoing={} result={}",
            self.own_seq, peer_ack, outstanding, window, outstanding.value() < window
        );

        outstanding.value() <= window
    }

    fn take_own_seq(&mut self) -> ModSeq8 {
        let seq = self.own_seq;
        self.own_seq = self.own_seq + 1;
        seq
    }

    fn prepare_ackable(&mut self, packet: Packet) -> Option<PacketOrDetect> {
        if !packet.is_ackable() {
            return Some(packet.into());
        }

        let now = self.clock.now();
        let timeout = now + Duration::from_millis(self.local_lsp.retransmission_timeout.into());

        self.retransmit_map.insert(
            packet.header.seq.0,
            RetransmitEntry {
                packet: packet.clone(),
                _sent_at: now,
                retry_count: 0,
                timeout_deadline: timeout,
            },
        );

        Some(packet.into())
    }

    fn counter_offer(&self, offer: &LSPPayload, negotiation_counter: u32) -> Option<LSPPayload> {
        let has_control = offer
            .sessions
            .iter()
            .any(|f| f.session_type == SessionType::Control);
        let counter = if has_control {
            self.lsp_negotiator.counter(offer, negotiation_counter)
        } else {
            Some(self.lsp_negotiator.start())
        };
        debug!(
            target: self.logger,
            "Received LSP[{negotiation_counter}]: {:?} control_session={} counter={:?}",
            offer,
            has_control,
            counter
        );
        counter
    }

    fn find_session_of_type(&self, session_type: SessionType) -> Option<u8> {
        self.local_lsp
            .sessions
            .iter()
            .find(|s| s.session_type == session_type)
            .map(|s| s.id)
    }

    fn next_retransmit_seq(&self, now: ClockInstant) -> Option<u8> {
        if let Some(seq) = self.eak.next_missing_psn() {
            return self.retransmit_map.contains_key(&seq).then_some(seq);
        }

        self.retransmit_map
            .iter()
            .filter(|(_, entry)| now >= entry.timeout_deadline)
            .min_by_key(|(_, entry)| entry.timeout_deadline)
            .map(|(&seq, _)| seq)
    }

    fn retransmit_one(&mut self, seq: u8) -> Option<PacketOrDetect> {
        let lsp = &self.local_lsp;
        let now = self.clock.now();

        let entry = self.retransmit_map.get_mut(&seq)?;

        if entry.retry_count >= lsp.max_retransmissions {
            debug!("Packet {:?} exceeded max retransmissions, closing", seq);
            // TODO: iPhone would send a RST here instead
            self.change_state(LinkStatus::Error(LinkError::RetransmissionsExceeded));
            return None;
        }

        // Special case - we don't care about initialized peer_seq, if we are retransmitting a SYN packet.
        if !entry.packet.header.control.is_syn() {
            let peer_seq = self.peer_seq?;
            entry.packet.header.ack = peer_seq;
        }
        debug!(
            target: self.logger,
            "Retransmitting packet {} x{}: {}",
            seq,
            entry.retry_count,
            entry.packet
        );

        let packet = entry.packet.clone();
        entry.retry_count += 1;
        entry.timeout_deadline = now + Duration::from_millis(lsp.retransmission_timeout.into());
        Some(packet.into())
    }

    fn take_pending_negotiation_transmit(&mut self) -> Option<PacketOrDetect> {
        let tx = self.negotiation.take_pending_tx()?;
        let peer_seq = self.peer_seq?;

        match tx {
            NegotiationTx::SynAck(lsp) => {
                let packet = Packet::new_syn_ack(self.take_own_seq(), peer_seq, &lsp);
                let next = self.prepare_ackable(packet);
                self.negotiation.transition_start();
                next
            }
            NegotiationTx::FinalAck => {
                let packet = Packet::new_ack(self.own_seq - 1, peer_seq, LSPPayload::SESSION_ID_CONTROL, None);
                self.negotiation.transition_accepted();
                self.change_state(LinkStatus::Writable);
                Some(packet.into())
            }
        }
    }

    /// Reconcile incoming data into [LinkEvent]s and transmit all pending outgoing packets without any limit.
    pub fn reconcile_all(&mut self) {
        self.reconcile();
        self.transmit_all();
    }

    /// Reconcile incoming data into [LinkEvent]s.
    pub fn reconcile(&mut self) {
        self.drain_events();
    }

    /// Transmit all pending outgoing packets in form of [LinkEvent::Write].
    pub fn transmit_all(&mut self) {
        while let Some(next) = self.next_transmit() {
            self.send_event(LinkEvent::Write(next));
            self.refresh_data_status();
        }
    }

    /// Transmit one outgoing packet in form of [LinkEvent::Write], if any, by internal priority.
    ///
    /// The goal of this API is to allow easy backpressure management in line with the transport.
    pub fn transmit_once(&mut self) -> bool {
        if let Some(next) = self.next_transmit() {
            self.send_event(LinkEvent::Write(next));
            self.refresh_data_status();
            true
        } else {
            false
        }
    }

    fn next_transmit(&mut self) -> Option<PacketOrDetect> {
        // Decide a single next frame to transmit by priority
        // 0) Acknowledge peer detect
        // 1) Retransmit DETECT
        // 2) Pending negotiation response + SYN retransmissions
        // 3) EAK
        // 4) Pending retransmission, including EAK-triggered ones (closest to deadline)
        // 5) Next session (CSM/File/EA) payload, if writable
        // 6) Empty ACK (if no session payload has cleared the "pending" status and deadline conditions are met OR we received a packet with duplicate seq)

        let now = self.clock.now();

        // 0) Acknowledge peer DETECT
        if self.status == LinkStatus::Detecting && self.server && self.pending_ack.needs_detect_acknowledge {
            self.pending_ack.needs_detect_acknowledge = false;
            return Some(PacketOrDetect::Detect);
        }

        // 1) Retransmit DETECT
        if self.status == LinkStatus::Detecting && !self.server {
            const DETECT_RETRANSMIT: Duration = Duration::from_millis(200);

            let should_send_detect = self.detect_last_transmit.is_none()
                || self
                    .detect_last_transmit
                    .is_some_and(|last_detect| now - last_detect > DETECT_RETRANSMIT);

            if should_send_detect {
                debug!(target: self.logger, "Trying DETECT prev={:?}", self.detect_last_transmit);
                self.detect_last_transmit.replace(now);
                return Some(PacketOrDetect::Detect);
            }

            return None;
        }

        // 2) Pending negotiation response + SYN retransmissions
        if self.status == LinkStatus::Negotiating {
            if let Some(next) = self.take_pending_negotiation_transmit() {
                return Some(next);
            }

            // Retransmit SYN
            if !self.server && self.syn_transmitted.is_none() {
                // Standard retransmission logic will apply to this created SYN packet (based on pre-negotiation LSP with a 1000ms timeout)
                debug!(target: self.logger, "Scheduling first SYN packet");
                self.syn_transmitted.replace(now);
                let syn = Packet::new_syn(self.take_own_seq(), self.negotiation.local_lsp());
                return self.prepare_ackable(syn);
            }

            if let Some(seq) = self.next_retransmit_seq(now) {
                return self.retransmit_one(seq);
            }

            return None;
        }

        if !matches!(self.status, LinkStatus::Recovery | LinkStatus::Writable | LinkStatus::Unwritable) {
            return None;
        }

        // 3) EAK
        if !self.pending_ack.out_of_order_ids.is_empty() {
            // TODO - send EAK
            self.pending_ack.out_of_order_ids.clear();
            // let missing = self.peer_seq + ModSeq(1);
            // let ack_packet = Packet::new_eak(self.own_seq, self.peer_seq, LSPPayload::SESSION_ID_CONTROL, vec![missing.0]);
            // debug!("Sending EAK: ACK={} PSNs={:?}", ack_packet.header.ack, ack_packet.payload);
            // return None;
        }

        // 4) Pending retransmission, including EAK-triggered ones (closest to deadline)
        if let Some(seq) = self.next_retransmit_seq(now) {
            return self.retransmit_one(seq);
        }

        // 5) Next session (CSM/File/EA) payload, if writable
        if self.is_window_writable()
            && self.status == LinkStatus::Writable
            && let Some(v) = self.next_payload()
        {
            let peer_seq = self.peer_seq?;
            let session = self
                .find_session_of_type(v.0)
                .expect("missing session but wants to transmit");
            let packet = Packet::new_ack(self.take_own_seq(), peer_seq, session, Some(v.1));

            if packet.size() > self.peer_lsp.max_len as _ {
                // Either an optimistically scheduled CSM/EA packet that violates peer's limit, or an improperly scheduled file transfer payload.
                self.change_state(LinkStatus::Error(LinkError::TransmitMaxLenViolation {
                    transmitting: packet.size(),
                    limit: self.peer_lsp.max_len as _,
                }));
                return None;
            }
            let next = self.prepare_ackable(packet);
            self.pending_ack.reset();
            return next;
        }

        // 6) Empty ACK (if no session payload has cleared the "pending" status and deadline conditions are met OR we received a packet with duplicate seq)
        // Only consider sending empty ACK if nothing was sent with a payload
        let deadline_elapsed = self
            .pending_ack
            .deadline
            .is_some_and(|deadline| self.clock.now() >= deadline);
        let ack_budget_exceeded = match self.local_lsp.max_ack {
            0 => self.pending_ack.unacked_packets >= 1,
            v => self.pending_ack.unacked_packets >= v as _,
        };
        let forced = self.pending_ack.force;

        if deadline_elapsed || ack_budget_exceeded || forced {
            let peer_seq = self.peer_seq?;
            let packet = Packet::new_ack(self.own_seq - 1, peer_seq, LSPPayload::SESSION_ID_CONTROL, None);
            let next = self.prepare_ackable(packet);
            debug!(target: self.logger, "Fallback: forced empty ACK now {:?}", self.pending_ack);
            self.pending_ack.reset();
            return next;
        }

        // Nothing to transmit
        None
    }

    #[allow(unused)]
    fn has_next_payload(&mut self) -> bool {
        self.csm.has_tx_pending() || self.files.has_tx_pending() || self.ea.has_tx_pending()
    }

    #[allow(clippy::manual_map)]
    fn next_payload(&mut self) -> Option<(SessionType, Vec<u8>)> {
        let max_payload_len = (self.peer_lsp.max_len as usize).saturating_sub(Packet::PACKET_OVERHEAD);

        if let Some(v) = self.csm.dequeue_tx(max_payload_len) {
            Some((SessionType::Control, v))
        } else if let Some(v) = self.files.dequeue_tx(max_payload_len) {
            Some((SessionType::FileTransfer, v))
        } else if let Some(v) = self.ea.dequeue_tx(max_payload_len) {
            Some((SessionType::ExternalAccessory, v))
        } else {
            None
        }
    }

    fn verify_neg_seq_ack(&self, packet: &Packet) -> bool {
        let Some(peer_seq) = self.peer_seq else { return false };
        debug!(target: self.logger, "verify_seq[host {}] {} {} verify_ack {} {}", self.server, packet.header.seq, peer_seq+1, packet.header.ack, self.own_seq-1);
        packet.header.seq == peer_seq + 1 && packet.header.ack == self.own_seq - 1
    }

    fn advance_peer_seq_from_ackable(&mut self, packet: &Packet) {
        if !packet.is_ackable() {
            return;
        }

        debug!(target: self.logger, "advance_peer_seq to {}", packet.header.seq);
        self.peer_seq = Some(packet.header.seq);
    }

    fn on_recv_neg_accessory(&mut self, ctl: &crate::LinkControl, packet: &Packet) {
        // We are an accessory
        // The expected packet here is always SYN+ACK with host's preferred LSP; up to us to
        // respond with ACK(accepted) or SYN+ACK(with our new preferred LSP; continue negotiating)

        // Ignore the packet if not SYN+ACK or seq mismatch
        if !ctl.is_syn_ack() || (self.peer_seq.is_some() && !self.verify_neg_seq_ack(packet)) {
            debug!(target: self.logger, "Accessory dropped invalid seq for link neg [known {}] {:?} vs {}", self.peer_seq.is_some(), self.peer_seq, packet.header.seq);
            return;
        }

        self.syn_ack_received = true;
        self.advance_peer_seq_from_ackable(packet);

        // Check if the host's LSP is acceptable
        // If no: send SYN+ACK with counter offer
        // If yes: send empty ACK and finish
        if let Some(lsp) = LSPPayload::from_packet(packet) {
            if let Some(counter) = self.counter_offer(&lsp, self.negotiation.negotiation_counter()) {
                debug!("Countering with LSP: {counter:?}");
                // Unacceptable, send the counter embedded in SYN+ACK and continue negotiation
                if self.negotiation.increment_counter() >= 10 {
                    self.change_state(LinkStatus::Error(LinkError::LinkNegotiationRoundsExceeded));
                    return;
                }

                self.negotiation.set_local_lsp(counter);
                self.negotiation.transition_offering();
            } else {
                // Acceptable, send ACK and finish negotiations after it is actually transmitted.
                let local_lsp = self.negotiation.local_lsp().clone();
                self.negotiation.note_lsp_accepted_by_peer(local_lsp);
                self.negotiation.note_lsp_accepted_by_us(lsp.clone());
                self.peer_lsp = lsp.clone();
                self.local_lsp = self.negotiation.local_lsp().clone();
                debug!(
                    target: self.logger,
                    "Accessory accepted peer LSP {:?}; local LSP {:?}; waiting to transmit final ACK",
                    self.peer_lsp,
                    self.local_lsp
                );
                self.negotiation.transition_accepting();
            }
        }
    }

    fn on_recv_neg_host(&mut self, ctl: &crate::LinkControl, packet: &Packet) {
        // We are a host, always expect SYN+ACK containing a LSP payload or an empty ACK
        let (is_syn, is_ack, is_syn_ack) = (ctl.is_syn(), ctl.is_ack(), ctl.is_syn_ack());
        if !(is_syn || is_ack || is_syn_ack) {
            return;
        }

        // Ignore the packet if seq mismatch unless it's the first SYN packet
        if (is_syn_ack || is_ack) && self.peer_seq.is_some() && !self.verify_neg_seq_ack(packet) {
            debug!(target: self.logger, "Host observed invalid seq for link neg [known {}] {:?} vs {}", self.peer_seq.is_some(), self.peer_seq, packet.header.seq);
            /* Some buggy clients fail this check, so don't enforce! */ /* return; */
        }

        self.advance_peer_seq_from_ackable(packet);

        if is_ack {
            // Our last offer was accepted by the accessory. Negotations are now finished with no response required.
            let local_lsp = self.negotiation.local_lsp().clone();
            self.negotiation.note_lsp_accepted_by_peer(local_lsp);
            self.local_lsp = self
                .negotiation
                .peer_accepted_lsp()
                .cloned()
                .unwrap_or_else(|| self.negotiation.local_lsp().clone());
            self.peer_lsp = self
                .negotiation
                .accepted_peer_lsp()
                .cloned()
                .unwrap_or_else(|| self.local_lsp.clone());
            self.negotiation.transition_accepted();
            self.change_state(LinkStatus::Writable);
            debug!(target: self.logger, "Host has successfully negotiated link");
            return;
        }

        // Check if the accessory LSP is acceptable
        // If no: send SYN+ACK with counter offer
        // If yes: send SYN+ACK with exact copy of accessory offer
        // Save our last offer in state to be referred to when the accessory responds with ACK
        let Some(lsp) = LSPPayload::from_packet(packet) else {
            return;
        };

        if let Some(counter) = self.counter_offer(&lsp, self.negotiation.negotiation_counter()) {
            debug!("Countering with LSP: {counter:?}");
            // Unacceptable, send a counter embedded in SYN+ACK and continue negotation
            if self.negotiation.increment_counter() >= 10 {
                self.change_state(LinkStatus::Error(LinkError::LinkNegotiationRoundsExceeded));
                return;
            }

            self.negotiation.set_local_lsp(counter);
            self.negotiation.transition_offering();
        } else {
            // Acceptable, send SYN+ACK with a copy of the LSP and wait for final ACK
            self.negotiation.note_lsp_accepted_by_us(lsp.clone());
            self.negotiation.set_local_lsp(lsp.clone());
            self.peer_lsp = lsp.clone();
            debug!(target: self.logger, "Host accepted peer LSP {:?}", self.peer_lsp);
            self.negotiation.transition_offering();
        }
    }

    fn on_recv_neg(&mut self, packet: &Packet) {
        let ctl = &packet.header.control;
        let header = &packet.header;
        // Don't enforce SID=0 here; cars violating this spotted in the wild
        if self.status != LinkStatus::Negotiating
        /*|| header.session_id != LSPPayload::SESSION_ID_CONTROL*/
        {
            return;
        }

        if self.server {
            self.on_recv_neg_host(ctl, packet);
        } else {
            self.on_recv_neg_accessory(ctl, packet);
        }
    }

    pub fn close(&mut self) {
        if self.status != LinkStatus::Error(LinkError::Eof) {
            self.change_state(LinkStatus::Error(LinkError::Eof));
        }
    }

    fn process_ack_payload(&mut self, packet: &Packet) {
        let session_id = packet.header.session_id;

        let Some(session) = self.peer_lsp.sessions.iter().find(|s| s.id == session_id) else {
            return;
        };

        if self.pending_ack.unacked_packets == 0 {
            self.pending_ack.deadline = Some(self.clock.now() + Duration::from_millis(self.local_lsp.ack_timeout.into()));
        }
        self.pending_ack.unacked_packets += 1;

        match session.session_type {
            SessionType::ExternalAccessory => {
                debug!("Don't know what to do with EA payload: {:?}", packet.payload);
                self.ea.enqueue_rx(packet);
            }
            SessionType::Control => {
                let Some(payload) = &packet.payload else { return };
                if payload.is_empty() {
                    return;
                }

                trace!(
                    target: self.logger,
                    "Processing CSM payload at session id {} size {}",
                    session_id,
                    payload.len()
                );

                self.csm.enqueue_rx(packet);
            }
            SessionType::FileTransfer => {
                self.files.enqueue_rx(packet);
            }
        }

        self.drain_events();
    }

    pub fn drain_events(&mut self) {
        while let Some(ev) = self.files.dequeue_local_event() {
            self.send_event(LinkEvent::FileTransfer(ev.0, ev.1));
        }

        while let Some(csm) = self.csm.dequeue_rx() {
            self.send_event(LinkEvent::ReadCsm(csm));
        }

        while let Some(_ea) = self.ea.dequeue_rx() {
            // self.send_event(LinkEvent::ReadCsm(csm));
        }
    }

    pub fn read(&mut self, packet: PacketOrDetect) {
        if matches!(self.status, LinkStatus::Error(_)) {
            debug!("Ignored packet in error state");
            return;
        }

        self.trace_packet(false, &packet);

        let PacketOrDetect::Packet(packet) = packet else {
            if self.server {
                self.pending_ack.needs_detect_acknowledge = true;
                self.reconcile();
            } else if self.status == LinkStatus::Detecting {
                self.change_state(LinkStatus::Negotiating);
            }

            return;
        };

        if packet.size() > self.local_lsp.max_len as _ {
            debug!(
                target: self.logger,
                "Closing on oversized packet payload: packet_len={} local_max_len={}",
                packet.size(),
                self.local_lsp.max_len
            );
            self.change_state(LinkStatus::Error(LinkError::RecvMaxLenViolation {
                received: packet.size(),
                limit: self.local_lsp.max_len as _,
            }));
            return;
        } else {
            trace!(target: self.logger, "Received packet sized {} vs limit {}", packet.size(), self.local_lsp.max_len);
        }

        if self.server && self.status == LinkStatus::Detecting {
            self.change_state(LinkStatus::Negotiating);
        }

        let ctl = &packet.header.control;
        if !self.server && ctl.is_rst() {
            self.change_state(LinkStatus::Error(LinkError::Reset));
            return;
        }

        let ack_seq = packet.header.ack;

        if ctl.is_any_ack() && (self.peer_ack.is_none() || ack_seq > self.peer_ack.unwrap()) {
            debug!(
                target: self.logger,
                "Updating peer_ack from {:?} to {:?}",
                self.peer_ack,
                ack_seq
            );

            self.peer_ack.replace(ack_seq);
            if self.eak.is_active() {
                self.eak.clear();
            }
            self.retransmit_map.retain(|&seq, _| ModSeq(seq) > ack_seq);
            self.refresh_data_status();
        }

        if self.status == LinkStatus::Negotiating {
            return self.on_recv_neg(&packet);
        }

        if !matches!(self.status, LinkStatus::Writable | LinkStatus::Unwritable | LinkStatus::Recovery) {
            return;
        }

        if ctl.is_eak()
            && let Some(payload) = EAKPayload::from_packet(&packet)
            && let Some(peer_ack) = self.peer_ack
        {
            let mut psns = Vec::new();
            let delta = self.own_seq - peer_ack;

            let cur = peer_ack + ModSeq(1);
            // while cur < self.own_seq {
            //     psns.push(cur.0);
            //     cur = cur + ModSeq(1);
            // }

            // for eak in &payload.psns {
            //     if psns.contains(eak) {
            //         psns.retain(|x| x != eak);
            //     }
            // }

            psns.push(cur.0);

            debug!(
                "Received EAK: ACK={} missing={:?} delta={} retransmits={:?}",
                packet.header.ack, payload.psns, delta, psns
            );

            self.eak.begin_recovery(&payload.psns);
            self.refresh_data_status();

            // Recovery-mode retransmission is now scheduled through transmit_once().
        }

        let Some(peer_seq) = self.peer_seq else {
            debug!(target: self.logger, "Ignored packet in data state without known peer_seq");
            return;
        };

        if !ctl.is_ack() || packet.payload.is_none() {
            return;
        }

        match packet.header.seq.partial_cmp(&(peer_seq + 1)) {
            Some(Ordering::Equal) => {
                // Proper in order packet
                self.process_ack_payload(&packet);
                self.peer_seq = Some(packet.header.seq);

                // Drain packets buffered up to this point ("future packets")
                let mut next_seq = packet.header.seq + 1;

                while let Some(next_packet) = self.out_of_order.remove(&next_seq.0) {
                    debug!(target: self.logger, "Processed buffered packet payload seq={:?}", next_seq);
                    self.process_ack_payload(&next_packet);
                    self.peer_seq = Some(next_seq);
                    next_seq = next_seq + 1;
                }
            }
            Some(Ordering::Greater) => {
                debug!(target: self.logger, "Buffered out-of-order packet seq={} vs peer_seq+1={}. TODO: send EAK", packet.header.seq, peer_seq+1);
                self.out_of_order
                    .insert(packet.header.seq.0, packet.clone());
                self.pending_ack.add_out_of_order_seq(packet.header.seq.0);
            }
            Some(Ordering::Less) => {
                debug!(target: self.logger, "Scheduling ACK for duplicate seq={}", packet.header.seq);
                self.pending_ack.force = true;
            }
            None => debug!("Dropping invalid packet: seq delta overflow"),
        }
    }

    /// Recommends a delay to wait between `Instant::now()` and next call to `reconcile`, which handles retransmissions.
    pub fn sleep(&self) -> Option<Duration> {
        let now = self.clock.now();

        if let LinkStatus::Error(_) = self.status {
            trace!("Sleep not required in error state");
            return None;
        }

        if self.status == LinkStatus::Detecting {
            if self.server {
                trace!("Sleep not required");
                return None;
            }

            let ret = self
                .detect_last_transmit
                .map(|last_detect| Self::DETECT_RETRANSMIT.saturating_sub(now.saturating_duration_since(last_detect)));
            debug!("Sleep for {ret:?}");
            return ret;
        }

        let nearest_retransmit = self
            .retransmit_map
            .values()
            .map(|entry| entry.timeout_deadline)
            .min();
        let nearest_deadline = [nearest_retransmit, self.pending_ack.deadline]
            .into_iter()
            .flatten()
            .min();
        let ret = nearest_deadline.map(|deadline| deadline.saturating_duration_since(now));
        trace!("Sleep post-neg for {ret:?}");
        ret
    }
}
