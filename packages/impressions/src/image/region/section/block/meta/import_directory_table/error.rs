/// An import directory table error.
#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum Error {
    /// An error occured with an import directory entry.
    #[error("entry error")]
    Entry(#[from] super::entry::Error),
}
