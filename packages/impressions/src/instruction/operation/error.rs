/// An operation error.
#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum Error {
    /// A problem was encountered decoding an operation.
    #[error("decode error")]
    Decode(#[from] crate::memory::region::ops::decode::Error),

    /// An invalid opcode was encountered.
    #[error("invalid opcode {0:02x}")]
    InvalidOpcode(u8),
}
