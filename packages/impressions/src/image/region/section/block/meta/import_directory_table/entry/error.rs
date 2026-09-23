/// An import directory entry error.
#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum Error {
    /// The directory entry could not be decoded.
    #[error("decode error")]
    Decode(#[from] crate::memory::region::ops::decode::Error),
}
