//! The standard fields of the Optional header.

mod cursor;
mod field;

use serde::{Deserialize, Serialize};

use crate::image::region::headers::Error;
use crate::memory::address::Address;
use crate::memory::cursor::AsCursor;
use crate::memory::extent::{Extent, FixedExtent, Size};
use crate::memory::inspect::{Inspect, Inspector};
use crate::memory::region::ops::decode::{Decode, Decoder};
use crate::memory::region::ops::encode::{self, Encode};

pub use self::cursor::StandardFieldsCursor;
pub use self::field::Field;

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

impl FixedExtent for StandardFields {
    const SIZE: Size = Size::new_valid(28);
}

impl Inspect for StandardFields {
    fn inspect(&self, inspector: &mut dyn Inspector) {
        inspector
            .record(self.address_space())
            .label(&"Standard Fields")
            .finish()
    }
}

impl Encode for StandardFields {
    fn encode(&self, encoder: &mut dyn encode::Encoder) -> Result<(), encode::Error> {
        encoder.write_u16_le(self.magic)?;
        encoder.write_u8(self.major_linker_version)?;
        encoder.write_u8(self.minor_linker_version)?;
        encoder.write_u32_le(self.size_of_code)?;
        encoder.write_u32_le(self.size_of_initialized_data)?;
        encoder.write_u32_le(self.size_of_uninitialized_data)?;
        self.address_of_entry_point.encode(encoder)?;
        encoder.write_u32_le(self.base_of_code)?;
        encoder.write_u32_le(self.base_of_data)?;

        Ok(())
    }
}

impl Decode for StandardFields {
    type Context<'a> = ();
    type Error = Error;

    fn decode_with(decoder: &mut dyn Decoder, _: Self::Context<'_>) -> Result<Self, Self::Error> {
        let magic = decoder.read_u16_le()?;

        if magic != Self::SIGNATURE {
            return Err(Error::UnsupportedArchitecture);
        }

        Ok(Self {
            magic,
            major_linker_version: decoder.read_u8()?,
            minor_linker_version: decoder.read_u8()?,
            size_of_code: decoder.read_u32_le()?,
            size_of_initialized_data: decoder.read_u32_le()?,
            size_of_uninitialized_data: decoder.read_u32_le()?,
            address_of_entry_point: Address::decode(decoder)?,
            base_of_code: decoder.read_u32_le()?,
            base_of_data: decoder.read_u32_le()?,
        })
    }
}

impl AsCursor for StandardFields {
    type Cursor<'a> = StandardFieldsCursor<'a>;

    fn cursor(&self) -> Self::Cursor<'_> {
        StandardFieldsCursor::new(self)
    }
}
