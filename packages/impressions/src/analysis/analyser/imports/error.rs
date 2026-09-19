/// The imports analyser error.
#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum Error {
    /// A problem was encountered with the cursor.
    #[error("cursor error")]
    Cursor(#[from] crate::memory::cursor::Error),

    /// A problem was encountered reading the import address table.
    #[error("import address table error")]
    ImportAddressTable(
        #[from]
        crate::memory::cursor::ops::read::Error<
            crate::image::region::section::block::meta::import_address_table::Error,
        >,
    ),

    /// A problem was encountered reading the import directory table.
    #[error("import directory table error")]
    ImportDirectoryTable(
        #[from]
        crate::memory::cursor::ops::read::Error<
            crate::image::region::section::block::meta::import_directory_table::Error,
        >,
    ),

    /// A problem was encountered reading the import lookup table.
    #[error("import lookup table error")]
    ImportLookupTable(
        #[from]
        crate::memory::cursor::ops::read::Error<
            crate::image::region::section::block::meta::import_lookup_table::Error,
        >,
    ),

    /// A problem was encountered reading the import name.
    #[error("import name error")]
    ImportName(
        #[from]
        crate::memory::cursor::ops::read::Error<
            crate::image::region::section::block::meta::import_name::Error,
        >,
    ),

    /// A problem was encountered reading the import hint/name entry.
    #[error("hint/name error")]
    HintName(
        #[from]
        crate::memory::cursor::ops::read::Error<
            crate::image::region::section::block::meta::hint_name_table::entry::Error,
        >,
    ),

    /// A problem was encountered with the image.
    #[error("image error")]
    Image(#[from] crate::image::Error),
}
