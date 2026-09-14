use std::fmt::{self, Debug};

use crate::memory::cursor::{Cursor, Error, Position, StructCursor};
use crate::memory::extent::Extent;
use crate::memory::inspect::{self, Inspect};

use super::{CoffHeader, Field};

/// A cursor over a COFF header.
#[derive(Clone)]
pub struct CoffHeaderCursor<'a>(StructCursor<'a, CoffHeader, Field>);

impl<'a> CoffHeaderCursor<'a> {
    /// Constructs a new COFF header cursor.
    pub(super) fn new(header: &'a CoffHeader) -> Self {
        Self(StructCursor::new(header))
    }
}

impl<'a> CoffHeaderCursor<'a> {
    /// Gets the header that this cursor is over.
    pub const fn header(&self) -> &'a CoffHeader {
        self.0.structure()
    }

    /// Gets the field that this cursor is at.
    pub fn field(&self) -> Option<Field> {
        self.0.field()
    }
}

impl Cursor for CoffHeaderCursor<'_> {
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

impl Inspect for CoffHeaderCursor<'_> {
    fn inspect(&self, inspector: &mut dyn inspect::Inspector) {
        let header = self.header();

        let Some(field) = self.field() else {
            return;
        };

        let value: &dyn inspect::InspectionValue = match field {
            Field::Machine => &header.machine,
            Field::NumberOfSections => &header.number_of_sections,
            Field::TimeDateStamp => &header.time_date_stamp,
            Field::PointerToSymbolTable => &header.pointer_to_symbol_table,
            Field::NumberOfSymbols => &header.number_of_symbols,
            Field::SizeOfOptionalHeader => &header.size_of_optional_header,
            Field::Characteristics => &header.characteristics,
        };

        inspector.record(field.address_space()).field(&field, value);
    }
}

impl Debug for CoffHeaderCursor<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CoffHeaderCursor")
            .field("position", &self.position())
            .field("field", &self.field())
            .finish()
    }
}
