use std::fmt::{self, Debug};

use crate::memory::cursor::{Cursor, Error, Position};
use crate::memory::inspect::{Inspect, Inspector};
use crate::memory::region::types::segmented::{Segmented, SegmentsCursor};

use super::ImportAddressTable;
use crate::image::region::section::block::meta::import_lookup_table::entry::ImportLookup;

/// A cursor over import address table entries.
#[derive(Clone)]
pub struct ImportAddressTableCursor<'a> {
    table: &'a ImportAddressTable,
    cursor: SegmentsCursor<'a, ImportLookup>,
}

impl<'a> ImportAddressTableCursor<'a> {
    /// Constructs a new import address table cursor.
    pub(super) fn new(table: &'a ImportAddressTable) -> Self {
        Self {
            table,
            cursor: SegmentsCursor::new(table.segments()),
        }
    }
}

impl<'a> ImportAddressTableCursor<'a> {
    /// Gets the table that this cursor is over.
    pub const fn table(&self) -> &'a ImportAddressTable {
        self.table
    }
}

impl Cursor for ImportAddressTableCursor<'_> {
    type Error = Error;

    fn position(&self) -> Position {
        self.cursor.position()
    }

    fn seek(&mut self, position: Position) -> Result<(), Self::Error> {
        self.cursor.seek(position)
    }

    fn advance(&mut self, offset: u32) -> Result<(), Self::Error> {
        self.cursor.advance(offset)
    }

    fn next(&mut self) -> Result<Option<Position>, Self::Error> {
        self.cursor.next()
    }

    fn step(&mut self) -> Result<Option<Position>, Self::Error> {
        self.cursor.step()
    }
}

impl Inspect for ImportAddressTableCursor<'_> {
    fn inspect(&self, inspector: &mut dyn Inspector) {
        self.table().inspect(inspector);
        self.cursor.inspect(inspector);
    }
}

impl Debug for ImportAddressTableCursor<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ImportAddressTableCursor")
            .field("position", &self.position())
            .field("cursor", self.cursor.cursor())
            .finish()
    }
}
