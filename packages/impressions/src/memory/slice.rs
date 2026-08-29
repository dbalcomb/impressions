/// Defines the ability to slice a region of memory.
pub trait Slice: Sized {
    /// The associated error type for slicing operations.
    type Error;

    /// Slices the region from the given offset with the provided size.
    fn slice(&self, offset: u32, size: u64) -> Result<Self, Self::Error>;
}

/// The requested slice is outside the source region.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error(
    "the slice at offset {offset} with size {size} is out of bounds for region size {region_size}"
)]
pub struct SliceBoundsError {
    pub offset: u32,
    pub size: u64,
    pub region_size: u64,
}
