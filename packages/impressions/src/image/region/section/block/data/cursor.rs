use std::fmt::{self, Debug};

use crate::memory::address::Address;
use crate::memory::cursor::{Cursor, Error, Position, SimpleCursor};
use crate::memory::inspect::{self, Inspect};

use super::Data;

/// A cursor over a block of data.
#[derive(Clone)]
pub enum DataCursor<'a> {
    Address(SimpleCursor<'a, Address>),
}

impl<'a> DataCursor<'a> {
    /// Constructs a new data cursor.
    pub(super) fn new(data: &'a Data) -> Self {
        match data {
            Data::Address(address) => Self::Address(SimpleCursor::new(address)),
        }
    }
}

impl Cursor for DataCursor<'_> {
    type Error = Error;

    fn position(&self) -> Position {
        match self {
            Self::Address(cursor) => cursor.position(),
        }
    }

    fn seek(&mut self, position: Position) -> Result<(), Self::Error> {
        match self {
            Self::Address(cursor) => cursor.seek(position),
        }
    }

    fn advance(&mut self, offset: u32) -> Result<(), Self::Error> {
        match self {
            Self::Address(cursor) => cursor.advance(offset),
        }
    }

    fn next(&mut self) -> Result<Option<Position>, Self::Error> {
        match self {
            Self::Address(cursor) => cursor.next(),
        }
    }

    fn step(&mut self) -> Result<Option<Position>, Self::Error> {
        match self {
            Self::Address(cursor) => cursor.step(),
        }
    }
}

impl Inspect for DataCursor<'_> {
    fn inspect(&self, _: &mut dyn inspect::Inspector) {
        match self {
            Self::Address(_) => (),
        }
    }
}

impl Debug for DataCursor<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Address(cursor) => Debug::fmt(cursor, f),
        }
    }
}
