use std::fmt::{self, Debug};

use crate::memory::cursor::{AsCursor, Cursor, Position, SimpleCursor};
use crate::memory::inspect::{self, Inspect};
use crate::memory::regions::uninitialized::Uninitialized;

/// A cursor over a segment in a sparse region of memory.
pub enum SegmentCursor<'a, T>
where
    T: AsCursor + 'a,
{
    Occupied(T::Cursor<'a>),
    Vacant(SimpleCursor<'a, Uninitialized>),
}

impl<'a, T> SegmentCursor<'a, T>
where
    T: AsCursor,
{
    /// Gets the segment cursor as an occupied cursor.
    pub const fn as_occupied(&self) -> Option<&T::Cursor<'a>> {
        match self {
            Self::Occupied(occupied) => Some(occupied),
            Self::Vacant(_) => None,
        }
    }

    /// Gets the segment cursor as a vacant cursor.
    pub const fn as_vacant(&self) -> Option<&SimpleCursor<'a, Uninitialized>> {
        match self {
            Self::Vacant(vacant) => Some(vacant),
            Self::Occupied(_) => None,
        }
    }
}

impl<'a, T> Cursor for SegmentCursor<'a, T>
where
    T: AsCursor,
{
    type Error = <T::Cursor<'a> as Cursor>::Error;

    fn position(&self) -> Position {
        match self {
            Self::Occupied(occupied) => occupied.position(),
            Self::Vacant(vacant) => vacant.position(),
        }
    }

    fn seek(&mut self, position: Position) -> Result<(), Self::Error> {
        match self {
            Self::Occupied(occupied) => occupied.seek(position),
            Self::Vacant(vacant) => vacant.seek(position).map_err(Into::into),
        }
    }

    fn advance(&mut self, offset: u32) -> Result<(), Self::Error> {
        match self {
            Self::Occupied(occupied) => occupied.advance(offset),
            Self::Vacant(vacant) => vacant.advance(offset).map_err(Into::into),
        }
    }

    fn next(&mut self) -> Result<Option<Position>, Self::Error> {
        match self {
            Self::Occupied(occupied) => occupied.next(),
            Self::Vacant(vacant) => vacant.next().map_err(Into::into),
        }
    }

    fn step(&mut self) -> Result<Option<Position>, Self::Error> {
        match self {
            Self::Occupied(occupied) => occupied.step(),
            Self::Vacant(vacant) => vacant.step().map_err(Into::into),
        }
    }
}

impl<'a, T> Inspect for SegmentCursor<'a, T>
where
    T: AsCursor<Cursor<'a>: Inspect>,
{
    fn inspect(&self, inspector: &mut inspect::Inspector<'_>) -> Result<(), inspect::Error> {
        match self {
            Self::Occupied(occupied) => occupied.inspect(inspector),
            Self::Vacant(_) => Ok(()),
        }
    }
}

impl<'a, T> Clone for SegmentCursor<'a, T>
where
    T: AsCursor<Cursor<'a>: Clone>,
{
    fn clone(&self) -> Self {
        match self {
            Self::Occupied(occupied) => Self::Occupied(occupied.clone()),
            Self::Vacant(vacant) => Self::Vacant(vacant.clone()),
        }
    }
}

impl<'a, T> Debug for SegmentCursor<'a, T>
where
    T: AsCursor<Cursor<'a>: Debug>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Occupied(occupied) => Debug::fmt(occupied, f),
            Self::Vacant(vacant) => Debug::fmt(vacant, f),
        }
    }
}
