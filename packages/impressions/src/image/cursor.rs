use std::fmt::{self, Debug};

use crate::memory::address::Address;
use crate::memory::cursor::{Cursor, Error, Position};
use crate::memory::inspect::{self, Inspect};
use crate::memory::regions::sparse::Segment;
use crate::memory::regions::unidentified::Unidentified;
use crate::memory::segmented::{Segmented, SegmentsCursor};

use super::Image;
use super::region::headers::{Header, Headers};
use super::region::section::Section;
use super::region::section::block::Block;
use super::region::{Region, RegionCursor};

/// A cursor over an image.
#[derive(Clone)]
pub struct ImageCursor<'a>(SegmentsCursor<'a, Segment<Region>>);

impl<'a> ImageCursor<'a> {
    /// Constructs a new image cursor.
    pub(super) fn new(image: &'a Image) -> Self {
        Self(SegmentsCursor::new(image.segments()))
    }
}

impl<'a> ImageCursor<'a> {
    /// Gets the address of the current position in the image.
    pub fn address(&self) -> Option<Address> {
        self.0
            .segments()
            .get(Address::MIN)
            .and_then(|entry| entry.segment().as_occupied())
            .and_then(Region::as_headers)
            .expect("an image always has headers at RVA 0")
            .optional()
            .image_address()
            .checked_add(self.0.position().get_addressable()?)
    }
}

impl<'a> ImageCursor<'a> {
    /// Gets the region at the cursor position.
    pub const fn region(&self) -> Option<&'a Region> {
        self.0.segment().as_occupied()
    }

    /// Gets the headers at the cursor position.
    pub const fn headers(&self) -> Option<&'a Headers> {
        let Some(region) = self.region() else {
            return None;
        };

        region.as_headers()
    }

    /// Gets the header at the cursor position.
    pub const fn header(&self) -> Option<&'a Header> {
        let Some(region) = self.0.cursor().as_occupied() else {
            return None;
        };

        let Some(headers) = region.as_headers() else {
            return None;
        };

        headers.header()
    }

    /// Gets the section at the cursor position.
    pub const fn section(&self) -> Option<&'a Section> {
        let Some(region) = self.region() else {
            return None;
        };

        region.as_section()
    }

    /// Gets the block at the cursor position.
    pub const fn block(&self) -> Option<&'a Block> {
        let Some(region) = self.0.cursor().as_occupied() else {
            return None;
        };

        let Some(section) = region.as_section() else {
            return None;
        };

        section.block()
    }

    /// Gets the unidentified region at the cursor position.
    pub const fn unidentified(&self) -> Option<&'a Unidentified> {
        let Some(region) = self.0.cursor().as_occupied() else {
            return None;
        };

        match region {
            RegionCursor::Headers(headers) => headers.unidentified(),
            RegionCursor::Section(section) => section.unidentified(),
        }
    }
}

impl<'a> Cursor for ImageCursor<'a> {
    type Error = Error;

    fn position(&self) -> Position {
        self.0.position()
    }

    fn seek(&mut self, position: Position) -> Result<(), Self::Error> {
        self.0.seek(position)
    }

    fn advance(&mut self, offset: u32) -> Result<(), Self::Error> {
        self.0.advance(offset)
    }

    fn next(&mut self) -> Result<Option<Position>, Self::Error> {
        self.0.next()
    }

    fn step(&mut self) -> Result<Option<Position>, Self::Error> {
        self.0.step()
    }
}

impl Inspect for ImageCursor<'_> {
    fn inspect(&self, inspector: &mut inspect::Inspector<'_>) -> Result<(), inspect::Error> {
        inspector.reset();

        self.0.inspect(inspector)
    }
}

impl<'a> Debug for ImageCursor<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ImageCursor")
            .field("position", &self.position())
            .field("cursor", self.0.cursor())
            .finish()
    }
}
