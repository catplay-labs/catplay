//! Typed field projection tables and projector capabilities.

mod capability;
mod entry;
mod macros;
mod offset;
mod option;
mod table;
mod vec;

pub use capability::{FieldAccess, FieldProjector, ProjectorCapability};
pub use entry::FieldEntry;
pub use offset::{ConstOffset, FieldOffset, OffsetWidth};
pub use option::ProjectorOptionalCapability;
pub use table::{FieldOp, FieldOps, FieldProject, FieldTable};
pub use vec::{ErasedContainerItem, ProjectorContainerCapability, apply_container_erased, project_container_item};
