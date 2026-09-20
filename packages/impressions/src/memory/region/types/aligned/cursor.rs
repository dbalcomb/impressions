use std::fmt::{self, Debug};

use crate::memory::address::Address;
use crate::memory::cursor::{AsCursor, Cursor, Error, Position};
use crate::memory::extent::{Extent, Size};
use crate::memory::inspect::{Inspect, Inspector};

use super::Aligned;

/// A cursor over an aligned region and its trailing padding.
pub struct AlignedCursor<'a, T, const ALIGNMENT: u32>
where
    T: AsCursor,
{
    aligned: &'a Aligned<T, ALIGNMENT>,
    cursor: Option<T::Cursor<'a>>,
    position: Position,
}

impl<'a, T, const ALIGNMENT: u32> AlignedCursor<'a, T, ALIGNMENT>
where
    T: AsCursor + Extent,
{
    pub(super) fn new(aligned: &'a Aligned<T, ALIGNMENT>) -> Self {
        Self {
            aligned,
            cursor: Some(aligned.region().cursor()),
            position: Position::START,
        }
    }

    fn region_size(&self) -> u64 {
        self.aligned.region().size().get()
    }
}

impl<'a, T, const ALIGNMENT: u32> Cursor for AlignedCursor<'a, T, ALIGNMENT>
where
    T: AsCursor + Extent,
{
    type Error = <T::Cursor<'a> as Cursor>::Error;

    fn position(&self) -> Position {
        self.position
    }

    fn seek(&mut self, position: Position) -> Result<(), Self::Error> {
        if position.get() > self.aligned.size().get() {
            return Err(Error::OutOfBounds(position, self.aligned.size()).into());
        }

        self.position = position;

        if position.get() >= self.region_size() {
            self.cursor = None;
        } else if let Some(cursor) = &mut self.cursor {
            cursor.seek(position)?;
        } else {
            let mut cursor = self.aligned.region().cursor();

            cursor.seek(position)?;

            self.cursor = Some(cursor);
        }

        Ok(())
    }

    fn advance(&mut self, offset: u32) -> Result<(), Self::Error> {
        let position = self
            .position
            .checked_add(offset)
            .ok_or(Error::CannotAdvance(self.position, offset))?;

        self.seek(position)
    }

    fn next(&mut self) -> Result<Option<Position>, Self::Error> {
        let end = Position::from(self.aligned.size());

        if self.position == end {
            return Ok(None);
        }

        if self.position.get() >= self.region_size() {
            self.position = end;

            return Ok(None);
        }

        let cursor = self.cursor.as_mut().expect("inner cursor is present");

        cursor.next()?;

        self.position = cursor.position();

        if self.position.get() == self.region_size() {
            self.cursor = None;

            if self.aligned.padding_size() == 0 {
                self.position = end;

                return Ok(None);
            }
        }

        Ok(Some(self.position))
    }

    fn step(&mut self) -> Result<Option<Position>, Self::Error> {
        if self.position.get() >= self.region_size() {
            return self.next();
        }

        let cursor = self
            .cursor
            .as_mut()
            .expect("inner cursor is present before region size");

        cursor.step()?;

        self.position = cursor.position();

        if self.position.get() == self.region_size() {
            self.cursor = None;

            if self.aligned.padding_size() == 0 {
                self.position = Position::from(self.aligned.size());

                return Ok(None);
            }
        }

        Ok(Some(self.position))
    }
}

impl<'a, T, const ALIGNMENT: u32> Inspect for AlignedCursor<'a, T, ALIGNMENT>
where
    T: AsCursor<Cursor<'a>: Inspect> + Extent + Inspect,
{
    fn inspect(&self, inspector: &mut dyn Inspector) {
        if self.position == Position::from(self.aligned.size()) {
            return;
        }

        if self.position.get() < self.region_size() {
            self.aligned.region().inspect(inspector);
            self.cursor
                .as_ref()
                .expect("inner cursor is present")
                .inspect(inspector);

            return;
        }

        if self.aligned.padding_size() != 0 {
            let mut inspector = inspector.at(Address::new(self.region_size() as u32));

            inspector
                .record(Size::new_valid(self.aligned.padding_size()).to_address_space())
                .vacant()
                .label(&"Padding")
                .finish();
        }
    }
}

impl<'a, T, const ALIGNMENT: u32> Clone for AlignedCursor<'a, T, ALIGNMENT>
where
    T: AsCursor<Cursor<'a>: Clone>,
{
    fn clone(&self) -> Self {
        Self {
            aligned: self.aligned,
            cursor: self.cursor.clone(),
            position: self.position,
        }
    }
}

impl<'a, T, const ALIGNMENT: u32> Debug for AlignedCursor<'a, T, ALIGNMENT>
where
    T: AsCursor<Cursor<'a>: Debug> + Extent,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AlignedCursor")
            .field("position", &self.position)
            .field("cursor", &self.cursor)
            .finish()
    }
}
