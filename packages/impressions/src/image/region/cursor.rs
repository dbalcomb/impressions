use std::fmt::{self, Debug};

use crate::memory::cursor::{AsCursor, Cursor, Error, Position};
use crate::memory::inspect::{self, Inspect};

use super::Region;
use super::headers::HeadersCursor;
use super::section::SectionCursor;

/// A cursor over an image region.
#[derive(Clone)]
pub enum RegionCursor<'a> {
    Headers(HeadersCursor<'a>),
    Section(SectionCursor<'a>),
}

impl<'a> RegionCursor<'a> {
    /// Constructs a new region cursor.
    pub(super) fn new(region: &'a Region) -> Self {
        match region {
            Region::Headers(headers) => Self::Headers(headers.cursor()),
            Region::Section(section) => Self::Section(section.cursor()),
        }
    }
}

impl<'a> RegionCursor<'a> {
    /// Gets the region cursor as a headers cursor.
    pub const fn as_headers(&self) -> Option<&HeadersCursor<'a>> {
        match self {
            Self::Headers(headers) => Some(headers),
            Self::Section(_) => None,
        }
    }

    /// Gets the region cursor as a section cursor.
    pub const fn as_section(&self) -> Option<&SectionCursor<'a>> {
        match self {
            Self::Section(section) => Some(section),
            Self::Headers(_) => None,
        }
    }
}

impl<'a> Cursor for RegionCursor<'a> {
    type Error = Error;

    fn position(&self) -> Position {
        match self {
            Self::Headers(headers) => headers.position(),
            Self::Section(section) => section.position(),
        }
    }

    fn seek(&mut self, position: Position) -> Result<(), Self::Error> {
        match self {
            Self::Headers(headers) => headers.seek(position),
            Self::Section(section) => section.seek(position),
        }
    }

    fn advance(&mut self, offset: u32) -> Result<(), Self::Error> {
        match self {
            Self::Headers(headers) => headers.advance(offset),
            Self::Section(section) => section.advance(offset),
        }
    }

    fn next(&mut self) -> Result<Option<Position>, Self::Error> {
        match self {
            Self::Headers(headers) => headers.next(),
            Self::Section(section) => section.next(),
        }
    }

    fn step(&mut self) -> Result<Option<Position>, Self::Error> {
        match self {
            Self::Headers(headers) => headers.step(),
            Self::Section(section) => section.step(),
        }
    }
}

impl Inspect for RegionCursor<'_> {
    fn inspect(&self, inspector: &mut inspect::Inspector<'_>) -> Result<(), inspect::Error> {
        match self {
            Self::Headers(headers) => headers.inspect(inspector),
            Self::Section(section) => section.inspect(inspector),
        }
    }
}

impl<'a> Debug for RegionCursor<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Headers(headers) => Debug::fmt(headers, f),
            Self::Section(section) => Debug::fmt(section, f),
        }
    }
}
