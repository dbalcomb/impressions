//! The standard fields of the Optional header.

use bytes::Buf;
use serde::{Deserialize, Serialize};

use crate::data::parse::Parse;
use crate::image::region::headers::Error;
use crate::memory::address::Address;
use crate::memory::extent::{Extent, Size};

/// The standard fields of the Optional header.
///
/// The first eight fields of the optional header are standard fields that are
/// defined for every implementation of COFF. These fields contain general
/// information that is useful for loading and running an executable file.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct StandardFields {
    /// The image file type.
    ///
    /// This indicates whether the image is 32-bit (PE32) or 64-bit (PE32+).
    magic: u16,

    /// The major version number of the linker.
    major_linker_version: u8,

    /// The minor version number of the linker.
    minor_linker_version: u8,

    /// The sum of the size of the code sections.
    size_of_code: u32,

    /// The sum of the size of the initialized data sections.
    size_of_initialized_data: u32,

    /// The sum of the size of the uninitialized data sections.
    size_of_uninitialized_data: u32,

    /// The address of the entry function, relative to the base address.
    address_of_entry_point: Address,

    /// The address of the code section, relative to the image base.
    base_of_code: u32,

    /// The address of the data section, relative to the image base.
    base_of_data: u32,
}

impl StandardFields {
    /// The signature of a 32-bit PE image file.
    const SIGNATURE: u16 = 0x10b;
}

impl Extent for StandardFields {
    fn size(&self) -> Size {
        Size::new(28).expect("valid size")
    }
}

impl Parse for StandardFields {
    type Context<'a> = ();
    type Error = Error;

    fn parse_with(mut buffer: impl Buf, _: Self::Context<'_>) -> Result<Self, Self::Error> {
        let magic = buffer.try_get_u16_le()?;

        if magic != Self::SIGNATURE {
            return Err(Error::UnsupportedArchitecture);
        }

        Ok(Self {
            magic,
            major_linker_version: buffer.try_get_u8()?,
            minor_linker_version: buffer.try_get_u8()?,
            size_of_code: buffer.try_get_u32_le()?,
            size_of_initialized_data: buffer.try_get_u32_le()?,
            size_of_uninitialized_data: buffer.try_get_u32_le()?,
            address_of_entry_point: Address::parse(&mut buffer)?,
            base_of_code: buffer.try_get_u32_le()?,
            base_of_data: buffer.try_get_u32_le()?,
        })
    }
}
