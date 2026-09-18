use std::fmt::{self, Debug};

use crate::memory::cursor::{Cursor, Error, Position};
use crate::memory::inspect::{Inspect, Inspector};
use crate::memory::segmented::{Segmented, SegmentsCursor};

use super::{ImportLookup, ImportLookupTable};

/// A cursor over import lookup table entries.
#[derive(Clone)]
pub struct ImportLookupTableCursor<'a> {
    table: &'a ImportLookupTable,
    cursor: SegmentsCursor<'a, ImportLookup>,
}

impl<'a> ImportLookupTableCursor<'a> {
    /// Constructs a new import lookup table cursor.
    pub(super) fn new(table: &'a ImportLookupTable) -> Self {
        Self {
            table,
            cursor: SegmentsCursor::new(table.segments()),
        }
    }
}

impl<'a> ImportLookupTableCursor<'a> {
    /// Gets the table that this cursor is over.
    pub const fn table(&self) -> &'a ImportLookupTable {
        self.table
    }
}

impl Cursor for ImportLookupTableCursor<'_> {
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

impl Inspect for ImportLookupTableCursor<'_> {
    fn inspect(&self, inspector: &mut dyn Inspector) {
        self.table().inspect(inspector);
        self.cursor.inspect(inspector);
    }
}

impl Debug for ImportLookupTableCursor<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ImportLookupTableCursor")
            .field("position", &self.position())
            .field("cursor", self.cursor.cursor())
            .finish()
    }
}
