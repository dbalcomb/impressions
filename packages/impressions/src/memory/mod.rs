//! Memory representation and manipulation.

pub mod address;
pub mod extent;
pub mod regions;
pub mod segmented;

mod slice;

pub use self::slice::{Slice, SliceBoundsError};
