//! The import lookup table.

pub mod entry;

mod cursor;
mod error;

use std::ops::Deref;

use serde::{Deserialize, Serialize};

use crate::analysis::Completion;
use crate::memory::cursor::AsCursor;
use crate::memory::extent::{Extent, Size};
use crate::memory::inspect::{Inspect, Inspector};
use crate::memory::region::ops::decode::{Decode, Decoder};
use crate::memory::region::ops::encode::{self, Encode};
use crate::memory::region::types::table::Table;

pub use self::cursor::ImportLookupTableCursor;
pub use self::error::Error;

use self::entry::ImportLookup;

/// A table of import lookups.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
#[repr(transparent)]
pub struct ImportLookupTable(Table<ImportLookup>);

impl Extent for ImportLookupTable {
    fn size(&self) -> Size {
        self.0.size()
    }
}

impl Completion for ImportLookupTable {
    fn identified(&self) -> u64 {
        self.size().get()
    }
}

impl Inspect for ImportLookupTable {
    fn inspect(&self, inspector: &mut dyn Inspector) {
        inspector
            .record(self.address_space())
            .label(&"Import Lookup Table")
            .finish()
    }
}

impl Encode for ImportLookupTable {
    fn encode(&self, encoder: &mut dyn encode::Encoder) -> Result<(), encode::Error> {
        self.0.encode(encoder)
    }
}

impl Decode for ImportLookupTable {
    type Context<'a> = ();
    type Error = Error;

    fn decode_with(decoder: &mut dyn Decoder, _: Self::Context<'_>) -> Result<Self, Self::Error> {
        Table::decode_with(decoder, None)
            .map(Self)
            .map_err(Error::Decode)
    }
}

impl AsCursor for ImportLookupTable {
    type Cursor<'a> = ImportLookupTableCursor<'a>;

    fn cursor(&self) -> Self::Cursor<'_> {
        ImportLookupTableCursor::new(self)
    }
}

impl Deref for ImportLookupTable {
    type Target = Table<ImportLookup>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
