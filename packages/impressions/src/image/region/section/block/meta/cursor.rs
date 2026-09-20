use std::fmt::{self, Debug};

use crate::memory::cursor::{AsCursor, Cursor, Error, Position};
use crate::memory::inspect::{Inspect, Inspector};

use super::Meta;
use super::hint_name_table::HintNameTableCursor;
use super::import_address_table::ImportAddressTableCursor;
use super::import_directory_table::ImportDirectoryTableCursor;
use super::import_lookup_table::ImportLookupTableCursor;
use super::import_name::ImportNameCursor;

/// A cursor over a block of metadata.
#[derive(Clone)]
pub enum MetaCursor<'a> {
    ImportDirectoryTable(ImportDirectoryTableCursor<'a>),
    ImportLookupTable(ImportLookupTableCursor<'a>),
    ImportAddressTable(ImportAddressTableCursor<'a>),
    HintNameTable(HintNameTableCursor<'a>),
    ImportName(ImportNameCursor<'a>),
}

impl<'a> MetaCursor<'a> {
    /// Constructs a new metadata cursor.
    pub(super) fn new(meta: &'a Meta) -> Self {
        match meta {
            Meta::ImportDirectoryTable(table) => Self::ImportDirectoryTable(table.cursor()),
            Meta::ImportLookupTable(table) => Self::ImportLookupTable(table.cursor()),
            Meta::ImportAddressTable(table) => Self::ImportAddressTable(table.cursor()),
            Meta::HintNameTable(table) => Self::HintNameTable(table.cursor()),
            Meta::ImportName(name) => Self::ImportName(name.cursor()),
        }
    }
}

impl Cursor for MetaCursor<'_> {
    type Error = Error;

    fn position(&self) -> Position {
        match self {
            Self::ImportDirectoryTable(cursor) => cursor.position(),
            Self::ImportLookupTable(cursor) => cursor.position(),
            Self::ImportAddressTable(cursor) => cursor.position(),
            Self::HintNameTable(cursor) => cursor.position(),
            Self::ImportName(cursor) => cursor.position(),
        }
    }

    fn seek(&mut self, position: Position) -> Result<(), Self::Error> {
        match self {
            Self::ImportDirectoryTable(cursor) => cursor.seek(position),
            Self::ImportLookupTable(cursor) => cursor.seek(position),
            Self::ImportAddressTable(cursor) => cursor.seek(position),
            Self::HintNameTable(cursor) => cursor.seek(position),
            Self::ImportName(cursor) => cursor.seek(position),
        }
    }

    fn advance(&mut self, offset: u32) -> Result<(), Self::Error> {
        match self {
            Self::ImportDirectoryTable(cursor) => cursor.advance(offset),
            Self::ImportLookupTable(cursor) => cursor.advance(offset),
            Self::ImportAddressTable(cursor) => cursor.advance(offset),
            Self::HintNameTable(cursor) => cursor.advance(offset),
            Self::ImportName(cursor) => cursor.advance(offset),
        }
    }

    fn next(&mut self) -> Result<Option<Position>, Self::Error> {
        match self {
            Self::ImportDirectoryTable(cursor) => cursor.next(),
            Self::ImportLookupTable(cursor) => cursor.next(),
            Self::ImportAddressTable(cursor) => cursor.next(),
            Self::HintNameTable(cursor) => cursor.next(),
            Self::ImportName(cursor) => cursor.next(),
        }
    }

    fn step(&mut self) -> Result<Option<Position>, Self::Error> {
        match self {
            Self::ImportDirectoryTable(cursor) => cursor.step(),
            Self::ImportLookupTable(cursor) => cursor.step(),
            Self::ImportAddressTable(cursor) => cursor.step(),
            Self::HintNameTable(cursor) => cursor.step(),
            Self::ImportName(cursor) => cursor.step(),
        }
    }
}

impl Inspect for MetaCursor<'_> {
    fn inspect(&self, inspector: &mut dyn Inspector) {
        match self {
            Self::ImportDirectoryTable(cursor) => cursor.inspect(inspector),
            Self::ImportLookupTable(cursor) => cursor.inspect(inspector),
            Self::ImportAddressTable(cursor) => cursor.inspect(inspector),
            Self::HintNameTable(cursor) => cursor.inspect(inspector),
            Self::ImportName(cursor) => cursor.inspect(inspector),
        }
    }
}

impl Debug for MetaCursor<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ImportDirectoryTable(cursor) => Debug::fmt(cursor, f),
            Self::ImportLookupTable(cursor) => Debug::fmt(cursor, f),
            Self::ImportAddressTable(cursor) => Debug::fmt(cursor, f),
            Self::HintNameTable(cursor) => Debug::fmt(cursor, f),
            Self::ImportName(cursor) => Debug::fmt(cursor, f),
        }
    }
}
