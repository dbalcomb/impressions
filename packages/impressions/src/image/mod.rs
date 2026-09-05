//! The 32-bit Portable Executable (PE) image file.

mod error;
mod padding;

pub mod region;

use std::fmt::{self, Debug};

use bytes::{Buf, Bytes};
use serde::{Deserialize, Serialize};

use crate::analysis::Completion;
use crate::data::parse::Parse;
use crate::memory::Extent;
use crate::memory::address::Address;
use crate::memory::regions::sparse::{Segment, Sparse};
use crate::memory::segmented::{Segmented, Segments};

pub use self::error::Error;
pub use self::padding::Padding;

use self::region::Region;
use self::region::headers::Headers;
use self::region::section::Section;

/// A 32-bit Portable Executable (PE) image file analysis.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Image {
    regions: Sparse<Region>,
}

impl Image {
    /// Gets the mapped image headers.
    pub fn headers(&self) -> &Headers {
        self.regions
            .get(Address::MIN)
            .and_then(|entry| entry.segment().as_occupied())
            .and_then(Region::as_headers)
            .expect("an image always has headers at RVA 0")
    }

    /// Gets an iterator over the sections.
    pub fn sections(&self) -> impl Iterator<Item = &Section> {
        self.regions
            .segments()
            .into_iter()
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

impl Segmented for Image {
    type Segment = Segment<Region>;

    fn segments(&self) -> Segments<'_, Self::Segment> {
        self.regions.segments()
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
                section_header.section_address(),
                Region::section(Section::parse_with(&mut buffer, section_header)?),
            )?;

            position = section_header.file_offset() + section_header.file_size();
        }

        regions.insert(Address::MIN, Region::headers(headers))?;

        Ok(Self { regions })
    }
}

impl Debug for Image {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let completion = std::fmt::from_fn(|f| write!(f, "{:.2}%", self.completion()));

        f.debug_struct("Image")
            .field("completion", &completion)
            .field("regions", &self.regions)
            .finish()
    }
}

impl TryFrom<Bytes> for Image {
    type Error = Error;

    fn try_from(bytes: Bytes) -> Result<Self, Self::Error> {
        Self::parse(bytes)
    }
}
