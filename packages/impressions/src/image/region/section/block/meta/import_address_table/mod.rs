//! The import address table.

mod cursor;
mod error;

use bytes::Buf;
use serde::{Deserialize, Serialize};

use crate::analysis::Completion;
use crate::data::parse::Parse;
use crate::image::region::section::block::meta::import_lookup_table::entry::ImportLookup;
use crate::memory::cursor::AsCursor;
use crate::memory::extent::{Extent, Size};
use crate::memory::inspect::{Inspect, Inspector};
use crate::memory::region::Null;
use crate::memory::region::types::segmented::{Segmented, Segments};

pub use self::cursor::ImportAddressTableCursor;
pub use self::error::Error;

/// A table of import addresses.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
#[repr(transparent)]
pub struct ImportAddressTable(Vec<ImportLookup>);

impl ImportAddressTable {
    /// Gets an iterator over the import addresses.
    pub fn iter(&self) -> impl Iterator<Item = &ImportLookup> {
        self.0.iter().filter(|lookup| !lookup.is_null())
    }
}

impl Extent for ImportAddressTable {
    fn size(&self) -> Size {
        Size::try_sum(self.0.iter().map(Extent::size)).expect("valid size")
    }
}

impl Completion for ImportAddressTable {
    fn identified(&self) -> u64 {
        self.size().get()
    }
}

impl Segmented for ImportAddressTable {
    type Segment = ImportLookup;

    fn segments(&self) -> Segments<'_, Self::Segment> {
        Segments::new(&self.0)
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

impl Parse for ImportAddressTable {
    type Context<'a> = ();
    type Error = Error;

    fn parse_with(mut buffer: impl Buf, _: Self::Context<'_>) -> Result<Self, Self::Error> {
        let mut table = Vec::new();

        loop {
            let import_address = ImportLookup::parse(&mut buffer)?;

            if import_address.is_null() {
                table.push(import_address);

                break;
            }

            table.push(import_address);
        }

        Ok(Self(table))
    }
}

impl AsCursor for ImportAddressTable {
    type Cursor<'a> = ImportAddressTableCursor<'a>;

    fn cursor(&self) -> Self::Cursor<'_> {
        ImportAddressTableCursor::new(self)
    }
}
