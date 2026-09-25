//! The Optional header of an image file.

pub mod directories;
pub mod standard;
pub mod windows;

mod cursor;

use serde::{Deserialize, Serialize};

use crate::image::region::headers::Error;
use crate::memory::address::Address;
use crate::memory::cursor::AsCursor;
use crate::memory::extent::{Extent, Size};
use crate::memory::inspect::{Inspect, Inspector};
use crate::memory::region::ops::decode::{Decode, Decoder};
use crate::memory::region::ops::encode::{self, Encode};

use self::directories::DataDirectories;
use self::standard::StandardFields;
use self::windows::WindowsFields;

pub use self::cursor::OptionalHeaderCursor;

/// The image file Optional header.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct OptionalHeader {
    /// The standard fields.
    standard: StandardFields,

    /// The Windows-specific fields.
    windows: WindowsFields,

    /// The data directories.
    data_directories: Option<DataDirectories>,
}

impl OptionalHeader {
    /// Gets the base address of the image.
    pub const fn image_address(&self) -> Address {
        self.windows.image_base()
    }

    /// Gets the total size of the image in memory.
    pub const fn image_size(&self) -> u32 {
        self.windows.size_of_image()
    }

    /// Gets the total size of the headers in memory.
    pub const fn headers_size(&self) -> u32 {
        self.windows.size_of_headers()
    }

    /// Gets the section alignment.
    pub const fn section_alignment(&self) -> u32 {
        self.windows.section_alignment()
    }

    /// Gets the data directories.
    pub const fn data_directories(&self) -> Option<&DataDirectories> {
        self.data_directories.as_ref()
    }

    /// Gets the address of the entry point.
    pub const fn entry_point(&self) -> Address {
        match self
            .image_address()
            .checked_add(self.standard.address_of_entry_point().value())
        {
            Some(address) => address,
            None => panic!("entry point address should fit in address space"),
        }
    }
}

impl Extent for OptionalHeader {
    fn size(&self) -> Size {
        let size = self
            .standard
            .size()
            .checked_add(self.windows.size())
            .expect("sum of sizes does not exceed maximum size");

        match &self.data_directories {
            Some(data_directories) => size
                .checked_add(data_directories.size())
                .expect("sum of sizes does not exceed maximum size"),
            None => size,
        }
    }
}

impl Inspect for OptionalHeader {
    fn inspect(&self, inspector: &mut dyn Inspector) {
        inspector
            .record(self.address_space())
            .label(&"Optional Header")
            .finish()
    }
}

impl Encode for OptionalHeader {
    fn encode(&self, encoder: &mut dyn encode::Encoder) -> Result<(), encode::Error> {
        self.standard.encode(encoder)?;
        self.windows.encode(encoder)?;

        if let Some(data_directories) = &self.data_directories {
            data_directories.encode(encoder)?;
        }

        Ok(())
    }
}

impl Decode for OptionalHeader {
    type Context<'a> = ();
    type Error = Error;

    fn decode_with(decoder: &mut dyn Decoder, _: Self::Context<'_>) -> Result<Self, Self::Error> {
        let standard = StandardFields::decode(decoder)?;
        let windows = WindowsFields::decode(decoder)?;
        let data_directories = match windows.number_of_rva_and_sizes() {
            0 => None,
            count @ 1..=16 => Some(DataDirectories::decode_with(decoder, count)?),
            count => return Err(Error::UnsupportedDataDirectoryCount(count)),
        };

        Ok(Self {
            standard,
            windows,
            data_directories,
        })
    }
}

impl AsCursor for OptionalHeader {
    type Cursor<'a> = OptionalHeaderCursor<'a>;

    fn cursor(&self) -> Self::Cursor<'_> {
        OptionalHeaderCursor::new(self)
    }
}
