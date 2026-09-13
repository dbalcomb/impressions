use std::fmt::{self, Debug, Display};

use crate::memory::cursor::{Cursor, Error, Position, StructCursor};
use crate::memory::inspect::{self, Inspect};

use super::{DosHeader, Field};

/// A cursor over a DOS header.
#[derive(Clone)]
pub struct DosHeaderCursor<'a>(StructCursor<'a, DosHeader, Field>);

impl<'a> DosHeaderCursor<'a> {
    /// Constructs a new DOS header cursor.
    pub(super) fn new(header: &'a DosHeader) -> Self {
        Self(StructCursor::new(header))
    }
}

impl<'a> DosHeaderCursor<'a> {
    /// Gets the header that this cursor is over.
    pub const fn header(&self) -> &'a DosHeader {
        self.0.structure()
    }

    /// Gets the field that this cursor is at.
    pub fn field(&self) -> Option<Field> {
        self.0.field()
    }
}

impl Cursor for DosHeaderCursor<'_> {
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

impl Inspect for DosHeaderCursor<'_> {
    fn inspect(&self, inspector: &mut inspect::Inspector<'_>) -> Result<(), inspect::Error> {
        let output = &mut inspector.identified();
        let header = self.header();

        let Some(field) = self.field() else {
            return Ok(());
        };

        match field {
            Field::Magic => writeln!(output, "{field}: {}", header.e_magic),
            Field::BytesInLastPage => writeln!(output, "{field}: {}", header.e_cblp),
            Field::Pages => writeln!(output, "{field}: {}", header.e_cp),
            Field::Relocations => writeln!(output, "{field}: {}", header.e_crlc),
            Field::HeaderParagraphs => writeln!(output, "{field}: {}", header.e_cparhdr),
            Field::MinimumAllocation => writeln!(output, "{field}: {}", header.e_minalloc),
            Field::MaximumAllocation => writeln!(output, "{field}: {}", header.e_maxalloc),
            Field::StackSegment => writeln!(output, "{field}: {}", header.e_ss),
            Field::StackPointer => writeln!(output, "{field}: {}", header.e_sp),
            Field::Checksum => writeln!(output, "{field}: {}", header.e_csum),
            Field::InstructionPointer => writeln!(output, "{field}: {}", header.e_ip),
            Field::CodeSegment => writeln!(output, "{field}: {}", header.e_cs),
            Field::RelocationTableOffset => writeln!(output, "{field}: {}", header.e_lfarlc),
            Field::OverlayNumber => writeln!(output, "{field}: {}", header.e_ovno),
            Field::Reserved => writeln!(output, "{field}: {}", ByteArray(&header.e_res)),
            Field::OemId => writeln!(output, "{field}: {}", header.e_oemid),
            Field::OemInfo => writeln!(output, "{field}: {}", header.e_oeminfo),
            Field::Reserved2 => writeln!(output, "{field}: {}", ByteArray(&header.e_res2)),
            Field::PeHeadersOffset => writeln!(output, "{field}: {}", header.e_lfanew),
        }
    }
}

impl Debug for DosHeaderCursor<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DosHeaderCursor")
            .field("position", &self.position())
            .field("field", &self.field())
            .finish()
    }
}

struct ByteArray<'a>(&'a [u16]);

impl Display for ByteArray<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (index, byte) in self
            .0
            .iter()
            .flat_map(|value| value.to_le_bytes())
            .enumerate()
        {
            if index > 0 {
                write!(f, " ")?;
            }

            write!(f, "{byte:02x}")?;
        }

        Ok(())
    }
}
