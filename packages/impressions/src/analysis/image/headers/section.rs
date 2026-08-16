use bytes::Buf;

use crate::data::parse::Parse;
use crate::data::types::array_string::ArrayString;
use crate::memory::address::Address;

use super::Error;

/// An image file Section header.
#[derive(Clone, Debug, PartialEq, Eq)]
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

    /// The characteristics of the image.
    characteristics: u32,
}

impl SectionHeader {
    /// The size of the section header.
    pub const SIZE: usize = 40;
}

impl SectionHeader {
    /// Gets the section name.
    pub fn name(&self) -> &ArrayString<8> {
        &self.name
    }

    /// Gets the address of the section.
    pub fn address(&self) -> Address {
        self.virtual_address
    }

    /// Gets the size of the section.
    pub fn size(&self) -> u64 {
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
}

impl Parse for SectionHeader {
    type Error = Error;

    fn parse(mut buffer: impl Buf) -> Result<Self, Self::Error> {
        Ok(Self {
            name: ArrayString::parse(&mut buffer)?,
            virtual_size: buffer.try_get_u32_le()?,
            virtual_address: Address::parse(&mut buffer)?,
            size_of_raw_data: buffer.try_get_u32_le()?,
            pointer_to_raw_data: buffer.try_get_u32_le()?,
            pointer_to_relocations: buffer.try_get_u32_le()?,
            pointer_to_linenumbers: buffer.try_get_u32_le()?,
            number_of_relocations: buffer.try_get_u16_le()?,
            number_of_linenumbers: buffer.try_get_u16_le()?,
            characteristics: buffer.try_get_u32_le()?,
        })
    }
}
