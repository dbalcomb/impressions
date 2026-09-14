/// An address space error.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum Error {
    /// An address in the address space was invalid.
    #[error(transparent)]
    Address(#[from] super::super::Error),

    /// The address space was invalid.
    #[error("invalid address space")]
    Invalid,
}
