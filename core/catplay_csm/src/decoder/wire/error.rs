#[derive(PartialEq, Eq, Debug)]
pub enum CsmError {
    Overflow,
    Underflow,

    PacketMagic,
    PacketUnderflow,
    PacketOverflow,
    PacketUnknown,
}

pub type CsmResult<T> = Result<T, CsmError>;
