use std::fmt::{self, Debug};

use crate::memory::cursor::{Cursor, Error, Position, StructCursor};
use crate::memory::extent::Extent;
use crate::memory::inspect::{Inspect, InspectionValue, Inspector};

use super::{Field, ImportDirectory};

/// A cursor over an import directory entry.
#[derive(Clone)]
pub struct ImportDirectoryCursor<'a>(StructCursor<'a, ImportDirectory, Field>);

impl<'a> ImportDirectoryCursor<'a> {
    /// Constructs a new import directory cursor.
    pub(super) fn new(directory: &'a ImportDirectory) -> Self {
        Self(StructCursor::new(directory))
    }
}

impl<'a> ImportDirectoryCursor<'a> {
    /// Gets the directory that this cursor is over.
    pub const fn directory(&self) -> &'a ImportDirectory {
        self.0.structure()
    }

    /// Gets the field that this cursor is at.
    pub fn field(&self) -> Option<Field> {
        self.0.field()
    }
}

impl Cursor for ImportDirectoryCursor<'_> {
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

impl Inspect for ImportDirectoryCursor<'_> {
    fn inspect(&self, inspector: &mut dyn Inspector) {
        let directory = self.directory();

        let Some(field) = self.field() else {
            return;
        };

        let value: &dyn InspectionValue = match field {
            Field::LookupTableAddress => &directory.lookup_table_address,
            Field::Timestamp => &directory.timestamp,
            Field::ForwarderChain => &directory.forwarder_chain,
            Field::NameAddress => &directory.name_address,
            Field::AddressTableAddress => &directory.address_table_address,
        };

        inspector.record(field.address_space()).field(&field, value);
    }
}

impl Debug for ImportDirectoryCursor<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ImportDirectoryCursor")
            .field("position", &self.position())
            .field("field", &self.field())
            .finish()
    }
}
