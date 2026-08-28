/// The unidentified region error.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum Error {
    /// The size of the unidentified region exceeds the maximum region size.
    #[error("the region size {0} exceeds maximum {max}", max = u32::MAX as u64 + 1)]
    SizeTooLarge(u64),

    /// The uninitialized region is invalid.
    #[error("the uninitialized region is invalid")]
    Uninitialized(#[from] crate::memory::regions::uninitialized::Error),
}
