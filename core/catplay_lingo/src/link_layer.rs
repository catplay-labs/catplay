//! iAP1 link state above the frame codec and below an application session.
//! The caller supplies elapsed time; this layer has no transport or clock dependency.

use alloc::vec::Vec;
use core::time::Duration;

use crate::{
    transaction_helper::{TransactionError, TransactionHelper},
    wire::{Frame, Outgoing, Packet},
};

pub struct LinkLayer {
    transactions: TransactionHelper,
    scheduled: Vec<(Duration, Outgoing)>,
}

impl LinkLayer {
    pub fn new() -> Self {
        Self::with_role(false)
    }

    /// `server = true` uses the Apple-device role.
    pub fn with_role(server: bool) -> Self {
        Self {
            transactions: TransactionHelper::new(server),
            scheduled: Vec::new(),
        }
    }

    pub fn transactions(&self) -> &TransactionHelper {
        &self.transactions
    }

    pub fn transactions_mut(&mut self) -> &mut TransactionHelper {
        &mut self.transactions
    }

    /// The application chooses the message; the link layer holds it until due.
    pub fn schedule(&mut self, due: Duration, outgoing: Outgoing) {
        self.scheduled.push((due, outgoing));
    }

    pub fn poll(&mut self, elapsed: Duration) -> Result<Vec<Packet>, TransactionError> {
        if !self.scheduled.iter().any(|(due, _)| *due <= elapsed) {
            return Ok(Vec::new());
        }
        let mut trial = self.transactions.clone();
        let mut ready = Vec::new();
        for (index, (due, outgoing)) in self.scheduled.iter().enumerate() {
            if *due <= elapsed {
                let transaction_id = trial.prepare_outgoing(&outgoing.message, outgoing.transaction)?;
                ready.push((index, transaction_id));
            }
        }
        self.transactions = trial;
        let mut packets = Vec::with_capacity(ready.len());
        for (index, transaction_id) in ready.into_iter().rev() {
            let (_, outgoing) = self.scheduled.remove(index);
            packets.push(Packet {
                message: outgoing.message,
                transaction_id,
            });
        }
        packets.reverse();
        Ok(packets)
    }

    /// Decode a frame without interpreting whether it completes a transaction.
    pub fn on_frame(&mut self, frame: &Frame<'_>) -> Result<Packet, TransactionError> {
        self.transactions.decode(frame)
    }

    pub fn send(&mut self, outgoing: Outgoing) -> Result<Packet, TransactionError> {
        let transaction_id = self
            .transactions
            .prepare_outgoing(&outgoing.message, outgoing.transaction)?;
        Ok(Packet {
            message: outgoing.message,
            transaction_id,
        })
    }
}

impl Default for LinkLayer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        Iap1Message,
        lingo_0x0::{self as general, LingoMessage as GeneralMessage},
        wire::TransactionIntent,
    };

    #[test]
    fn schedules_and_leaves_completion_to_caller() {
        let mut link = LinkLayer::new();
        link.schedule(
            Duration::from_secs(3),
            Outgoing::new(
                GeneralMessage::StartIDPS(general::StartIDPS {}),
                TransactionIntent::New { start_transaction: true },
            ),
        );
        assert!(link.poll(Duration::from_secs(2)).unwrap().is_empty());
        let start = link.poll(Duration::from_secs(3)).unwrap().remove(0);
        let id = start.transaction_id.unwrap();
        let ack = Packet {
            message: GeneralMessage::IPodAck(general::IPodAck {
                command_result: general::IPodAckCommandResult::OK,
                acked_command_id: general::StartIDPS::META.command as u8,
                maximum_pending_wait: None,
                session_id: None,
                num_bytes_dropped: None,
            })
            .into(),
            transaction_id: Some(id),
        };
        let body = ack.encode_body().unwrap();
        let frame = Frame { body: &body };
        let received = link.on_frame(&frame).unwrap();
        assert_eq!(received.transaction_id, Some(id));
        assert_eq!(link.transactions().pending_command(id), Some(general::StartIDPS::KEY));
        assert!(link.transactions_mut().finish_transaction(id).is_some());
    }
}
