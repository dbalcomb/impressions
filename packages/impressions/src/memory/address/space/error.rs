/// An address space error.
#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum Error {
    /// The address space was invalid.
    #[error("Invalid address space")]
    Invalid,
}
