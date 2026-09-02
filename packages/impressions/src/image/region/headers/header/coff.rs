use bytes::Buf;
use serde::{Deserialize, Serialize};

use crate::data::parse::Parse;
use crate::image::region::headers::Error;
use crate::memory::Extent;

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

impl Extent for CoffHeader {
    fn size(&self) -> u64 {
        20
    }
}

impl Parse for CoffHeader {
    type Context<'a> = ();
    type Error = Error;

    fn parse_with(mut buffer: impl Buf, _: Self::Context<'_>) -> Result<Self, Self::Error> {
        let machine = buffer.try_get_u16_le()?;

        if machine != COFF_MACHINE_X86 {
            return Err(Error::UnsupportedArchitecture);
        }

        Ok(Self {
            machine,
            number_of_sections: buffer.try_get_u16_le()?,
            time_date_stamp: buffer.try_get_u32_le()?,
            pointer_to_symbol_table: buffer.try_get_u32_le()?,
            number_of_symbols: buffer.try_get_u32_le()?,
            size_of_optional_header: buffer.try_get_u16_le()?,
            characteristics: buffer.try_get_u16_le()?,
        })
    }
}
