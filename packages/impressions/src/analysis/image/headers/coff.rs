use bytes::Buf;

use super::Error;

/// The signature of an x86 CPU.
const COFF_MACHINE_X86: u16 = 0x14c;

/// The image file COFF header.
#[derive(Clone, Debug, PartialEq, Eq)]
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
    /// Parses the COFF header from the given buffer.
    pub fn parse(mut buffer: impl Buf) -> Result<Self, Error> {
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
