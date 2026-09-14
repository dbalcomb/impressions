/// An address error.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum Error {
    /// The address was not a valid hexadecimal 32-bit value.
    #[error("invalid hexadecimal address")]
    Parse(#[from] std::num::ParseIntError),
}
