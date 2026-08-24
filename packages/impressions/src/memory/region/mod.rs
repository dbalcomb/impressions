//! Memory region representation and manipulation.

/// Defines a memory region.
///
/// A memory region is a contiguous block of memory that can be used for storing
/// data or code.
pub trait Region {
    /// Gets the size of the region.
    fn size(&self) -> u64;
}
