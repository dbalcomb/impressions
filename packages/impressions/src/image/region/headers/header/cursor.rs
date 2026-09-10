use std::fmt::{self, Debug};

use crate::memory::cursor::{Cursor, Error, Position, SimpleCursor};
use crate::memory::inspect::{self, Inspect};

use super::Header;

/// A cursor over a single header.
#[derive(Clone)]
pub struct HeaderCursor<'a>(SimpleCursor<'a, Header>);

impl<'a> HeaderCursor<'a> {
    /// Constructs a new header cursor.
    pub(super) fn new(header: &'a Header) -> Self {
        Self(SimpleCursor::new(header))
    }
}

impl<'a> Cursor for HeaderCursor<'a> {
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

impl Inspect for HeaderCursor<'_> {
    fn inspect(&self, inspector: &mut inspect::Inspector<'_>) -> Result<(), inspect::Error> {
        self.0.inspect(inspector)
    }
}

impl Debug for HeaderCursor<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("HeaderCursor")
            .field("position", &self.position())
            .field("region", self.0.region())
            .finish()
    }
}
