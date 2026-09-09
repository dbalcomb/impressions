use std::fmt::{self, Debug};

use crate::memory::cursor::{AsCursor, Cursor, Position};
use crate::memory::regions::unidentified::Segment;
use crate::memory::segmented::SegmentsCursor;

/// A cursor over a segment in a contiguous region of memory.
pub enum SegmentCursor<'a, T>
where
    T: AsCursor + 'a,
{
    Identified(T::Cursor<'a>),
    Unidentified(SegmentsCursor<'a, Segment>),
}

impl<'a, T> Cursor for SegmentCursor<'a, T>
where
    T: AsCursor,
{
    type Error = <T::Cursor<'a> as Cursor>::Error;

    fn position(&self) -> Position {
        match self {
            Self::Identified(identified) => identified.position(),
            Self::Unidentified(unidentified) => unidentified.position(),
        }
    }

    fn seek(&mut self, position: Position) -> Result<(), Self::Error> {
        match self {
            Self::Identified(identified) => identified.seek(position),
            Self::Unidentified(unidentified) => unidentified.seek(position).map_err(Into::into),
        }
    }

    fn advance(&mut self, offset: u32) -> Result<(), Self::Error> {
        match self {
            Self::Identified(identified) => identified.advance(offset),
            Self::Unidentified(unidentified) => unidentified.advance(offset).map_err(Into::into),
        }
    }

    fn next(&mut self) -> Result<Option<Position>, Self::Error> {
        match self {
            Self::Identified(identified) => identified.next(),
            Self::Unidentified(unidentified) => unidentified.next().map_err(Into::into),
        }
    }

    fn step(&mut self) -> Result<Option<Position>, Self::Error> {
        match self {
            Self::Identified(identified) => identified.step(),
            Self::Unidentified(unidentified) => unidentified.step().map_err(Into::into),
        }
    }
}

impl<'a, T> Clone for SegmentCursor<'a, T>
where
    T: AsCursor<Cursor<'a>: Clone>,
{
    fn clone(&self) -> Self {
        match self {
            Self::Identified(identified) => Self::Identified(identified.clone()),
            Self::Unidentified(unidentified) => Self::Unidentified(unidentified.clone()),
        }
    }
}

impl<'a, T> Debug for SegmentCursor<'a, T>
where
    T: AsCursor<Cursor<'a>: Debug>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Identified(identified) => Debug::fmt(identified, f),
            Self::Unidentified(unidentified) => Debug::fmt(unidentified, f),
        }
    }
}
