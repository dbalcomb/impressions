use std::fmt::{self, Debug};

use crate::memory::cursor::{AsCursor, Cursor, Error, Position};
use crate::memory::inspect::{Inspect, Inspector};

use super::Meta;
use super::import_directory_table::ImportDirectoryTableCursor;
use super::import_lookup_table::ImportLookupTableCursor;

/// A cursor over a block of metadata.
#[derive(Clone)]
pub enum MetaCursor<'a> {
    ImportDirectoryTable(ImportDirectoryTableCursor<'a>),
    ImportLookupTable(ImportLookupTableCursor<'a>),
}

impl<'a> MetaCursor<'a> {
    /// Constructs a new metadata cursor.
    pub(super) fn new(meta: &'a Meta) -> Self {
        match meta {
            Meta::ImportDirectoryTable(table) => Self::ImportDirectoryTable(table.cursor()),
            Meta::ImportLookupTable(table) => Self::ImportLookupTable(table.cursor()),
        }
    }
}

impl Cursor for MetaCursor<'_> {
    type Error = Error;

    fn position(&self) -> Position {
        match self {
            Self::ImportDirectoryTable(cursor) => cursor.position(),
            Self::ImportLookupTable(cursor) => cursor.position(),
        }
    }

    fn seek(&mut self, position: Position) -> Result<(), Self::Error> {
        match self {
            Self::ImportDirectoryTable(cursor) => cursor.seek(position),
            Self::ImportLookupTable(cursor) => cursor.seek(position),
        }
    }

    fn advance(&mut self, offset: u32) -> Result<(), Self::Error> {
        match self {
            Self::ImportDirectoryTable(cursor) => cursor.advance(offset),
            Self::ImportLookupTable(cursor) => cursor.advance(offset),
        }
    }

    fn next(&mut self) -> Result<Option<Position>, Self::Error> {
        match self {
            Self::ImportDirectoryTable(cursor) => cursor.next(),
            Self::ImportLookupTable(cursor) => cursor.next(),
        }
    }

    fn step(&mut self) -> Result<Option<Position>, Self::Error> {
        match self {
            Self::ImportDirectoryTable(cursor) => cursor.step(),
            Self::ImportLookupTable(cursor) => cursor.step(),
        }
    }
}

impl Inspect for MetaCursor<'_> {
    fn inspect(&self, inspector: &mut dyn Inspector) {
        match self {
            Self::ImportDirectoryTable(cursor) => cursor.inspect(inspector),
            Self::ImportLookupTable(cursor) => cursor.inspect(inspector),
        }
    }
}

impl Debug for MetaCursor<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ImportDirectoryTable(cursor) => Debug::fmt(cursor, f),
            Self::ImportLookupTable(cursor) => Debug::fmt(cursor, f),
        }
    }
}
