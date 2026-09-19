//! The import directory table.

pub mod entry;

mod cursor;

use bytes::Buf;
use serde::{Deserialize, Serialize};

use crate::analysis::Completion;
use crate::data::parse::Parse;
use crate::image::region::headers::header::optional::directories::directory::DataDirectory;
use crate::image::region::section::Error;
use crate::memory::cursor::AsCursor;
use crate::memory::extent::{Extent, FixedExtent, Size};
use crate::memory::inspect::{Inspect, Inspector};
use crate::memory::region::types::segmented::{Segmented, Segments};

pub use self::cursor::ImportDirectoryTableCursor;

use self::entry::ImportDirectory;

/// A table of import directories.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
#[repr(transparent)]
pub struct ImportDirectoryTable(Vec<ImportDirectory>);

impl ImportDirectoryTable {
    /// Gets an iterator over the import directories.
    pub fn iter(&self) -> impl Iterator<Item = &ImportDirectory> {
        self.0.iter().filter(|directory| !directory.is_null())
    }
}

impl Extent for ImportDirectoryTable {
    fn size(&self) -> Size {
        Size::try_sum(self.0.iter().map(Extent::size))
            .expect("sum of sizes does not exceed maximum size")
    }
}

impl Completion for ImportDirectoryTable {
    fn identified(&self) -> u64 {
        self.size().get()
    }
}

impl Segmented for ImportDirectoryTable {
    type Segment = ImportDirectory;

    fn segments(&self) -> Segments<'_, Self::Segment> {
        Segments::new(&self.0)
    }
}

impl Inspect for ImportDirectoryTable {
    fn inspect(&self, inspector: &mut dyn Inspector) {
        inspector
            .record(self.address_space())
            .label(&"Import Directory Table")
            .finish()
    }
}

impl Parse for ImportDirectoryTable {
    type Context<'a> = &'a DataDirectory;
    type Error = Error;

    fn parse_with(mut buffer: impl Buf, context: Self::Context<'_>) -> Result<Self, Self::Error> {
        let len = context.target_size() as usize / ImportDirectory::SIZE.get() as usize;
        let mut table = Vec::new();

        for i in 1..=context.target_size() {
            let import_directory = ImportDirectory::parse(&mut buffer)?;

            if import_directory.is_null() {
                assert!(
                    i == len as u32,
                    "import directory table should be null-terminated"
                );

                table.push(import_directory);

                break;
            }

            table.push(import_directory);
        }

        Ok(Self(table))
    }
}

impl AsCursor for ImportDirectoryTable {
    type Cursor<'a> = ImportDirectoryTableCursor<'a>;

    fn cursor(&self) -> Self::Cursor<'_> {
        ImportDirectoryTableCursor::new(self)
    }
}
