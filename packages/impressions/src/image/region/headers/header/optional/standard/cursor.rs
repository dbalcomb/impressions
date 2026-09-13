use std::fmt::{self, Debug};

use crate::memory::cursor::{Cursor, Error, Position, StructCursor};
use crate::memory::inspect::{self, Inspect};

use super::{Field, StandardFields};

/// A cursor over the standard fields of an Optional header.
#[derive(Clone)]
pub struct StandardFieldsCursor<'a>(StructCursor<'a, StandardFields, Field>);

impl<'a> StandardFieldsCursor<'a> {
    /// Constructs a new standard fields cursor.
    pub(super) fn new(fields: &'a StandardFields) -> Self {
        Self(StructCursor::new(fields))
    }
}

impl<'a> StandardFieldsCursor<'a> {
    /// Gets the standard fields that this cursor is over.
    pub const fn fields(&self) -> &'a StandardFields {
        self.0.structure()
    }

    /// Gets the field that this cursor is at.
    pub fn field(&self) -> Option<Field> {
        self.0.field()
    }
}

impl Cursor for StandardFieldsCursor<'_> {
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

impl Inspect for StandardFieldsCursor<'_> {
    fn inspect(&self, inspector: &mut inspect::Inspector<'_>) -> Result<(), inspect::Error> {
        let output = &mut inspector.identified();
        let fields = self.fields();

        let Some(field) = self.field() else {
            return Ok(());
        };

        match field {
            Field::Magic => writeln!(output, "{field}: {}", fields.magic),
            Field::MajorLinkerVersion => {
                writeln!(output, "{field}: {}", fields.major_linker_version)
            }
            Field::MinorLinkerVersion => {
                writeln!(output, "{field}: {}", fields.minor_linker_version)
            }
            Field::SizeOfCode => writeln!(output, "{field}: {}", fields.size_of_code),
            Field::SizeOfInitializedData => {
                writeln!(output, "{field}: {}", fields.size_of_initialized_data)
            }
            Field::SizeOfUninitializedData => {
                writeln!(output, "{field}: {}", fields.size_of_uninitialized_data)
            }
            Field::AddressOfEntryPoint => {
                writeln!(output, "{field}: {}", fields.address_of_entry_point)
            }
            Field::BaseOfCode => writeln!(output, "{field}: {}", fields.base_of_code),
            Field::BaseOfData => writeln!(output, "{field}: {}", fields.base_of_data),
        }
    }
}

impl Debug for StandardFieldsCursor<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("StandardFieldsCursor")
            .field("position", &self.position())
            .field("field", &self.field())
            .finish()
    }
}
