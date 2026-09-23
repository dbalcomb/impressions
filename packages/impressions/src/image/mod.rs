//! The 32-bit Portable Executable (PE) image file.

mod cursor;
mod error;
mod padding;

pub mod region;

use std::fmt::{self, Debug};

use bytes::Bytes;
use serde::{Deserialize, Serialize};

use crate::analysis::Completion;
use crate::memory::address::{Address, AddressSpace};
use crate::memory::cursor::AsCursor;
use crate::memory::extent::{Extent, Size};
use crate::memory::region::ops::decode::{Decode, Decoder};
use crate::memory::region::ops::encode::encoder::TruncatingEncoder;
use crate::memory::region::ops::encode::{self, Encode};
use crate::memory::region::ops::insert::{Error as InsertError, Insert};
use crate::memory::region::types::segmented::{Segmented, Segments};
use crate::memory::region::types::sparse::{Segment, Sparse};

pub use self::cursor::ImageCursor;
pub use self::error::Error;
pub use self::padding::Padding;

use self::region::Region;
use self::region::headers::Headers;
use self::region::section::Section;
use self::region::section::block::Block;

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

    /// Gets the virtual address of the image.
    pub fn address(&self) -> Address {
        self.headers().optional().image_address()
    }
}

impl Insert<Block> for Image {
    type Error = Error;

    fn insert(&mut self, address: Address, region: Block) -> Result<(), Self::Error> {
        let Some(address) = address.checked_sub(self.address().value()) else {
            return Err(Error::Insert(InsertError::OutOfBounds(
                address.to_space(region.size())?,
                self.address_space(),
            )));
        };

        let Some((offset, segment)) = self.regions.get_mut(address) else {
            return Err(Error::Insert(InsertError::Unsupported(address)));
        };

        let Segment::Occupied(Region::Section(section)) = segment else {
            return Err(Error::Insert(InsertError::Unsupported(address)));
        };

        section.insert(Address::new(offset), region)?;

        Ok(())
    }
}

impl Extent for Image {
    fn size(&self) -> Size {
        self.regions.size()
    }

    fn address_space(&self) -> AddressSpace {
        self.address()
            .to_space(self.size())
            .expect("image address space validated on decode")
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

impl Encode for Image {
    fn encode(&self, encoder: &mut dyn encode::Encoder) -> Result<(), encode::Error> {
        let headers = self.headers();

        if headers.sections().count() != self.sections().count() {
            return Err(encode::Error::InvalidLayout);
        }

        headers.encode(encoder)?;

        let mut file_position = headers.size().get() as usize;
        let mut sections = self.sections();

        for section_header in headers.sections() {
            let padding_size = section_header
                .file_offset()
                .checked_sub(file_position)
                .ok_or(encode::Error::InvalidLayout)?;

            if padding_size > 0 {
                let padding = Padding::new(
                    Size::new(padding_size as u64).expect("valid padding size"),
                    0,
                );

                padding.encode(encoder)?;
            }

            let section = sections.next().ok_or(encode::Error::InvalidLayout)?;
            let mut section_encoder = TruncatingEncoder::new(encoder, section_header.file_size());

            section.encode(&mut section_encoder)?;

            let padding_size = section_header
                .file_size()
                .saturating_sub(section.size().get() as usize);

            if padding_size > 0 {
                let padding = Padding::new(
                    Size::new(padding_size as u64).expect("valid padding size"),
                    0,
                );

                padding.encode(&mut section_encoder)?;
            }

            file_position = section_header.file_offset() + section_header.file_size();
        }

        if sections.next().is_some() {
            return Err(encode::Error::InvalidLayout);
        }

        Ok(())
    }
}

impl Decode for Image {
    type Context<'a> = ();
    type Error = Error;

    fn decode_with(decoder: &mut dyn Decoder, _: Self::Context<'_>) -> Result<Self, Self::Error> {
        let headers = Headers::decode(decoder)?;
        let optional = headers.optional();
        let image_size = Size::new(optional.image_size() as u64)?;

        optional.image_address().to_space(image_size)?;

        let mut regions = Sparse::new(image_size);
        let mut position = headers.size().get() as usize;

        for section_header in headers.sections() {
            decoder.skip(section_header.file_offset() - position)?;

            regions.insert(
                section_header.section_address(),
                Region::section(Section::decode_with(decoder, section_header)?),
            )?;

            position = section_header.file_offset() + section_header.file_size();
        }

        regions.insert(Address::MIN, Region::headers(headers))?;

        Ok(Self { regions })
    }
}

impl AsCursor for Image {
    type Cursor<'a> = ImageCursor<'a>;

    fn cursor(&self) -> Self::Cursor<'_> {
        ImageCursor::new(self)
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

    fn try_from(mut bytes: Bytes) -> Result<Self, Self::Error> {
        Self::decode(&mut bytes)
    }
}
