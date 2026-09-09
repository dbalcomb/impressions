use std::fmt::{self, Debug};

use crate::memory::address::Address;
use crate::memory::cursor::{Cursor, Error, Position};
use crate::memory::regions::sparse::Segment;
use crate::memory::segmented::{Segmented, SegmentsCursor};

use super::Image;
use super::region::Region;

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

impl<'a> Debug for ImageCursor<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ImageCursor")
            .field("position", &self.position())
            .field("cursor", self.0.cursor())
            .finish()
    }
}
