use std::fmt::{self, Debug};

use crate::image::Padding;
use crate::memory::cursor::{AsCursor, Cursor, Error, Position, SimpleCursor};
use crate::memory::inspect::{self, Inspect};

use super::Block;

/// A cursor over a block in a section.
#[derive(Clone)]
pub enum BlockCursor<'a> {
    Padding(SimpleCursor<'a, Padding>),
}

impl<'a> BlockCursor<'a> {
    /// Constructs a new block cursor.
    pub(super) fn new(block: &'a Block) -> Self {
        match block {
            Block::Padding(padding) => Self::Padding(padding.cursor()),
        }
    }
}

impl<'a> BlockCursor<'a> {
    /// Gets the block cursor as a padding cursor.
    pub const fn as_padding(&self) -> Option<&SimpleCursor<'a, Padding>> {
        match self {
            Self::Padding(cursor) => Some(cursor),
        }
    }
}

impl<'a> Cursor for BlockCursor<'a> {
    type Error = Error;

    fn position(&self) -> Position {
        match self {
            Self::Padding(cursor) => cursor.position(),
        }
    }

    fn seek(&mut self, position: Position) -> Result<(), Self::Error> {
        match self {
            Self::Padding(cursor) => cursor.seek(position),
        }
    }

    fn advance(&mut self, offset: u32) -> Result<(), Self::Error> {
        match self {
            Self::Padding(cursor) => cursor.advance(offset),
        }
    }

    fn next(&mut self) -> Result<Option<Position>, Self::Error> {
        match self {
            Self::Padding(cursor) => cursor.next(),
        }
    }

    fn step(&mut self) -> Result<Option<Position>, Self::Error> {
        match self {
            Self::Padding(cursor) => cursor.step(),
        }
    }
}

impl Inspect for BlockCursor<'_> {
    fn inspect(&self, inspector: &mut inspect::Inspector<'_>) -> Result<(), inspect::Error> {
        match self {
            Self::Padding(cursor) => cursor.inspect(inspector),
        }
    }
}

impl Debug for BlockCursor<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Padding(cursor) => Debug::fmt(cursor, f),
        }
    }
}
