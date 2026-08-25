/// Defines the size of a memory region.
pub trait Extent {
    /// Gets the size of the region.
    fn size(&self) -> u64;
}
