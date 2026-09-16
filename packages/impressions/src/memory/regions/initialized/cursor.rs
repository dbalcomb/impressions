use std::fmt::{self, Debug};

use crate::data::parse::Parse;
use crate::memory::cursor::{Cursor, Error, Position, Read, ReadError, SimpleCursor};
use crate::memory::extent::Extent;

use super::Initialized;

/// A cursor over an initialized region of memory.
#[derive(Clone)]
pub struct InitializedCursor<'a>(SimpleCursor<'a, Initialized>);

impl<'a> InitializedCursor<'a> {
    /// Constructs a new initialized cursor.
    pub const fn new(region: &'a Initialized) -> Self {
        Self(SimpleCursor::new(region))
    }
}

impl<'a> InitializedCursor<'a> {
    /// Gets the initialized region.
    pub const fn region(&self) -> &'a Initialized {
        self.0.region()
    }
}

impl Cursor for InitializedCursor<'_> {
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

impl Read for InitializedCursor<'_> {
    fn read_with<'a, T>(
        &mut self,
        context: T::Context<'a>,
    ) -> Result<T, ReadError<T::Error, Self::Error>>
    where
        T: Extent + Parse,
    {
        let bytes = self.region().bytes();
        let buffer = &bytes[self.position().get() as usize..];
        let region = T::parse_with(buffer, context).map_err(ReadError::Parse)?;

        if let Some(offset) = region.size().get_addressable() {
            self.advance(offset).map_err(ReadError::Cursor)?;
        } else {
            self.next().map_err(ReadError::Cursor)?;
        }

        Ok(region)
    }
}

impl Debug for InitializedCursor<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("InitializedCursor")
            .field("position", &self.position())
            .field("region", self.0.region())
            .finish()
    }
}
