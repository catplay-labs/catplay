#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "alloc")]
use alloc::boxed::Box;
use core::any::Any;
use core::fmt::Debug;

use super::super::*;

pub trait CsmPacketId {
    const PACKET_ID: u16;
}

pub trait CsmPacket: CsmPayloadEncode + Any + Debug + Send + Sync + 'static {
    #[cfg(feature = "clone_box")]
    fn clone_box(&self) -> CsmPacketBox;
}

#[cfg(feature = "clone_box")]
impl<T> CsmPacket for T
where
    T: CsmPayloadEncode + Any + Debug + Send + Sync + Clone + 'static,
{
    fn clone_box(&self) -> CsmPacketBox {
        self.clone().into()
    }
}

#[cfg(all(feature = "alloc", not(feature = "clone_box")))]
impl<T> CsmPacket for T where T: CsmPayloadEncode + Any + Debug + Send + Sync + 'static {}

#[cfg(not(feature = "alloc"))]
impl<T> CsmPacket for T where T: CsmPayloadEncode + Any + Debug + Send + Sync + 'static {}

#[cfg(feature = "alloc")]
pub type CsmPacketBox = Box<dyn CsmPacket>;

#[cfg(feature = "alloc")]
impl<T: CsmPacket + 'static> From<T> for CsmPacketBox {
    fn from(value: T) -> Self {
        Box::new(value)
    }
}

#[cfg(feature = "clone_box")]
impl Clone for CsmPacketBox {
    fn clone(&self) -> Self {
        self.as_ref().clone_box()
    }
}

impl AsRef<dyn CsmPacket> for dyn CsmPacket {
    fn as_ref(&self) -> &dyn CsmPacket {
        self
    }
}

pub trait AsCsmPacket: Send + Sync {
    fn as_csm(&self) -> &dyn CsmPacket;
}

impl<T> AsCsmPacket for T
where
    T: AsRef<dyn CsmPacket> + Send + Sync,
{
    fn as_csm(&self) -> &dyn CsmPacket {
        self.as_ref()
    }
}

impl dyn CsmPacket {
    pub fn as_any(&self) -> &dyn Any {
        self as &dyn Any
    }

    /// Casts CsmPacket to it's implementation subtype or returns None if not a match
    pub fn cast<T: CsmPacket>(&self) -> Option<&T> {
        self.as_any().downcast_ref::<T>()
    }
}
