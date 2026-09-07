//! Memory representation and manipulation.

pub mod address;
pub mod regions;
pub mod segmented;

mod extent;
mod slice;

pub use self::extent::{Error as SizeError, Extent, Size};
pub use self::slice::{Slice, SliceBoundsError};
