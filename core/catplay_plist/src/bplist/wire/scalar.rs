//! Scalar operations shared by binary and XML wire codecs.

/// The result can be an object reference (binary plist) or `()` (streaming XML).
pub trait ScalarEncoder {
    type Error;
    type Item;

    fn boolean(&mut self, value: bool) -> Result<Self::Item, Self::Error>;
    fn integer(&mut self, value: i64) -> Result<Self::Item, Self::Error>;
    fn unsigned_integer(&mut self, value: u64) -> Result<Self::Item, Self::Error>;
    fn real(&mut self, value: f64) -> Result<Self::Item, Self::Error>;
    fn string(&mut self, value: &str) -> Result<Self::Item, Self::Error>;
    fn data(&mut self, value: &[u8]) -> Result<Self::Item, Self::Error>;
    fn uid(&mut self, value: u64) -> Result<Self::Item, Self::Error>;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DecodedInteger {
    Signed(i64),
    Unsigned(u64),
}

/// Typed access to scalars after each wire format has identified an item.
pub trait ScalarDecoder {
    type Error;

    fn boolean(&self) -> Result<Option<bool>, Self::Error>;
    fn integer(&self) -> Result<Option<DecodedInteger>, Self::Error>;
    fn real(&self) -> Result<Option<f64>, Self::Error>;
}
