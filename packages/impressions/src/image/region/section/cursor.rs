use std::fmt::{self, Debug};

use crate::data::parse::Parse;
use crate::memory::cursor::ops::read::{Error as ReadError, Read};
use crate::memory::cursor::{Cursor, Error, Position};
use crate::memory::extent::Extent;
use crate::memory::inspect::{Inspect, Inspector};
use crate::memory::region::types::contiguous::Segment;
use crate::memory::region::types::segmented::{Segmented, SegmentsCursor};
use crate::memory::region::types::unidentified::Unidentified;

use super::Section;
use super::block::Block;

/// A cursor over a section in an image.
#[derive(Clone)]
pub struct SectionCursor<'a>(SegmentsCursor<'a, Segment<Block>>);

impl<'a> SectionCursor<'a> {
    /// Constructs a new section cursor.
    pub(super) fn new(section: &'a Section) -> Self {
        Self(SegmentsCursor::new(section.segments()))
    }
}

impl<'a> SectionCursor<'a> {
    /// Gets the block at the cursor position.
    pub const fn block(&self) -> Option<&'a Block> {
        self.0.segment().as_identified()
    }

    /// Gets the unidentified region at the cursor position.
    pub const fn unidentified(&self) -> Option<&'a Unidentified> {
        self.0.segment().as_unidentified()
    }
}

impl<'a> Cursor for SectionCursor<'a> {
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

impl Read for SectionCursor<'_> {
    fn read_with<'a, T>(
        &mut self,
        context: T::Context<'a>,
    ) -> Result<T, ReadError<T::Error, Self::Error>>
    where
        T: Extent + Parse,
    {
        let Some(unidentified) = self.0.cursor_mut().as_unidentified_mut() else {
            return Err(ReadError::Unsupported);
        };

        let Some(initialized) = unidentified.cursor_mut().as_initialized_mut() else {
            return Err(ReadError::Unsupported);
        };

        initialized.read_with(context)
    }
}

impl Inspect for SectionCursor<'_> {
    fn inspect(&self, inspector: &mut dyn Inspector) {
        self.0.inspect(inspector)
    }
}

impl<'a> Debug for SectionCursor<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SectionCursor")
            .field("position", &self.position())
            .field("cursor", self.0.cursor())
            .finish()
    }
}
