/// An import address table error.
#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum Error {
    /// An error occured with an import address entry.
    #[error("entry error")]
    Entry(#[from] crate::image::region::section::block::meta::import_lookup_table::entry::Error),
}
