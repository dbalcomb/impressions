use std::fmt::{self, Debug};

use crate::memory::cursor::{Cursor, Error, Position, StructCursor};
use crate::memory::inspect::{self, Inspect};

use super::{DataDirectory, Field};

/// A cursor over a data directory entry.
#[derive(Clone)]
pub struct DataDirectoryCursor<'a>(StructCursor<'a, DataDirectory, Field>);

impl<'a> DataDirectoryCursor<'a> {
    /// Constructs a new data directory cursor.
    pub(super) fn new(directory: &'a DataDirectory) -> Self {
        Self(StructCursor::new(directory))
    }
}

impl<'a> DataDirectoryCursor<'a> {
    /// Gets the directory that this cursor is over.
    pub const fn directory(&self) -> &'a DataDirectory {
        self.0.structure()
    }

    /// Gets the field that this cursor is at.
    pub fn field(&self) -> Option<Field> {
        self.0.field()
    }
}

impl Cursor for DataDirectoryCursor<'_> {
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

impl Inspect for DataDirectoryCursor<'_> {
    fn inspect(&self, inspector: &mut inspect::Inspector<'_>) -> Result<(), inspect::Error> {
        let output = &mut inspector.identified();
        let directory = self.directory();

        let Some(field) = self.field() else {
            return Ok(());
        };

        match field {
            Field::VirtualAddress => writeln!(output, "{field}: {}", directory.virtual_address),
            Field::Size => writeln!(output, "{field}: {}", directory.size),
        }
    }
}

impl Debug for DataDirectoryCursor<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DataDirectoryCursor")
            .field("position", &self.position())
            .field("field", &self.field())
            .finish()
    }
}
