use std::fmt::{self, Debug};

use crate::data::parse::Parse;
use crate::memory::address::Address;
use crate::memory::cursor::ops::read::{Error as ReadError, Read};
use crate::memory::cursor::{Cursor, Error, Position};
use crate::memory::extent::Extent;
use crate::memory::inspect::{Inspect, Inspector};
use crate::memory::regions::sparse::Segment;
use crate::memory::regions::unidentified::Unidentified;
use crate::memory::segmented::{Segmented, SegmentsCursor};

use super::Image;
use super::region::headers::Headers;
use super::region::headers::header::Header;
use super::region::section::Section;
use super::region::section::block::Block;
use super::region::{Region, RegionCursor};

/// A cursor over an image.
#[derive(Clone)]
pub struct ImageCursor<'a> {
    image: &'a Image,
    cursor: SegmentsCursor<'a, Segment<Region>>,
    relative: bool,
}

impl<'a> ImageCursor<'a> {
    /// Constructs a new image cursor.
    pub(super) fn new(image: &'a Image) -> Self {
        Self {
            image,
            cursor: SegmentsCursor::new(image.segments()),
            relative: false,
        }
    }
}

impl ImageCursor<'_> {
    /// Sets whether the cursor is relative to the image address.
    ///
    /// A relative address starts from `0x00000000` instead of the image base
    /// address and goes up to the image size. This is useful for working with
    /// relative virtual addresses (RVAs).
    ///
    /// Setting this to `true` will alter the behavior of the [`Self::address`]
    /// method to return a relative address, and will also affect the output of
    /// the [`Inspect`] implementation. This does not affect the position of the
    /// cursor, which is always relative to the image base address.
    pub const fn relative(mut self, relative: bool) -> Self {
        self.relative = relative;
        self
    }
}

impl ImageCursor<'_> {
    /// Gets the address of the current position in the image.
    ///
    /// This returns `None` if the cursor is at the end of the 32-bit address
    /// space.
    pub fn address(&self) -> Option<Address> {
        if self.relative {
            self.cursor.position().get_addressable().map(Into::into)
        } else {
            self.image
                .address()
                .checked_add(self.cursor.position().get_addressable()?)
        }
    }

    /// Seeks to the given address in the image.
    pub fn seek_address(&mut self, address: Address) -> Result<(), Error> {
        match self.relative {
            true => self.seek(address.into()),
            false => match address.checked_sub(self.image.address().value()) {
                Some(address) => self.seek(address.into()),
                None => Err(Error::OutOfBounds(address.into(), self.image.size())),
            },
        }
    }
}

impl<'a> ImageCursor<'a> {
    /// Gets the region at the cursor position.
    pub const fn region(&self) -> Option<&'a Region> {
        self.cursor.segment().as_occupied()
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
        let Some(region) = self.cursor.cursor().as_occupied() else {
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
        let Some(region) = self.cursor.cursor().as_occupied() else {
            return None;
        };

        let Some(section) = region.as_section() else {
            return None;
        };

        section.block()
    }

    /// Gets the unidentified region at the cursor position.
    pub const fn unidentified(&self) -> Option<&'a Unidentified> {
        let Some(region) = self.cursor.cursor().as_occupied() else {
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
        self.cursor.position()
    }

    fn seek(&mut self, position: Position) -> Result<(), Self::Error> {
        self.cursor.seek(position)
    }

    fn advance(&mut self, offset: u32) -> Result<(), Self::Error> {
        self.cursor.advance(offset)
    }

    fn next(&mut self) -> Result<Option<Position>, Self::Error> {
        self.cursor.next()
    }

    fn step(&mut self) -> Result<Option<Position>, Self::Error> {
        self.cursor.step()
    }
}

impl Read for ImageCursor<'_> {
    fn read_with<'a, T>(
        &mut self,
        context: T::Context<'a>,
    ) -> Result<T, ReadError<T::Error, Self::Error>>
    where
        T: Extent + Parse,
    {
        let Some(region) = self.cursor.cursor_mut().as_occupied_mut() else {
            return Err(ReadError::Unsupported);
        };

        region.read_with(context)
    }
}

impl Inspect for ImageCursor<'_> {
    fn inspect(&self, inspector: &mut dyn Inspector) {
        if self.relative {
            self.cursor.inspect(inspector);
        } else {
            self.cursor.inspect(&mut inspector.at(self.image.address()));
        }
    }
}

impl<'a> Debug for ImageCursor<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ImageCursor")
            .field("position", &self.position())
            .field("cursor", self.cursor.cursor())
            .finish()
    }
}
