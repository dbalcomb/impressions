use std::fmt::{self, Debug};

use crate::memory::cursor::{Cursor, Error, Position, StructCursor};
use crate::memory::inspect::{self, Inspect};

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
    fn inspect(&self, inspector: &mut inspect::Inspector<'_>) -> Result<(), inspect::Error> {
        let output = &mut inspector.identified();
        let fields = self.fields();

        let Some(field) = self.field() else {
            return Ok(());
        };

        match field {
            Field::ImageBase => writeln!(output, "{field}: {}", fields.image_base),
            Field::SectionAlignment => writeln!(output, "{field}: {}", fields.section_alignment),
            Field::FileAlignment => writeln!(output, "{field}: {}", fields.file_alignment),
            Field::MajorOperatingSystemVersion => {
                writeln!(output, "{field}: {}", fields.major_operating_system_version)
            }
            Field::MinorOperatingSystemVersion => {
                writeln!(output, "{field}: {}", fields.minor_operating_system_version)
            }
            Field::MajorImageVersion => writeln!(output, "{field}: {}", fields.major_image_version),
            Field::MinorImageVersion => writeln!(output, "{field}: {}", fields.minor_image_version),
            Field::MajorSubsystemVersion => {
                writeln!(output, "{field}: {}", fields.major_subsystem_version)
            }
            Field::MinorSubsystemVersion => {
                writeln!(output, "{field}: {}", fields.minor_subsystem_version)
            }
            Field::Win32VersionValue => writeln!(output, "{field}: {}", fields.win32_version_value),
            Field::SizeOfImage => writeln!(output, "{field}: {}", fields.size_of_image),
            Field::SizeOfHeaders => writeln!(output, "{field}: {}", fields.size_of_headers),
            Field::CheckSum => writeln!(output, "{field}: {}", fields.check_sum),
            Field::Subsystem => writeln!(output, "{field}: {}", fields.subsystem),
            Field::DllCharacteristics => {
                writeln!(output, "{field}: {}", fields.dll_characteristics)
            }
            Field::SizeOfStackReserve => {
                writeln!(output, "{field}: {}", fields.size_of_stack_reserve)
            }
            Field::SizeOfStackCommit => {
                writeln!(output, "{field}: {}", fields.size_of_stack_commit)
            }
            Field::SizeOfHeapReserve => {
                writeln!(output, "{field}: {}", fields.size_of_heap_reserve)
            }
            Field::SizeOfHeapCommit => writeln!(output, "{field}: {}", fields.size_of_heap_commit),
            Field::LoaderFlags => writeln!(output, "{field}: {}", fields.loader_flags),
            Field::NumberOfRvaAndSizes => {
                writeln!(output, "{field}: {}", fields.number_of_rva_and_sizes)
            }
        }
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
