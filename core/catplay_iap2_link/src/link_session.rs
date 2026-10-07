use alloc::vec::Vec;

pub struct AckToken(pub u64);

pub trait LinkSession {
    fn dequeue_tx(&mut self, max_payload_len: usize) -> Option<Vec<u8>>;

    fn has_tx_pending(&self) -> bool;
}
