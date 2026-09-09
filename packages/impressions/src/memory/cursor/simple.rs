use crate::memory::extent::Extent;

use super::{Cursor, Error, Position};

/// A simple cursor over a region of memory.
#[derive(Debug)]
pub struct SimpleCursor<'a, T> {
    position: Position,
    region: &'a T,
}

impl<'a, T> SimpleCursor<'a, T>
where
    T: Extent,
{
    /// Constructs a new simple cursor.
    pub const fn new(region: &'a T) -> Self {
        Self {
            position: Position::START,
            region,
        }
    }
}

impl<'a, T> SimpleCursor<'a, T> {
    /// Gets the region of memory that this cursor is over.
    pub const fn region(&self) -> &'a T {
        self.region
    }
}

impl<T> Cursor for SimpleCursor<'_, T>
where
    T: Extent,
{
    type Error = Error;

    fn position(&self) -> Position {
        self.position
    }

    fn seek(&mut self, position: Position) -> Result<(), Self::Error> {
        if position.get() > self.region.size().get() {
            return Err(Error::OutOfBounds(position, self.region.size()));
        }

        self.position = position;

        Ok(())
    }

    fn advance(&mut self, offset: u32) -> Result<(), Self::Error> {
        let Some(position) = self.position.checked_add(offset) else {
            return Err(Error::CannotAdvance(self.position, offset));
        };

        self.seek(position)
    }

    fn next(&mut self) -> Result<Option<Position>, Self::Error> {
        self.seek(self.region.size().into())?;

        Ok(None)
    }

    fn step(&mut self) -> Result<Option<Position>, Self::Error> {
        self.next()
    }
}

impl<T> Clone for SimpleCursor<'_, T> {
    fn clone(&self) -> Self {
        Self {
            position: self.position,
            region: self.region,
        }
    }
}
