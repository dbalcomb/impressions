use std::fmt::{self, Debug};

use crate::image::Padding;
use crate::memory::cursor::{AsCursor, Cursor, Error, Position, SimpleCursor};
use crate::memory::inspect::{self, Inspect};

use super::{CoffHeader, DosHeader, Header, OptionalHeader, SectionHeader};

/// A cursor over a single header.
#[derive(Clone)]
pub enum HeaderCursor<'a> {
    Dos(SimpleCursor<'a, DosHeader>),
    Signature(SimpleCursor<'a, Header>),
    Coff(SimpleCursor<'a, CoffHeader>),
    Optional(SimpleCursor<'a, OptionalHeader>),
    Section(SimpleCursor<'a, SectionHeader>),
    Padding(SimpleCursor<'a, Padding>),
}

impl<'a> HeaderCursor<'a> {
    /// Constructs a new header cursor.
    pub(super) fn new(header: &'a Header) -> Self {
        match header {
            Header::Dos(dos) => Self::Dos(dos.cursor()),
            Header::Signature => Self::Signature(SimpleCursor::new(header)),
            Header::Coff(coff) => Self::Coff(coff.cursor()),
            Header::Optional(optional) => Self::Optional(optional.cursor()),
            Header::Section(section) => Self::Section(section.cursor()),
            Header::Padding(padding) => Self::Padding(padding.cursor()),
        }
    }
}

impl<'a> HeaderCursor<'a> {
    /// Gets the header cursor as a DOS header cursor.
    pub const fn as_dos(&self) -> Option<&SimpleCursor<'a, DosHeader>> {
        match self {
            Self::Dos(cursor) => Some(cursor),
            _ => None,
        }
    }

    /// Gets the header cursor as a signature cursor.
    pub const fn as_signature(&self) -> Option<&SimpleCursor<'a, Header>> {
        match self {
            Self::Signature(cursor) => Some(cursor),
            _ => None,
        }
    }

    /// Gets the header cursor as a COFF header cursor.
    pub const fn as_coff(&self) -> Option<&SimpleCursor<'a, CoffHeader>> {
        match self {
            Self::Coff(cursor) => Some(cursor),
            _ => None,
        }
    }

    /// Gets the header cursor as an optional header cursor.
    pub const fn as_optional(&self) -> Option<&SimpleCursor<'a, OptionalHeader>> {
        match self {
            Self::Optional(cursor) => Some(cursor),
            _ => None,
        }
    }

    /// Gets the header cursor as a section header cursor.
    pub const fn as_section(&self) -> Option<&SimpleCursor<'a, SectionHeader>> {
        match self {
            Self::Section(cursor) => Some(cursor),
            _ => None,
        }
    }

    /// Gets the header cursor as a padding cursor.
    pub const fn as_padding(&self) -> Option<&SimpleCursor<'a, Padding>> {
        match self {
            Self::Padding(cursor) => Some(cursor),
            _ => None,
        }
    }
}

impl<'a> Cursor for HeaderCursor<'a> {
    type Error = Error;

    fn position(&self) -> Position {
        match self {
            Self::Dos(cursor) => cursor.position(),
            Self::Signature(cursor) => cursor.position(),
            Self::Coff(cursor) => cursor.position(),
            Self::Optional(cursor) => cursor.position(),
            Self::Section(cursor) => cursor.position(),
            Self::Padding(cursor) => cursor.position(),
        }
    }

    fn seek(&mut self, position: Position) -> Result<(), Self::Error> {
        match self {
            Self::Dos(cursor) => cursor.seek(position),
            Self::Signature(cursor) => cursor.seek(position),
            Self::Coff(cursor) => cursor.seek(position),
            Self::Optional(cursor) => cursor.seek(position),
            Self::Section(cursor) => cursor.seek(position),
            Self::Padding(cursor) => cursor.seek(position),
        }
    }

    fn advance(&mut self, offset: u32) -> Result<(), Self::Error> {
        match self {
            Self::Dos(cursor) => cursor.advance(offset),
            Self::Signature(cursor) => cursor.advance(offset),
            Self::Coff(cursor) => cursor.advance(offset),
            Self::Optional(cursor) => cursor.advance(offset),
            Self::Section(cursor) => cursor.advance(offset),
            Self::Padding(cursor) => cursor.advance(offset),
        }
    }

    fn next(&mut self) -> Result<Option<Position>, Self::Error> {
        match self {
            Self::Dos(cursor) => cursor.next(),
            Self::Signature(cursor) => cursor.next(),
            Self::Coff(cursor) => cursor.next(),
            Self::Optional(cursor) => cursor.next(),
            Self::Section(cursor) => cursor.next(),
            Self::Padding(cursor) => cursor.next(),
        }
    }

    fn step(&mut self) -> Result<Option<Position>, Self::Error> {
        match self {
            Self::Dos(cursor) => cursor.step(),
            Self::Signature(cursor) => cursor.step(),
            Self::Coff(cursor) => cursor.step(),
            Self::Optional(cursor) => cursor.step(),
            Self::Section(cursor) => cursor.step(),
            Self::Padding(cursor) => cursor.step(),
        }
    }
}

impl Inspect for HeaderCursor<'_> {
    fn inspect(&self, inspector: &mut inspect::Inspector<'_>) -> Result<(), inspect::Error> {
        match self {
            Self::Dos(cursor) => cursor.inspect(inspector),
            Self::Signature(cursor) => cursor.inspect(inspector),
            Self::Coff(cursor) => cursor.inspect(inspector),
            Self::Optional(cursor) => cursor.inspect(inspector),
            Self::Section(cursor) => cursor.inspect(inspector),
            Self::Padding(cursor) => cursor.inspect(inspector),
        }
    }
}

impl Debug for HeaderCursor<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Dos(cursor) => Debug::fmt(cursor, f),
            Self::Signature(cursor) => Debug::fmt(cursor, f),
            Self::Coff(cursor) => Debug::fmt(cursor, f),
            Self::Optional(cursor) => Debug::fmt(cursor, f),
            Self::Section(cursor) => Debug::fmt(cursor, f),
            Self::Padding(cursor) => Debug::fmt(cursor, f),
        }
    }
}
