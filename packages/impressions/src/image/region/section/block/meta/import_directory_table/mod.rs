//! The import directory table.

pub mod entry;

mod cursor;
mod error;

use std::ops::Deref;

use bytes::Buf;
use serde::{Deserialize, Serialize};

use crate::analysis::Completion;
use crate::data::parse::Parse;
use crate::image::region::headers::header::optional::directories::directory::DataDirectory;
use crate::memory::cursor::AsCursor;
use crate::memory::extent::{Extent, Size};
use crate::memory::inspect::{Inspect, Inspector};
use crate::memory::region::types::table::Table;

pub use self::cursor::ImportDirectoryTableCursor;
pub use self::error::Error;

use self::entry::ImportDirectory;

/// A table of import directories.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
#[repr(transparent)]
pub struct ImportDirectoryTable(Table<ImportDirectory>);

impl Extent for ImportDirectoryTable {
    fn size(&self) -> Size {
        self.0.size()
    }
}

impl Completion for ImportDirectoryTable {
    fn identified(&self) -> u64 {
        self.size().get()
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

    fn parse_with(buffer: impl Buf, context: Self::Context<'_>) -> Result<Self, Self::Error> {
        let size = Size::new(context.target_size().into())?;

        Table::parse_with(buffer, Some(size))
            .map(Self)
            .map_err(Error::Parse)
    }
}

impl AsCursor for ImportDirectoryTable {
    type Cursor<'a> = ImportDirectoryTableCursor<'a>;

    fn cursor(&self) -> Self::Cursor<'_> {
        ImportDirectoryTableCursor::new(self)
    }
}

impl Deref for ImportDirectoryTable {
    type Target = Table<ImportDirectory>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
