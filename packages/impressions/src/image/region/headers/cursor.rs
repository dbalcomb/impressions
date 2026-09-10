use std::fmt::{self, Debug};

use crate::memory::cursor::{Cursor, Error, Position};
use crate::memory::regions::contiguous::Segment;
use crate::memory::regions::unidentified::Unidentified;
use crate::memory::segmented::{Segmented, SegmentsCursor};

use super::{Header, Headers};

/// A cursor over the headers in an image.
#[derive(Clone)]
pub struct HeadersCursor<'a>(SegmentsCursor<'a, Segment<Header>>);

impl<'a> HeadersCursor<'a> {
    /// Constructs a new headers cursor.
    pub(super) fn new(headers: &'a Headers) -> Self {
        Self(SegmentsCursor::new(headers.segments()))
    }
}

impl<'a> HeadersCursor<'a> {
    /// Gets the header at the cursor position.
    pub const fn header(&self) -> Option<&'a Header> {
        self.0.segment().as_identified()
    }

    /// Gets the unidentified region at the cursor position.
    pub const fn unidentified(&self) -> Option<&'a Unidentified> {
        self.0.segment().as_unidentified()
    }
}

impl<'a> Cursor for HeadersCursor<'a> {
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

impl<'a> Debug for HeadersCursor<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("HeadersCursor")
            .field("position", &self.position())
            .field("cursor", self.0.cursor())
            .finish()
    }
}
