//! The Section header of an image file.

mod characteristics;
mod cursor;
mod field;

use bytes::Buf;
use serde::{Deserialize, Serialize};

use crate::data::parse::Parse;
use crate::data::types::array_string::ArrayString;
use crate::image::region::headers::Error;
use crate::memory::address::Address;
use crate::memory::cursor::AsCursor;
use crate::memory::extent::{Extent, FixedExtent, Size};
use crate::memory::inspect::{Inspect, Inspector};
use crate::memory::region::ops::encode::{self, Encode};

pub use self::characteristics::SectionCharacteristics;
pub use self::cursor::SectionHeaderCursor;
pub use self::field::Field;

/// An image file Section header.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SectionHeader {
    /// The section name.
    ///
    /// This is an 8-byte, null-padded UTF-8 string. There is no terminating
    /// null character if the string is exactly eight characters long.
    name: ArrayString<8>,

    /// The total size of the section when loaded into memory, in bytes.
    ///
    /// If this value is greater than the size of raw data then the section is
    /// filled with zeroes.
    virtual_size: u32,

    /// The address of the section when loaded into memory, relative to the
    /// image base.
    virtual_address: Address,

    /// The size of the initialized data on disk, in bytes.
    ///
    /// This value must be a multiple of the file alignment. If this value is
    /// less than the virtual size then the remainder of the section is filled
    /// with zeroes. If the section contains only uninitialized data then this
    /// value is zero.
    size_of_raw_data: u32,

    /// A file pointer to the first page within the file.
    ///
    /// This value must be a multiple of the file alignment. If a section
    /// contains only uninitialized data then this value is zero.
    pointer_to_raw_data: u32,

    /// A file pointer to the beginning of the relocation entries for the
    /// section.
    ///
    /// If there are no relocations then this value is zero.
    pointer_to_relocations: u32,

    /// A file pointer to the beginning of the line-number entries for the
    /// section.
    ///
    /// If there are no COFF line numbers then this value is zero.
    pointer_to_linenumbers: u32,

    /// The number of relocation entries for the section.
    ///
    /// This value is zero for executable images.
    number_of_relocations: u16,

    /// The number of line-number entries for the section.
    number_of_linenumbers: u16,

    /// The characteristics of the section.
    characteristics: SectionCharacteristics,
}

impl SectionHeader {
    /// Gets the section name.
    pub fn name(&self) -> &ArrayString<8> {
        &self.name
    }

    /// Gets the address of the section.
    pub fn section_address(&self) -> Address {
        self.virtual_address
    }

    /// Gets the size of the section.
    pub fn section_size(&self) -> u64 {
        self.virtual_size as u64
    }

    /// Gets the file offset of the data.
    pub fn file_offset(&self) -> usize {
        self.pointer_to_raw_data as usize
    }

    /// Gets the size of the data.
    pub fn file_size(&self) -> usize {
        self.size_of_raw_data as usize
    }

    /// Gets the section characteristics.
    pub fn characteristics(&self) -> SectionCharacteristics {
        self.characteristics
    }
}

impl FixedExtent for SectionHeader {
    const SIZE: Size = Size::new_valid(40);
}

impl Inspect for SectionHeader {
    fn inspect(&self, inspector: &mut dyn Inspector) {
        inspector
            .record(self.address_space())
            .label(&"Section Header")
            .finish()
    }
}

impl Encode for SectionHeader {
    fn encode(&self, encoder: &mut dyn encode::Encoder) -> Result<(), encode::Error> {
        self.name.encode(encoder)?;
        encoder.write_u32_le(self.virtual_size)?;
        self.virtual_address.encode(encoder)?;
        encoder.write_u32_le(self.size_of_raw_data)?;
        encoder.write_u32_le(self.pointer_to_raw_data)?;
        encoder.write_u32_le(self.pointer_to_relocations)?;
        encoder.write_u32_le(self.pointer_to_linenumbers)?;
        encoder.write_u16_le(self.number_of_relocations)?;
        encoder.write_u16_le(self.number_of_linenumbers)?;
        self.characteristics.encode(encoder)?;

        Ok(())
    }
}

impl Parse for SectionHeader {
    type Context<'a> = ();
    type Error = Error;

    fn parse_with(mut buffer: impl Buf, _: Self::Context<'_>) -> Result<Self, Self::Error> {
        Ok(Self {
            name: ArrayString::parse(&mut buffer).map_err(Error::InvalidSectionName)?,
            virtual_size: buffer.try_get_u32_le()?,
            virtual_address: Address::parse(&mut buffer)?,
            size_of_raw_data: buffer.try_get_u32_le()?,
            pointer_to_raw_data: buffer.try_get_u32_le()?,
            pointer_to_relocations: buffer.try_get_u32_le()?,
            pointer_to_linenumbers: buffer.try_get_u32_le()?,
            number_of_relocations: buffer.try_get_u16_le()?,
            number_of_linenumbers: buffer.try_get_u16_le()?,
            characteristics: SectionCharacteristics::parse(&mut buffer)?,
        })
    }
}

impl AsCursor for SectionHeader {
    type Cursor<'a> = SectionHeaderCursor<'a>;

    fn cursor(&self) -> Self::Cursor<'_> {
        SectionHeaderCursor::new(self)
    }
}
