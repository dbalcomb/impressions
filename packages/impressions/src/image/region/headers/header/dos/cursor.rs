use std::fmt::{self, Debug, Display};

use crate::memory::cursor::{Cursor, Error, Position, StructCursor};
use crate::memory::extent::Extent;
use crate::memory::inspect::{Inspect, InspectionValue, Inspector};
use crate::memory::region::ops::encode::{self, Encode};

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
    fn inspect(&self, inspector: &mut dyn Inspector) {
        let header = self.header();

        let Some(field) = self.field() else {
            return;
        };

        let reserved = ByteArray(&header.e_res);
        let reserved2 = ByteArray(&header.e_res2);
        let value: &dyn InspectionValue = match field {
            Field::Magic => &header.e_magic,
            Field::BytesInLastPage => &header.e_cblp,
            Field::Pages => &header.e_cp,
            Field::Relocations => &header.e_crlc,
            Field::HeaderParagraphs => &header.e_cparhdr,
            Field::MinimumAllocation => &header.e_minalloc,
            Field::MaximumAllocation => &header.e_maxalloc,
            Field::StackSegment => &header.e_ss,
            Field::StackPointer => &header.e_sp,
            Field::Checksum => &header.e_csum,
            Field::InstructionPointer => &header.e_ip,
            Field::CodeSegment => &header.e_cs,
            Field::RelocationTableOffset => &header.e_lfarlc,
            Field::OverlayNumber => &header.e_ovno,
            Field::Reserved => &reserved,
            Field::OemId => &header.e_oemid,
            Field::OemInfo => &header.e_oeminfo,
            Field::Reserved2 => &reserved2,
            Field::PeHeadersOffset => &header.e_lfanew,
        };

        inspector.record(field.address_space()).field(&field, value);
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

impl Encode for ByteArray<'_> {
    fn encode(&self, encoder: &mut dyn encode::Encoder) -> Result<(), encode::Error> {
        for value in self.0 {
            encoder.write_u16_le(*value)?;
        }

        Ok(())
    }
}

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

impl InspectionValue for ByteArray<'_> {
    fn data_type(&self) -> &dyn Display {
        &"bytes"
    }
}
