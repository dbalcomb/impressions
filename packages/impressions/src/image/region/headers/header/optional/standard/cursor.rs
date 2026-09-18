use std::fmt::{self, Debug};

use crate::memory::cursor::{Cursor, Error, Position, StructCursor};
use crate::memory::extent::Extent;
use crate::memory::inspect::{Inspect, InspectionValue, Inspector};

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
    fn inspect(&self, inspector: &mut dyn Inspector) {
        let fields = self.fields();

        fields.inspect(inspector);

        let Some(field) = self.field() else {
            return;
        };

        let value: &dyn InspectionValue = match field {
            Field::Magic => &fields.magic,
            Field::MajorLinkerVersion => &fields.major_linker_version,
            Field::MinorLinkerVersion => &fields.minor_linker_version,
            Field::SizeOfCode => &fields.size_of_code,
            Field::SizeOfInitializedData => &fields.size_of_initialized_data,
            Field::SizeOfUninitializedData => &fields.size_of_uninitialized_data,
            Field::AddressOfEntryPoint => &fields.address_of_entry_point,
            Field::BaseOfCode => &fields.base_of_code,
            Field::BaseOfData => &fields.base_of_data,
        };

        inspector.record(field.address_space()).field(&field, value);
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
