//! The Optional header of an image file.

pub mod directories;
pub mod standard;
pub mod windows;

use bytes::Buf;
use serde::{Deserialize, Serialize};

use crate::data::parse::Parse;
use crate::image::region::headers::Error;
use crate::memory::address::Address;
use crate::memory::cursor::{AsCursor, SimpleCursor};
use crate::memory::extent::{Extent, Size};
use crate::memory::inspect::{self, Inspect};

use self::directories::DataDirectories;
use self::standard::StandardFields;
use self::windows::WindowsFields;

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
    fn inspect(&self, inspector: &mut inspect::Inspector<'_>) -> Result<(), inspect::Error> {
        writeln!(inspector.identified(), "Optional Header")
    }
}

impl Parse for OptionalHeader {
    type Context<'a> = ();
    type Error = Error;

    fn parse_with(mut buffer: impl Buf, _: Self::Context<'_>) -> Result<Self, Self::Error> {
        let standard = StandardFields::parse(&mut buffer)?;
        let windows = WindowsFields::parse(&mut buffer)?;
        let data_directories = match windows.number_of_rva_and_sizes() {
            0 => None,
            count @ 1..=16 => Some(DataDirectories::parse_with(&mut buffer, count)?),
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
    type Cursor<'a> = SimpleCursor<'a, Self>;

    fn cursor(&self) -> Self::Cursor<'_> {
        SimpleCursor::new(self)
    }
}
