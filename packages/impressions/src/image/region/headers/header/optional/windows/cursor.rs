use std::fmt::{self, Debug};

use crate::memory::cursor::{Cursor, Error, Position, StructCursor};
use crate::memory::extent::Extent;
use crate::memory::inspect::{Inspect, InspectionValue, Inspector};

use super::{Field, WindowsFields};

/// A cursor over the Windows-specific fields of an Optional header.
#[derive(Clone)]
pub struct WindowsFieldsCursor<'a>(StructCursor<'a, WindowsFields, Field>);

impl<'a> WindowsFieldsCursor<'a> {
    /// Constructs a new Windows fields cursor.
    pub(super) fn new(fields: &'a WindowsFields) -> Self {
        Self(StructCursor::new(fields))
    }
}

impl<'a> WindowsFieldsCursor<'a> {
    /// Gets the Windows fields that this cursor is over.
    pub const fn fields(&self) -> &'a WindowsFields {
        self.0.structure()
    }

    /// Gets the field that this cursor is at.
    pub fn field(&self) -> Option<Field> {
        self.0.field()
    }
}

impl Cursor for WindowsFieldsCursor<'_> {
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

impl Inspect for WindowsFieldsCursor<'_> {
    fn inspect(&self, inspector: &mut dyn Inspector) {
        let fields = self.fields();

        fields.inspect(inspector);

        let Some(field) = self.field() else {
            return;
        };

        let value: &dyn InspectionValue = match field {
            Field::ImageBase => &fields.image_base,
            Field::SectionAlignment => &fields.section_alignment,
            Field::FileAlignment => &fields.file_alignment,
            Field::MajorOperatingSystemVersion => &fields.major_operating_system_version,
            Field::MinorOperatingSystemVersion => &fields.minor_operating_system_version,
            Field::MajorImageVersion => &fields.major_image_version,
            Field::MinorImageVersion => &fields.minor_image_version,
            Field::MajorSubsystemVersion => &fields.major_subsystem_version,
            Field::MinorSubsystemVersion => &fields.minor_subsystem_version,
            Field::Win32VersionValue => &fields.win32_version_value,
            Field::SizeOfImage => &fields.size_of_image,
            Field::SizeOfHeaders => &fields.size_of_headers,
            Field::CheckSum => &fields.check_sum,
            Field::Subsystem => &fields.subsystem,
            Field::DllCharacteristics => &fields.dll_characteristics,
            Field::SizeOfStackReserve => &fields.size_of_stack_reserve,
            Field::SizeOfStackCommit => &fields.size_of_stack_commit,
            Field::SizeOfHeapReserve => &fields.size_of_heap_reserve,
            Field::SizeOfHeapCommit => &fields.size_of_heap_commit,
            Field::LoaderFlags => &fields.loader_flags,
            Field::NumberOfRvaAndSizes => &fields.number_of_rva_and_sizes,
        };

        inspector.record(field.address_space()).field(&field, value);
    }
}

impl Debug for WindowsFieldsCursor<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("WindowsFieldsCursor")
            .field("position", &self.position())
            .field("field", &self.field())
            .finish()
    }
}
