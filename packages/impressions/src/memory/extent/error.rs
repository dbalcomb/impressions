/// The error type for memory region sizes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
pub enum Error {
    #[error("a memory region cannot have zero size")]
    Zero,

    #[error("the size {0} exceeds the 32-bit address space")]
    TooLarge(u64),
}
