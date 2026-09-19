use std::fmt::{self, Debug};

use crate::memory::cursor::{Cursor, Error, Position};
use crate::memory::inspect::{Inspect, Inspector};
use crate::memory::region::types::segmented::{Segmented, SegmentsCursor};

use super::HintNameTable;
use super::entry::HintName;

/// A cursor over the hint/name table.
#[derive(Clone)]
pub struct HintNameTableCursor<'a>(SegmentsCursor<'a, HintName>);

impl<'a> HintNameTableCursor<'a> {
    /// Constructs a new section cursor.
    pub(super) fn new(table: &'a HintNameTable) -> Self {
        Self(SegmentsCursor::new(table.segments()))
    }
}

impl<'a> HintNameTableCursor<'a> {
    /// Gets the entry at the cursor position.
    pub const fn entry(&self) -> &'a HintName {
        self.0.segment()
    }
}

impl Cursor for HintNameTableCursor<'_> {
    type Error = Error;

    fn position(&self) -> Position {
        self.0.position()
    }

    fn seek(&mut self, position: Position) -> Result<(), Self::Error> {
        self.0.seek(position)
    }

    fn advance(&mut self, offset: u32) -> Result<(), Self::Error> {
        self.0.advance(offset)
    }

    fn next(&mut self) -> Result<Option<Position>, Self::Error> {
        self.0.next()
    }

    fn step(&mut self) -> Result<Option<Position>, Self::Error> {
        self.0.step()
    }
}

impl Inspect for HintNameTableCursor<'_> {
    fn inspect(&self, inspector: &mut dyn Inspector) {
        self.0.inspect(inspector)
    }
}

impl Debug for HintNameTableCursor<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("HintNameTableCursor")
            .field("position", &self.position())
            .field("cursor", self.0.cursor())
            .finish()
    }
}
