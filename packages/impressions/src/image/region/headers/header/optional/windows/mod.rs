//! The Windows-specific fields of the Optional header.

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

pub use self::cursor::WindowsFieldsCursor;
pub use self::field::Field;

/// The Windows-specific fields of the Optional header.
///
/// The next 21 fields are an extension to the COFF optional header format. They
/// contain additional information that is required by the linker and loader in
/// Windows.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WindowsFields {
    /// The preferred address of the image in memory.
    image_base: Address,

    /// The alignment of sections in memory.
    section_alignment: u32,

    /// The alignment of sections in the image file.
    file_alignment: u32,

    /// The major version number of the required operating system.
    major_operating_system_version: u16,

    /// The minor version number of the required operating system.
    minor_operating_system_version: u16,

    /// The major version number of the image.
    major_image_version: u16,

    /// The minor version number of the image.
    minor_image_version: u16,

    /// The major version number of the subsystem.
    major_subsystem_version: u16,

    /// The minor version number of the subsystem.
    minor_subsystem_version: u16,

    /// This member is reserved and must be 0.
    win32_version_value: u32,

    /// The size of the image, including all headers.
    size_of_image: u32,

    /// The combined size of the following items, rounded to a multiple of the
    /// file alignment.
    ///
    /// * PE Headers offset
    /// * PE Signature
    /// * Size of COFF header
    /// * Size of Optional header
    /// * Size of all section headers
    size_of_headers: u32,

    /// The image file checksum.
    check_sum: u32,

    /// The subsystem required to run the image.
    subsystem: u16,

    /// The DLL characteristics of the image.
    dll_characteristics: u16,

    /// The number of bytes to reserve for the stack.
    size_of_stack_reserve: u32,

    /// The number of bytes to commit for the stack.
    size_of_stack_commit: u32,

    /// The number of bytes to reserve for the local heap.
    size_of_heap_reserve: u32,

    /// The number of bytes to commit for the local heap.
    size_of_heap_commit: u32,

    /// This member is obsolete.
    loader_flags: u32,

    /// The number of data-directory entries in the remainder of the optional
    /// header.
    number_of_rva_and_sizes: u32,
}

impl WindowsFields {
    /// Gets the preferred address of the image in memory.
    pub const fn image_base(&self) -> Address {
        self.image_base
    }

    /// Gets the size of the image in memory.
    pub const fn size_of_image(&self) -> u32 {
        self.size_of_image
    }

    /// Gets the combined size of the headers in the image file.
    pub const fn size_of_headers(&self) -> u32 {
        self.size_of_headers
    }

    /// Gets the alignment of sections in memory.
    pub const fn section_alignment(&self) -> u32 {
        self.section_alignment
    }

    /// Gets the number of data-directory entries in the remainder of the
    /// optional header.
    pub const fn number_of_rva_and_sizes(&self) -> u32 {
        self.number_of_rva_and_sizes
    }
}

impl FixedExtent for WindowsFields {
    const SIZE: Size = Size::new_valid(68);
}

impl Inspect for WindowsFields {
    fn inspect(&self, inspector: &mut dyn Inspector) {
        inspector
            .record(self.address_space())
            .label(&"Windows Fields")
            .finish()
    }
}

impl Encode for WindowsFields {
    fn encode(&self, encoder: &mut dyn encode::Encoder) -> Result<(), encode::Error> {
        self.image_base.encode(encoder)?;
        encoder.write_u32_le(self.section_alignment)?;
        encoder.write_u32_le(self.file_alignment)?;
        encoder.write_u16_le(self.major_operating_system_version)?;
        encoder.write_u16_le(self.minor_operating_system_version)?;
        encoder.write_u16_le(self.major_image_version)?;
        encoder.write_u16_le(self.minor_image_version)?;
        encoder.write_u16_le(self.major_subsystem_version)?;
        encoder.write_u16_le(self.minor_subsystem_version)?;
        encoder.write_u32_le(self.win32_version_value)?;
        encoder.write_u32_le(self.size_of_image)?;
        encoder.write_u32_le(self.size_of_headers)?;
        encoder.write_u32_le(self.check_sum)?;
        encoder.write_u16_le(self.subsystem)?;
        encoder.write_u16_le(self.dll_characteristics)?;
        encoder.write_u32_le(self.size_of_stack_reserve)?;
        encoder.write_u32_le(self.size_of_stack_commit)?;
        encoder.write_u32_le(self.size_of_heap_reserve)?;
        encoder.write_u32_le(self.size_of_heap_commit)?;
        encoder.write_u32_le(self.loader_flags)?;
        encoder.write_u32_le(self.number_of_rva_and_sizes)?;

        Ok(())
    }
}

impl Decode for WindowsFields {
    type Context<'a> = ();
    type Error = Error;

    fn decode_with(decoder: &mut dyn Decoder, _: Self::Context<'_>) -> Result<Self, Self::Error> {
        Ok(Self {
            image_base: Address::decode(decoder)?,
            section_alignment: decoder.read_u32_le()?,
            file_alignment: decoder.read_u32_le()?,
            major_operating_system_version: decoder.read_u16_le()?,
            minor_operating_system_version: decoder.read_u16_le()?,
            major_image_version: decoder.read_u16_le()?,
            minor_image_version: decoder.read_u16_le()?,
            major_subsystem_version: decoder.read_u16_le()?,
            minor_subsystem_version: decoder.read_u16_le()?,
            win32_version_value: decoder.read_u32_le()?,
            size_of_image: decoder.read_u32_le()?,
            size_of_headers: decoder.read_u32_le()?,
            check_sum: decoder.read_u32_le()?,
            subsystem: decoder.read_u16_le()?,
            dll_characteristics: decoder.read_u16_le()?,
            size_of_stack_reserve: decoder.read_u32_le()?,
            size_of_stack_commit: decoder.read_u32_le()?,
            size_of_heap_reserve: decoder.read_u32_le()?,
            size_of_heap_commit: decoder.read_u32_le()?,
            loader_flags: decoder.read_u32_le()?,
            number_of_rva_and_sizes: decoder.read_u32_le()?,
        })
    }
}

impl AsCursor for WindowsFields {
    type Cursor<'a> = WindowsFieldsCursor<'a>;

    fn cursor(&self) -> Self::Cursor<'_> {
        WindowsFieldsCursor::new(self)
    }
}
