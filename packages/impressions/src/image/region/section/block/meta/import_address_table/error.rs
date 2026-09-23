/// An import address table error.
#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum Error {
    /// An error occured while decoding an import address table.
    #[error("decode error")]
    Decode(
        #[from]
        crate::memory::region::types::table::Error<
            crate::image::region::section::block::meta::import_lookup_table::entry::Error,
        >,
    ),
}
