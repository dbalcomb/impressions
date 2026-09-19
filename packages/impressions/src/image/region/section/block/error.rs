/// A block region error.
#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum Error {
    /// An error occurred with the metadata block.
    #[error("metadata error")]
    Meta(#[from] super::meta::Error),
}
