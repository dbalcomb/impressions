//! The COFF header of an image file.

mod cursor;
mod field;

use serde::{Deserialize, Serialize};

use crate::image::region::headers::Error;
use crate::memory::cursor::AsCursor;
use crate::memory::extent::{Extent, FixedExtent, Size};
use crate::memory::inspect::{Inspect, Inspector};
use crate::memory::region::ops::decode::{Decode, Decoder};
use crate::memory::region::ops::encode::{self, Encode};

pub use self::cursor::CoffHeaderCursor;
pub use self::field::Field;

/// The signature of an x86 CPU.
const COFF_MACHINE_X86: u16 = 0x14c;

/// The image file COFF header.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CoffHeader {
    /// The type of target machine.
    machine: u16,

    /// The number of sections.
    ///
    /// This indicates the size of the section table, which immediately follows
    /// the headers.
    number_of_sections: u16,

    /// The timestamp of when the image was created by the linker.
    time_date_stamp: u32,

    /// The file offset of the COFF symbol table.
    ///
    /// This is not relevant to image files.
    pointer_to_symbol_table: u32,

    /// The number of entries in the COFF symbol table.
    ///
    /// This is not relevant to image files.
    number_of_symbols: u32,

    /// The size of the optional header.
    size_of_optional_header: u16,

    /// The flags that indicate the attributes of the file.
    characteristics: u16,
}

impl CoffHeader {
    /// Gets the number of sections.
    pub fn number_of_sections(&self) -> usize {
        self.number_of_sections as usize
    }
}

impl FixedExtent for CoffHeader {
    const SIZE: Size = Size::new_valid(20);
}

impl Inspect for CoffHeader {
    fn inspect(&self, inspector: &mut dyn Inspector) {
        inspector
            .record(self.address_space())
            .label(&"COFF Header")
            .finish()
    }
}

impl AsCursor for CoffHeader {
    type Cursor<'a> = CoffHeaderCursor<'a>;

    fn cursor(&self) -> Self::Cursor<'_> {
        CoffHeaderCursor::new(self)
    }
}

impl Encode for CoffHeader {
    fn encode(&self, encoder: &mut dyn encode::Encoder) -> Result<(), encode::Error> {
        encoder.write_u16_le(self.machine)?;
        encoder.write_u16_le(self.number_of_sections)?;
        encoder.write_u32_le(self.time_date_stamp)?;
        encoder.write_u32_le(self.pointer_to_symbol_table)?;
        encoder.write_u32_le(self.number_of_symbols)?;
        encoder.write_u16_le(self.size_of_optional_header)?;
        encoder.write_u16_le(self.characteristics)?;

        Ok(())
    }
}

impl Decode for CoffHeader {
    type Context<'a> = ();
    type Error = Error;

    fn decode_with(decoder: &mut dyn Decoder, _: Self::Context<'_>) -> Result<Self, Self::Error> {
        let machine = decoder.read_u16_le()?;

        if machine != COFF_MACHINE_X86 {
            return Err(Error::UnsupportedArchitecture);
        }

        Ok(Self {
            machine,
            number_of_sections: decoder.read_u16_le()?,
            time_date_stamp: decoder.read_u32_le()?,
            pointer_to_symbol_table: decoder.read_u32_le()?,
            number_of_symbols: decoder.read_u32_le()?,
            size_of_optional_header: decoder.read_u16_le()?,
            characteristics: decoder.read_u16_le()?,
        })
    }
}
