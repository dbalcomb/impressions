/// An import directory table error.
#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum Error {
    /// An error occured while decoding an import directory table.
    #[error("decode error")]
    Decode(#[from] crate::memory::region::types::table::Error<super::entry::Error>),

    /// An error occured with the size of the import directory table.
    #[error("size error")]
    Size(#[from] crate::memory::extent::Error),
}
