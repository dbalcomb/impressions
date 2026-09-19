/// A metadata block error.
#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum Error {
    /// An import address table error.
    #[error("import address table error")]
    ImportAddressTable(#[from] super::import_address_table::Error),

    /// An import directory table error.
    #[error("import directory table error")]
    ImportDirectoryTable(#[from] super::import_directory_table::Error),

    /// An import lookup table error.
    #[error("import lookup table error")]
    ImportLookupTable(#[from] super::import_lookup_table::Error),

    /// An import name error.
    #[error("import name error")]
    ImportName(#[from] super::import_name::Error),
}
