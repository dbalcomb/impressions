use std::fmt::{self, Debug};
use std::ops::Deref;

use bytes::Buf;

use crate::data::parse::Parse;

use super::Error;

/// An image file Section header.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SectionHeader {
    /// The section name.
    ///
    /// This is an 8-byte, null-padded UTF-8 string. There is no terminating
    /// null character if the string is exactly eight characters long.
    name: SectionName,

    /// The total size of the section when loaded into memory, in bytes.
    ///
    /// If this value is greater than the size of raw data then the section is
    /// filled with zeroes.
    virtual_size: u32,

    /// The address of the section when loaded into memory, relative to the
    /// image base.
    virtual_address: u32,

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
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Gets the address of the section.
    pub fn address(&self) -> u32 {
        self.virtual_address
    }
}

impl Parse for SectionHeader {
    type Error = Error;

    fn parse(mut buffer: impl Buf) -> Result<Self, Self::Error> {
        Ok(Self {
            name: SectionName::parse(&mut buffer)?,
            virtual_size: buffer.try_get_u32_le()?,
            virtual_address: buffer.try_get_u32_le()?,
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

/// The section name.
///
/// This is an 8-byte, null-padded UTF-8 encoded string. The documentation
/// states that image files do not support the long name format so this
/// representation is sufficient.
#[derive(Clone, PartialEq, Eq)]
#[repr(transparent)]
struct SectionName([u8; 8]);

impl Parse for SectionName {
    type Error = Error;

    fn parse(mut buffer: impl Buf) -> Result<Self, Self::Error> {
        let bytes = array_init::try_array_init(|_| buffer.try_get_u8())?;

        str::from_utf8(trim_trailing_null(&bytes))?;

        Ok(Self(bytes))
    }
}

impl Deref for SectionName {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        str::from_utf8(trim_trailing_null(&self.0)).expect("validated str")
    }
}

impl Debug for SectionName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", &**self)
    }
}

/// Trims the trailing null bytes.
const fn trim_trailing_null(mut bytes: &[u8]) -> &[u8] {
    while let [rest @ .., 0] = bytes {
        bytes = rest;
    }

    bytes
}
