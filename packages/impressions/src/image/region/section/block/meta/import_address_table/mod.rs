//! The import address table.

mod cursor;
mod error;

use std::ops::Deref;

use serde::{Deserialize, Serialize};

use crate::analysis::Completion;
use crate::image::region::section::block::meta::import_lookup_table::entry::ImportLookup;
use crate::memory::cursor::AsCursor;
use crate::memory::extent::{Extent, Size};
use crate::memory::inspect::{Inspect, Inspector};
use crate::memory::region::ops::decode::{Decode, Decoder};
use crate::memory::region::ops::encode::{self, Encode};
use crate::memory::region::types::table::Table;

pub use self::cursor::ImportAddressTableCursor;
pub use self::error::Error;

/// A table of import addresses.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
#[repr(transparent)]
pub struct ImportAddressTable(Table<ImportLookup>);

impl Extent for ImportAddressTable {
    fn size(&self) -> Size {
        self.0.size()
    }
}

impl Completion for ImportAddressTable {
    fn identified(&self) -> u64 {
        self.size().get()
    }
}

impl Inspect for ImportAddressTable {
    fn inspect(&self, inspector: &mut dyn Inspector) {
        inspector
            .record(self.address_space())
            .label(&"Import Address Table")
            .finish()
    }
}

impl Encode for ImportAddressTable {
    fn encode(&self, encoder: &mut dyn encode::Encoder) -> Result<(), encode::Error> {
        self.0.encode(encoder)
    }
}

impl Decode for ImportAddressTable {
    type Context<'a> = ();
    type Error = Error;

    fn decode_with(decoder: &mut dyn Decoder, _: Self::Context<'_>) -> Result<Self, Self::Error> {
        Table::decode_with(decoder, None)
            .map(Self)
            .map_err(Error::Decode)
    }
}

impl AsCursor for ImportAddressTable {
    type Cursor<'a> = ImportAddressTableCursor<'a>;

    fn cursor(&self) -> Self::Cursor<'_> {
        ImportAddressTableCursor::new(self)
    }
}

impl Deref for ImportAddressTable {
    type Target = Table<ImportLookup>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
