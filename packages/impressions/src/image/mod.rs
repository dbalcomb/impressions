//! The 32-bit Portable Executable (PE) image file.

mod error;
mod padding;
mod region;

pub mod headers;
pub mod section;

use bytes::{Buf, Bytes};
use serde::{Deserialize, Serialize};

use crate::analysis::Completion;
use crate::data::parse::Parse;
use crate::memory::Extent;
use crate::memory::regions::sparse::Sparse;

pub use self::error::Error;
pub use self::padding::Padding;
pub use self::region::Region;

use self::headers::Headers;
use self::section::Section;

/// A 32-bit Portable Executable (PE) image file analysis.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Image {
    regions: Sparse<Region>,
}

impl Image {
    /// Gets the mapped image headers.
    pub fn headers(&self) -> &Headers {
        self.regions
            .get(0)
            .and_then(|entry| entry.segment().as_occupied())
            .and_then(Region::as_headers)
            .expect("an image always has headers at RVA 0")
    }

    /// Gets an iterator over the sections.
    pub fn sections(&self) -> impl Iterator<Item = &Section> {
        self.regions
            .segments()
            .filter_map(|entry| entry.segment().as_occupied())
            .filter_map(Region::as_section)
    }
}

impl Extent for Image {
    fn size(&self) -> u64 {
        self.regions.size()
    }
}

impl Completion for Image {
    fn identified(&self) -> u64 {
        self.regions.identified()
    }
}

impl Parse for Image {
    type Context<'a> = ();
    type Error = Error;

    fn parse_with(mut buffer: impl Buf, _: Self::Context<'_>) -> Result<Self, Self::Error> {
        let headers = Headers::parse(&mut buffer)?;

        let mut regions = Sparse::new(headers.optional().image_size())?;
        let mut position = headers.size() as usize;

        for section_header in headers.sections() {
            buffer.advance(section_header.file_offset() - position);

            regions.insert(
                section_header.section_address().value(),
                Region::section(Section::parse_with(&mut buffer, section_header)?),
            )?;

            position = section_header.file_offset() + section_header.file_size();
        }

        regions.insert(0, Region::headers(headers))?;

        Ok(Self { regions })
    }
}

impl TryFrom<Bytes> for Image {
    type Error = Error;

    fn try_from(bytes: Bytes) -> Result<Self, Self::Error> {
        Self::parse(bytes)
    }
}
