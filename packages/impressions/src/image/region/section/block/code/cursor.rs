use std::fmt::{self, Debug};

use crate::instruction::Instruction;
use crate::memory::cursor::Error;
use crate::memory::cursor::{Cursor, Position};
use crate::memory::inspect::{Inspect, Inspector};
use crate::memory::region::types::segmented::{Segments, SegmentsCursor};

use super::Code;

/// A cursor over a block of code.
pub struct CodeCursor<'a> {
    code: &'a Code,
    cursor: SegmentsCursor<'a, Instruction>,
}

impl<'a> CodeCursor<'a> {
    /// Constructs a new code cursor.
    pub(super) fn new(code: &'a Code) -> Self {
        Self {
            code,
            cursor: SegmentsCursor::new(Segments::new(&code.instructions)),
        }
    }
}

impl<'a> Cursor for CodeCursor<'a> {
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

impl Inspect for CodeCursor<'_> {
    fn inspect(&self, inspector: &mut dyn Inspector) {
        self.cursor.inspect(inspector);
    }
}

impl Clone for CodeCursor<'_> {
    fn clone(&self) -> Self {
        Self {
            code: self.code,
            cursor: self.cursor.clone(),
        }
    }
}

impl Debug for CodeCursor<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CodeCursor")
            .field("code", &self.code)
            .field("cursor", &self.cursor)
            .finish()
    }
}
