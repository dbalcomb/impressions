use std::fmt::{self, Debug};

use crate::memory::cursor::{Cursor, Error, Position, StructCursor};
use crate::memory::inspect::{self, Inspect};

use super::{Field, SectionHeader};

/// A cursor over a section header.
#[derive(Clone)]
pub struct SectionHeaderCursor<'a>(StructCursor<'a, SectionHeader, Field>);

impl<'a> SectionHeaderCursor<'a> {
    /// Constructs a new section header cursor.
    pub(super) fn new(header: &'a SectionHeader) -> Self {
        Self(StructCursor::new(header))
    }
}

impl<'a> SectionHeaderCursor<'a> {
    /// Gets the header that this cursor is over.
    pub const fn header(&self) -> &'a SectionHeader {
        self.0.structure()
    }

    /// Gets the field that this cursor is at.
    pub fn field(&self) -> Option<Field> {
        self.0.field()
    }
}

impl<'a> Cursor for SectionHeaderCursor<'a> {
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

impl Inspect for SectionHeaderCursor<'_> {
    fn inspect(&self, inspector: &mut inspect::Inspector<'_>) -> Result<(), inspect::Error> {
        let output = &mut inspector.identified();
        let header = self.header();

        let Some(field) = self.field() else {
            return Ok(());
        };

        match field {
            Field::Name => writeln!(output, "{field}: {}", header.name),
            Field::VirtualSize => {
                writeln!(output, "{field}: {}", header.virtual_size)
            }
            Field::VirtualAddress => writeln!(output, "{field}: {}", header.virtual_address),
            Field::SizeOfRawData => writeln!(output, "{field}: {}", header.size_of_raw_data),
            Field::PointerToRawData => writeln!(output, "{field}: {}", header.pointer_to_raw_data),
            Field::PointerToRelocations => {
                writeln!(output, "{field}: {}", header.pointer_to_relocations)
            }
            Field::PointerToLinenumbers => {
                writeln!(output, "{field}: {}", header.pointer_to_linenumbers)
            }
            Field::NumberOfRelocations => {
                writeln!(output, "{field}: {}", header.number_of_relocations)
            }
            Field::NumberOfLinenumbers => {
                writeln!(output, "{field}: {}", header.number_of_linenumbers)
            }
            Field::Characteristics => writeln!(output, "{field}: {}", header.characteristics),
        }
    }
}

impl<'a> Debug for SectionHeaderCursor<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SectionHeaderCursor")
            .field("position", &self.position())
            .field("field", &self.field())
            .finish()
    }
}
