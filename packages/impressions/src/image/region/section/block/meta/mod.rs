//! The metadata block.

pub mod hint_name_table;
pub mod import_address_table;
pub mod import_directory_table;
pub mod import_lookup_table;
pub mod import_name;

mod cursor;

use std::fmt::{self, Debug};

use serde::{Deserialize, Serialize};

use crate::analysis::Completion;
use crate::memory::cursor::AsCursor;
use crate::memory::extent::{Extent, Size};
use crate::memory::inspect::{Inspect, Inspector};

pub use self::cursor::MetaCursor;

use self::hint_name_table::HintNameTable;
use self::import_address_table::ImportAddressTable;
use self::import_directory_table::ImportDirectoryTable;
use self::import_lookup_table::ImportLookupTable;
use self::import_name::ImportName;

/// A block of metadata.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Meta {
    /// The import directory table.
    ImportDirectoryTable(ImportDirectoryTable),

    /// An import lookup table.
    ImportLookupTable(ImportLookupTable),

    /// An import address table.
    ImportAddressTable(ImportAddressTable),

    /// The hint/name table.
    HintNameTable(HintNameTable),

    /// An imported DLL name.
    ImportName(ImportName),
}

impl Extent for Meta {
    fn size(&self) -> Size {
        match self {
            Self::ImportDirectoryTable(table) => table.size(),
            Self::ImportLookupTable(table) => table.size(),
            Self::ImportAddressTable(table) => table.size(),
            Self::HintNameTable(table) => table.size(),
            Self::ImportName(name) => name.size(),
        }
    }
}

impl Completion for Meta {
    fn identified(&self) -> u64 {
        match self {
            Self::ImportDirectoryTable(table) => table.identified(),
            Self::ImportLookupTable(table) => table.identified(),
            Self::ImportAddressTable(table) => table.identified(),
            Self::HintNameTable(table) => table.identified(),
            Self::ImportName(name) => name.identified(),
        }
    }
}

impl Inspect for Meta {
    fn inspect(&self, inspector: &mut dyn Inspector) {
        match self {
            Self::ImportDirectoryTable(table) => table.inspect(inspector),
            Self::ImportLookupTable(table) => table.inspect(inspector),
            Self::ImportAddressTable(table) => table.inspect(inspector),
            Self::HintNameTable(table) => table.inspect(inspector),
            Self::ImportName(name) => name.inspect(inspector),
        }
    }
}

impl Debug for Meta {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ImportDirectoryTable(table) => Debug::fmt(table, f),
            Self::ImportLookupTable(table) => Debug::fmt(table, f),
            Self::ImportAddressTable(table) => Debug::fmt(table, f),
            Self::HintNameTable(table) => Debug::fmt(table, f),
            Self::ImportName(name) => Debug::fmt(name, f),
        }
    }
}

impl AsCursor for Meta {
    type Cursor<'a> = MetaCursor<'a>;

    fn cursor(&self) -> Self::Cursor<'_> {
        MetaCursor::new(self)
    }
}
