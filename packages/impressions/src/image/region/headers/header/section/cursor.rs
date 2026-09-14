use std::fmt::{self, Debug};

use crate::memory::cursor::{Cursor, Error, Position, StructCursor};
use crate::memory::extent::Extent;
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
    fn inspect(&self, inspector: &mut dyn inspect::Inspector) {
        let header = self.header();

        let Some(field) = self.field() else {
            return;
        };

        let value: &dyn inspect::InspectionValue = match field {
            Field::Name => &header.name,
            Field::VirtualSize => &header.virtual_size,
            Field::VirtualAddress => &header.virtual_address,
            Field::SizeOfRawData => &header.size_of_raw_data,
            Field::PointerToRawData => &header.pointer_to_raw_data,
            Field::PointerToRelocations => &header.pointer_to_relocations,
            Field::PointerToLinenumbers => &header.pointer_to_linenumbers,
            Field::NumberOfRelocations => &header.number_of_relocations,
            Field::NumberOfLinenumbers => &header.number_of_linenumbers,
            Field::Characteristics => &header.characteristics,
        };

        inspector.record(field.address_space()).field(&field, value);
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
