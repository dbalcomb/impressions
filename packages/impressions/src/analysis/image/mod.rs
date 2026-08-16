//! The image file analysis.

mod error;
pub mod headers;
pub mod section;

use std::collections::BTreeMap;
use std::path::Path;

use bytes::{Buf, Bytes};

use crate::data::parse::Parse;

pub use self::error::Error;
use self::headers::Headers;
use self::section::Section;

/// A 32-bit Portable Executable (PE) image file analysis.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Image {
    headers: Headers,
    sections: BTreeMap<u32, Section>,
}

impl Image {
    /// Constructs a new image file analysis from the given image file path.
    pub fn open(path: impl AsRef<Path>) -> Result<Self, Error> {
        Self::parse(Bytes::from(std::fs::read(path)?))
    }
}

impl Image {
    /// Gets the image address.
    pub fn address(&self) -> u32 {
        self.headers.address()
    }

    /// Gets the image size.
    pub fn size(&self) -> u64 {
        self.headers.optional().image_size()
    }

    /// Gets an iterator over the sections.
    pub fn sections(&self) -> impl Iterator<Item = &Section> {
        self.sections.values()
    }
}

impl Parse for Image {
    type Error = Error;

    fn parse(mut buffer: impl Buf) -> Result<Self, Self::Error> {
        let headers = Headers::parse(&mut buffer)?;
        let mut position = headers.size() as usize;
        let mut sections = BTreeMap::new();

        for section_header in headers.sections() {
            buffer.advance(section_header.file_offset() - position);
            sections.insert(
                headers.address() + section_header.address(),
                Section::parse_with(&mut buffer, headers.optional(), section_header)?,
            );

            position = section_header.file_offset() + section_header.file_size();
        }

        Ok(Self { headers, sections })
    }
}

impl TryFrom<Bytes> for Image {
    type Error = Error;

    fn try_from(bytes: Bytes) -> Result<Self, Self::Error> {
        Self::parse(bytes)
    }
}
