//! Memory region inspection utilities.

mod inspector;
mod record;
mod status;
mod value;

pub use self::inspector::{Inspector, RebasedInspector};
pub use self::record::{Record, RecordBuilder};
pub use self::status::Status;
pub use self::value::InspectionValue;

/// Defines the ability to inspect a memory region.
pub trait Inspect {
    /// Inspects the region by emitting region records.
    fn inspect(&self, inspector: &mut dyn Inspector);
}
