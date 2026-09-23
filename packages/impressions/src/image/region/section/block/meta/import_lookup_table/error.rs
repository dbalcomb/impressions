/// An import lookup table error.
#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum Error {
    /// An error occured while decoding an import lookup table.
    #[error("decode error")]
    Decode(#[from] crate::memory::region::types::table::Error<super::entry::Error>),
}
