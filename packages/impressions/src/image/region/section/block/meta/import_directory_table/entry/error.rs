/// An import directory entry error.
#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum Error {
    /// The directory entry could not be parsed.
    #[error("parse error")]
    Parse(#[from] bytes::TryGetError),
}
