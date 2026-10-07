#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Iap1Source {
    Accessory,
    Device,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransactionIdPolicy {
    Permitted,
    Prohibited,
    Required,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Iap1MessageMetadata {
    pub lingo: u8,
    pub command: u16,
    pub source: Iap1Source,
    pub response: bool,
    pub ack: bool,
    pub deprecated: bool,
    pub transaction_id: TransactionIdPolicy,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct DecodeContext {
    pub adjusted_payload_length: usize,
    pub has_transaction_id: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct EncodeContext {
    pub adjusted_payload_length: usize,
    pub has_transaction_id: bool,
}

impl DecodeContext {
    pub const fn encoding(&self) -> bool {
        false
    }
}
impl EncodeContext {
    pub const fn encoding(&self) -> bool {
        true
    }
}
