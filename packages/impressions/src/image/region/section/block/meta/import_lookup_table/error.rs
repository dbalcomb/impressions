/// An import lookup table error.
#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum Error {
    /// An error occured while parsing an import lookup table.
    #[error("parse error")]
    Parse(#[from] crate::memory::region::types::table::Error<super::entry::Error>),
}
