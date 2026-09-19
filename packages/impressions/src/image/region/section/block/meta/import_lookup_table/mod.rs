//! The import lookup table.

pub mod entry;

mod cursor;
mod error;

use bytes::Buf;
use serde::{Deserialize, Serialize};

use crate::analysis::Completion;
use crate::data::parse::Parse;
use crate::memory::cursor::AsCursor;
use crate::memory::extent::{Extent, Size};
use crate::memory::inspect::{Inspect, Inspector};
use crate::memory::region::Null;
use crate::memory::region::types::segmented::{Segmented, Segments};

pub use self::cursor::ImportLookupTableCursor;
pub use self::error::Error;

use self::entry::ImportLookup;

/// A table of import lookups.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
#[repr(transparent)]
pub struct ImportLookupTable(Vec<ImportLookup>);

impl ImportLookupTable {
    /// Gets an iterator over the import lookups.
    pub fn iter(&self) -> impl Iterator<Item = &ImportLookup> {
        self.0.iter().filter(|lookup| !lookup.is_null())
    }
}

impl Extent for ImportLookupTable {
    fn size(&self) -> Size {
        Size::try_sum(self.0.iter().map(Extent::size))
            .expect("sum of sizes does not exceed maximum size")
    }
}

impl Completion for ImportLookupTable {
    fn identified(&self) -> u64 {
        self.size().get()
    }
}

impl Segmented for ImportLookupTable {
    type Segment = ImportLookup;

    fn segments(&self) -> Segments<'_, Self::Segment> {
        Segments::new(&self.0)
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

impl Parse for ImportLookupTable {
    type Context<'a> = ();
    type Error = Error;

    fn parse_with(mut buffer: impl Buf, _: Self::Context<'_>) -> Result<Self, Self::Error> {
        let mut table = Vec::new();

        loop {
            let import_lookup = ImportLookup::parse(&mut buffer)?;

            if import_lookup.is_null() {
                table.push(import_lookup);

                break;
            }

            table.push(import_lookup);
        }

        Ok(Self(table))
    }
}

impl AsCursor for ImportLookupTable {
    type Cursor<'a> = ImportLookupTableCursor<'a>;

    fn cursor(&self) -> Self::Cursor<'_> {
        ImportLookupTableCursor::new(self)
    }
}
