use std::fmt::{self, Debug};

use crate::memory::cursor::{Cursor, Error, Position, SimpleCursor};
use crate::memory::regions::initialized::Initialized;
use crate::memory::regions::uninitialized::Uninitialized;

/// A cursor over a segment in an unidentified region of memory.
#[derive(Clone)]
pub enum SegmentCursor<'a> {
    Initialized(SimpleCursor<'a, Initialized>),
    Uninitialized(SimpleCursor<'a, Uninitialized>),
}

impl<'a> SegmentCursor<'a> {
    /// Gets the segment cursor as an initialized cursor.
    pub const fn as_initialized(&self) -> Option<&SimpleCursor<'a, Initialized>> {
        match self {
            Self::Initialized(initialized) => Some(initialized),
            Self::Uninitialized(_) => None,
        }
    }

    /// Gets the segment cursor as an uninitialized cursor.
    pub const fn as_uninitialized(&self) -> Option<&SimpleCursor<'a, Uninitialized>> {
        match self {
            Self::Uninitialized(uninitialized) => Some(uninitialized),
            Self::Initialized(_) => None,
        }
    }
}

impl<'a> Cursor for SegmentCursor<'a> {
    type Error = Error;

    fn position(&self) -> Position {
        match self {
            Self::Initialized(initialized) => initialized.position(),
            Self::Uninitialized(uninitialized) => uninitialized.position(),
        }
    }

    fn seek(&mut self, position: Position) -> Result<(), Self::Error> {
        match self {
            Self::Initialized(initialized) => initialized.seek(position),
            Self::Uninitialized(uninitialized) => uninitialized.seek(position),
        }
    }

    fn advance(&mut self, offset: u32) -> Result<(), Self::Error> {
        match self {
            Self::Initialized(initialized) => initialized.advance(offset),
            Self::Uninitialized(uninitialized) => uninitialized.advance(offset),
        }
    }

    fn next(&mut self) -> Result<Option<Position>, Self::Error> {
        match self {
            Self::Initialized(initialized) => initialized.next(),
            Self::Uninitialized(uninitialized) => uninitialized.next(),
        }
    }

    fn step(&mut self) -> Result<Option<Position>, Self::Error> {
        match self {
            Self::Initialized(initialized) => initialized.step(),
            Self::Uninitialized(uninitialized) => uninitialized.step(),
        }
    }
}

impl<'a> Debug for SegmentCursor<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Initialized(initialized) => Debug::fmt(initialized, f),
            Self::Uninitialized(uninitialized) => Debug::fmt(uninitialized, f),
        }
    }
}
