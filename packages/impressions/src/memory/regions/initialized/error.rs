/// The initialized region error.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum Error {
    /// The size of the initialized region exceeds the maximum region size.
    #[error("the region size {0} exceeds maximum {max}", max = u32::MAX as u64 + 1)]
    SizeTooLarge(u64),
}
