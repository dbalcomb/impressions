//! Memory size representation and manipulation.

mod error;
mod size;

pub use self::error::Error;
pub use self::size::Size;

/// Defines the size of a memory region.
pub trait Extent {
    /// Gets the size of the region.
    fn size(&self) -> Size;
}
