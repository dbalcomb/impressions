use std::fmt::{self, Debug};

use crate::memory::cursor::{Cursor, Error, Position, SimpleCursor};

use super::Block;

/// A cursor over a block in a section.
#[derive(Clone)]
pub struct BlockCursor<'a>(SimpleCursor<'a, Block>);

impl<'a> BlockCursor<'a> {
    /// Constructs a new block cursor.
    pub(super) fn new(block: &'a Block) -> Self {
        Self(SimpleCursor::new(block))
    }
}

impl<'a> Cursor for BlockCursor<'a> {
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

impl Debug for BlockCursor<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BlockCursor")
            .field("position", &self.position())
            .field("region", self.0.region())
            .finish()
    }
}
