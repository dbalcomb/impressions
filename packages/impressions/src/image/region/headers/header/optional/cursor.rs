use std::fmt::{self, Debug};

use crate::memory::cursor::{AsCursor, Cursor, Error, Position};
use crate::memory::extent::{Extent, Size};
use crate::memory::inspect::{self, Inspect};

use super::OptionalHeader;
use super::directories::DataDirectoriesCursor;
use super::standard::StandardFieldsCursor;
use super::windows::WindowsFieldsCursor;

const WINDOWS_OFFSET: u32 = 28;
const DIRECTORIES_OFFSET: u32 = 96;

/// A cursor over an Optional header.
#[derive(Clone)]
pub struct OptionalHeaderCursor<'a> {
    header: &'a OptionalHeader,
    cursor: FieldsCursor<'a>,
}

impl<'a> OptionalHeaderCursor<'a> {
    /// Constructs a new Optional header cursor.
    pub(super) fn new(header: &'a OptionalHeader) -> Self {
        Self {
            header,
            cursor: FieldsCursor::Standard(header.standard.cursor()),
        }
    }
}

impl<'a> OptionalHeaderCursor<'a> {
    /// Gets the Optional header that this cursor is over.
    pub const fn header(&self) -> &'a OptionalHeader {
        self.header
    }
}

impl<'a> OptionalHeaderCursor<'a> {
    /// Gets the inner cursor.
    fn cursor(&self) -> &dyn Cursor<Error = Error> {
        match &self.cursor {
            FieldsCursor::Standard(cursor) => cursor,
            FieldsCursor::Windows(cursor) => cursor,
            FieldsCursor::Directories(cursor) => cursor,
        }
    }

    /// Gets the inner cursor as mutable.
    fn cursor_mut(&mut self) -> &mut dyn Cursor<Error = Error> {
        match &mut self.cursor {
            FieldsCursor::Standard(cursor) => cursor,
            FieldsCursor::Windows(cursor) => cursor,
            FieldsCursor::Directories(cursor) => cursor,
        }
    }

    /// Sets the inner cursor based on the given position.
    fn set_cursor(&mut self, position: Position) {
        let offset = position.get_addressable().expect("not at header end");

        self.cursor = if offset < WINDOWS_OFFSET {
            FieldsCursor::Standard(self.header.standard.cursor())
        } else if offset < DIRECTORIES_OFFSET {
            FieldsCursor::Windows(self.header.windows.cursor())
        } else {
            FieldsCursor::Directories(
                self.header
                    .data_directories
                    .as_ref()
                    .expect("only optional headers with directories extend past Windows fields")
                    .cursor(),
            )
        };
    }

    /// Gets the current size of the target field.
    fn current_size(&self) -> Size {
        match &self.cursor {
            FieldsCursor::Standard(cursor) => cursor.fields().size(),
            FieldsCursor::Windows(cursor) => cursor.fields().size(),
            FieldsCursor::Directories(cursor) => cursor.directories().size(),
        }
    }
}

impl Cursor for OptionalHeaderCursor<'_> {
    type Error = Error;

    fn position(&self) -> Position {
        let position = self.cursor().position();

        match position.get_addressable() {
            Some(position) => Position::new(self.cursor.offset() + position),
            None => Position::new(
                self.cursor.offset()
                    + self
                        .current_size()
                        .get_addressable()
                        .expect("optional header components fit in the address space"),
            ),
        }
    }

    fn seek(&mut self, position: Position) -> Result<(), Self::Error> {
        let size = self.header.size();

        if position.get() > size.get() {
            return Err(Error::OutOfBounds(position, size));
        }

        if position == Position::from(size) {
            self.cursor = match self.header.data_directories.as_ref() {
                Some(directories) => FieldsCursor::Directories(directories.cursor()),
                None => FieldsCursor::Windows(self.header.windows.cursor()),
            };

            let size = self.current_size();

            return self.cursor_mut().seek(Position::from(size));
        }

        self.set_cursor(position);

        let local = Position::new(
            position
                .get_addressable()
                .expect("in-bounds position before the end is addressable")
                - self.cursor.offset(),
        );

        self.cursor_mut().seek(local)
    }

    fn advance(&mut self, offset: u32) -> Result<(), Self::Error> {
        let position = self
            .position()
            .checked_add(offset)
            .ok_or(Error::CannotAdvance(self.position(), offset))?;

        self.seek(position)
    }

    fn next(&mut self) -> Result<Option<Position>, Self::Error> {
        if self.position() == Position::from(self.header.size()) {
            return Ok(None);
        }

        let position = match &self.cursor {
            FieldsCursor::Standard(_) => Position::new(WINDOWS_OFFSET),
            FieldsCursor::Windows(_) => match self.header.data_directories.as_ref() {
                Some(_) => Position::new(DIRECTORIES_OFFSET),
                None => Position::from(self.header.size()),
            },
            FieldsCursor::Directories(_) => Position::from(self.header.size()),
        };

        self.seek(position)?;

        match position == Position::from(self.header.size()) {
            true => Ok(None),
            false => Ok(Some(position)),
        }
    }

    fn step(&mut self) -> Result<Option<Position>, Self::Error> {
        if self.position() == Position::from(self.header.size()) {
            return Ok(None);
        }

        let offset = self.cursor.offset();

        if let Some(position) = self.cursor_mut().step()? {
            return Ok(Some(Position::new(
                offset
                    + position
                        .get_addressable()
                        .expect("a cursor only returns addressable positions from step"),
            )));
        }

        self.next()
    }
}

impl Inspect for OptionalHeaderCursor<'_> {
    fn inspect(&self, inspector: &mut inspect::Inspector<'_>) -> Result<(), inspect::Error> {
        match &self.cursor {
            FieldsCursor::Standard(cursor) => cursor.inspect(inspector),
            FieldsCursor::Windows(cursor) => cursor.inspect(inspector),
            FieldsCursor::Directories(cursor) => cursor.inspect(inspector),
        }
    }
}

impl Debug for OptionalHeaderCursor<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("OptionalHeaderCursor")
            .field("position", &self.position())
            .field("cursor", &self.cursor)
            .finish()
    }
}

/// The inner cursor over the fields of an Optional header.
#[derive(Clone)]
enum FieldsCursor<'a> {
    Standard(StandardFieldsCursor<'a>),
    Windows(WindowsFieldsCursor<'a>),
    Directories(DataDirectoriesCursor<'a>),
}

impl FieldsCursor<'_> {
    /// Gets the offset of the cursor within the Optional header.
    fn offset(&self) -> u32 {
        match self {
            Self::Standard(_) => 0,
            Self::Windows(_) => WINDOWS_OFFSET,
            Self::Directories(_) => DIRECTORIES_OFFSET,
        }
    }
}

impl Debug for FieldsCursor<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Standard(cursor) => Debug::fmt(cursor, f),
            Self::Windows(cursor) => Debug::fmt(cursor, f),
            Self::Directories(cursor) => Debug::fmt(cursor, f),
        }
    }
}
