/// An import lookup entry error.
#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum Error {
    /// The lookup entry could not be parsed.
    #[error("parse error")]
    Parse(#[from] bytes::TryGetError),
}
