use std::fmt::{self, Debug};

use crate::memory::cursor::{Cursor, Error, Position};
use crate::memory::inspect::{Inspect, Inspector};
use crate::memory::region::types::segmented::{Segmented, SegmentsCursor};

use super::{ImportDirectory, ImportDirectoryTable};

/// A cursor over import directory table entries.
#[derive(Clone)]
pub struct ImportDirectoryTableCursor<'a> {
    table: &'a ImportDirectoryTable,
    cursor: SegmentsCursor<'a, ImportDirectory>,
}

impl<'a> ImportDirectoryTableCursor<'a> {
    /// Constructs a new import directory table cursor.
    pub(super) fn new(table: &'a ImportDirectoryTable) -> Self {
        Self {
            table,
            cursor: SegmentsCursor::new(table.segments()),
        }
    }
}

impl<'a> ImportDirectoryTableCursor<'a> {
    /// Gets the table that this cursor is over.
    pub const fn table(&self) -> &'a ImportDirectoryTable {
        self.table
    }
}

impl Cursor for ImportDirectoryTableCursor<'_> {
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

impl Inspect for ImportDirectoryTableCursor<'_> {
    fn inspect(&self, inspector: &mut dyn Inspector) {
        self.table().inspect(inspector);
        self.cursor.inspect(inspector);
    }
}

impl Debug for ImportDirectoryTableCursor<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ImportDirectoryTableCursor")
            .field("position", &self.position())
            .field("cursor", self.cursor.cursor())
            .finish()
    }
}
