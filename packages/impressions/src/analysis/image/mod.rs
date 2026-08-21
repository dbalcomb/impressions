//! The image file analysis.

mod error;
pub mod headers;
pub mod section;

use bytes::{Buf, Bytes};
use serde::{Deserialize, Serialize};

use crate::data::parse::Parse;
use crate::memory::address::AddressSpace;
use crate::memory::map::{Iter, Map};
use crate::memory::region::Region;

pub use self::error::Error;
use self::headers::Headers;
use self::section::Section;

use super::Completion;

/// A 32-bit Portable Executable (PE) image file analysis.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Image {
    headers: Headers,
    sections: Map<Section>,
}

impl Image {
    /// Gets an iterator over the sections.
    pub fn sections(&self) -> Iter<'_, Section> {
        self.sections.iter()
    }
}

impl Region for Image {
    fn address_space(&self) -> AddressSpace {
        self.headers.image_address_space()
    }
}

impl Completion for Image {
    fn identified(&self) -> u64 {
        self.headers.identified() + self.sections().map(Section::identified).sum::<u64>()
    }
}

impl Parse for Image {
    type Error = Error;

    fn parse(mut buffer: impl Buf) -> Result<Self, Self::Error> {
        let headers = Headers::parse(&mut buffer)?;
        let mut position = headers.size() as usize;
        let mut sections = Map::new(headers.sections_address_space());

        for section_header in headers.sections() {
            buffer.advance(section_header.file_offset() - position);
            sections.insert(Section::parse_with(
                &mut buffer,
                headers.optional(),
                section_header,
            )?)?;

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
