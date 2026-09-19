use std::fmt::{self, Debug};

use crate::memory::address::Address;
use crate::memory::cursor::{AsCursor, Cursor, Error, Position};
use crate::memory::extent::{Extent, FixedExtent};
use crate::memory::inspect::{Inspect, Inspector};
use crate::memory::region::types::segmented::{Segments, SegmentsCursor};

use super::Table;

/// A cursor over a null-terminated table.
pub struct TableCursor<'a, T>
where
    T: AsCursor,
{
    table: &'a Table<T>,
    position: Position,
    cursor: Option<SegmentsCursor<'a, T>>,
}

impl<'a, T> TableCursor<'a, T>
where
    T: FixedExtent + AsCursor,
{
    /// Constructs a new table cursor.
    pub(super) fn new(table: &'a Table<T>) -> Self {
        Self {
            table,
            position: Position::START,
            cursor: if !table.is_empty() {
                Some(SegmentsCursor::new(Segments::new(&table.0)))
            } else {
                None
            },
        }
    }
}

impl<'a, T> TableCursor<'a, T>
where
    T: AsCursor,
{
    /// Gets the table this cursor is over.
    pub const fn table(&self) -> &'a Table<T> {
        self.table
    }

    /// Gets the cursor over the table rows.
    pub const fn cursor(&self) -> Option<&SegmentsCursor<'a, T>> {
        self.cursor.as_ref()
    }
}

impl<T> TableCursor<'_, T>
where
    T: FixedExtent + AsCursor,
{
    fn rows_size(&self) -> u64 {
        self.table.len() as u64 * T::SIZE.get()
    }

    fn is_at_end(&self) -> bool {
        self.position() == Position::from(self.table.size())
    }
}

impl<'a, T> Cursor for TableCursor<'a, T>
where
    T: FixedExtent + AsCursor,
{
    type Error = <T::Cursor<'a> as Cursor>::Error;

    fn position(&self) -> Position {
        self.position
    }

    fn seek(&mut self, position: Position) -> Result<(), Self::Error> {
        if position.get() > self.table.size().get() {
            return Err(Error::OutOfBounds(position, self.table.size()).into());
        }

        self.position = position;

        if position.get() >= self.rows_size() {
            self.cursor = None;
        } else if let Some(cursor) = &mut self.cursor {
            cursor.seek(position)?;
        } else {
            let mut cursor = SegmentsCursor::new(Segments::new(&self.table.0));

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
        let end = Position::from(self.table.size());

        if self.position == end {
            return Ok(None);
        }

        if self.position.get() >= self.rows_size() {
            self.position = end;

            return Ok(None);
        }

        let cursor = self
            .cursor
            .as_mut()
            .expect("a table with stored rows has a segments cursor");

        cursor.next()?;

        self.position = cursor.position();

        if self.position.get() == self.rows_size() {
            self.cursor = None;
        }

        Ok(Some(self.position))
    }

    fn step(&mut self) -> Result<Option<Position>, Self::Error> {
        if self.is_at_end() {
            return Ok(None);
        }

        if self.position.get() >= self.rows_size() {
            return self.next();
        }

        let cursor = self
            .cursor
            .as_mut()
            .expect("a table with stored rows has a segments cursor");

        cursor.step()?;

        self.position = cursor.position();

        if self.position.get() == self.rows_size() {
            self.cursor = None;
        }

        Ok(Some(self.position))
    }
}

impl<'a, T> Inspect for TableCursor<'a, T>
where
    T: FixedExtent + AsCursor + Inspect,
    T::Cursor<'a>: Cursor + Inspect,
{
    fn inspect(&self, inspector: &mut dyn Inspector) {
        if self.is_at_end() {
            return;
        }

        if self.position.get() < self.rows_size() {
            self.cursor
                .as_ref()
                .expect("a table with stored rows has a segments cursor")
                .inspect(inspector);

            return;
        }

        let address = Address::new(self.rows_size() as u32);
        let mut inspector = inspector.at(address);

        inspector
            .record(T::SIZE.to_address_space())
            .vacant()
            .label(&"Null")
            .finish();
    }
}

impl<'a, T> Clone for TableCursor<'a, T>
where
    T: AsCursor<Cursor<'a>: Clone>,
{
    fn clone(&self) -> Self {
        Self {
            position: self.position,
            table: self.table,
            cursor: self.cursor.clone(),
        }
    }
}

impl<'a, T> Debug for TableCursor<'a, T>
where
    T: FixedExtent + AsCursor<Cursor<'a>: Debug>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TableCursor")
            .field("position", &self.position)
            .field("cursor", &self.cursor)
            .finish()
    }
}
