/// The imports analyser error.
#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum Error {
    /// A problem was encountered with the cursor.
    #[error("cursor error")]
    Cursor(#[from] crate::memory::cursor::Error),

    /// A problem was encountered reading the import table.
    #[error("import table error")]
    ImportTable(
        #[from] crate::memory::cursor::ops::read::Error<crate::image::region::section::Error>,
    ),

    /// A problem was encountered reading the import name.
    #[error("import name error")]
    ImportName(
        #[from]
        crate::memory::cursor::ops::read::Error<
            crate::image::region::section::block::meta::import_name::Error,
        >,
    ),

    /// A problem was encountered with the image.
    #[error("image error")]
    Image(#[from] crate::image::Error),
}
