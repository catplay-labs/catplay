#![no_std]
#![deny(unsafe_op_in_unsafe_fn)]
#![doc = include_str!("../README.md")]

mod pool;
mod project;

pub use pool::{HandleShared, PoolHandle, StaticHandle, StaticObjectPool, StringPool};
pub use project::{
    ConstOffset, ErasedContainerItem, FieldAccess, FieldEntry, FieldOffset, FieldOp, FieldOps, FieldProject, FieldProjector, FieldTable,
    OffsetWidth, ProjectorCapability, ProjectorContainerCapability, ProjectorOptionalCapability, apply_container_erased,
    project_container_item,
};

/// Implementation details used by exported macros; not a stable public API.
#[doc(hidden)]
pub use pool::__private;
