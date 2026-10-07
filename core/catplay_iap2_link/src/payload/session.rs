use alloc::{collections::VecDeque, vec::Vec};

use crate::LinkSession;

pub struct PayloadSession {
    queue_tx: VecDeque<Vec<u8>>,
}

impl PayloadSession {
    pub fn new() -> Self {
        Self { queue_tx: VecDeque::new() }
    }

    pub fn enqueue_tx(&mut self, packet: Vec<u8>) {
        self.queue_tx.push_back(packet);
    }
}

impl LinkSession for PayloadSession {
    fn dequeue_tx(&mut self, _max_payload_len: usize) -> Option<Vec<u8>> {
        self.queue_tx.pop_front()
    }

    fn has_tx_pending(&self) -> bool {
        !self.queue_tx.is_empty()
    }
}
