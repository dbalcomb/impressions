use std::fmt::{self, Debug};

use crate::image::Padding;
use crate::memory::cursor::{AsCursor, Cursor, Error, Position, SimpleCursor};
use crate::memory::inspect::{Inspect, Inspector};

use super::Block;
use super::code::CodeCursor;
use super::data::DataCursor;
use super::meta::MetaCursor;

/// A cursor over a block in a section.
#[derive(Clone)]
pub enum BlockCursor<'a> {
    Code(CodeCursor<'a>),
    Data(DataCursor<'a>),
    Meta(MetaCursor<'a>),
    Padding(SimpleCursor<'a, Padding>),
}

impl<'a> BlockCursor<'a> {
    /// Constructs a new block cursor.
    pub(super) fn new(block: &'a Block) -> Self {
        match block {
            Block::Code(code) => Self::Code(code.cursor()),
            Block::Data(data) => Self::Data(data.cursor()),
            Block::Meta(meta) => Self::Meta(meta.cursor()),
            Block::Padding(padding) => Self::Padding(padding.cursor()),
        }
    }
}

impl<'a> BlockCursor<'a> {
    /// Gets the block cursor as a code cursor.
    pub const fn as_code(&self) -> Option<&CodeCursor<'a>> {
        match self {
            Self::Code(cursor) => Some(cursor),
            _ => None,
        }
    }

    /// Gets the block cursor as a data cursor.
    pub const fn as_data(&self) -> Option<&DataCursor<'a>> {
        match self {
            Self::Data(cursor) => Some(cursor),
            _ => None,
        }
    }

    /// Gets the block cursor as a meta cursor.
    pub const fn as_meta(&self) -> Option<&MetaCursor<'a>> {
        match self {
            Self::Meta(cursor) => Some(cursor),
            _ => None,
        }
    }

    /// Gets the block cursor as a padding cursor.
    pub const fn as_padding(&self) -> Option<&SimpleCursor<'a, Padding>> {
        match self {
            Self::Padding(cursor) => Some(cursor),
            _ => None,
        }
    }
}

impl<'a> Cursor for BlockCursor<'a> {
    type Error = Error;

    fn position(&self) -> Position {
        match self {
            Self::Code(cursor) => cursor.position(),
            Self::Data(cursor) => cursor.position(),
            Self::Meta(cursor) => cursor.position(),
            Self::Padding(cursor) => cursor.position(),
        }
    }

    fn seek(&mut self, position: Position) -> Result<(), Self::Error> {
        match self {
            Self::Code(cursor) => cursor.seek(position),
            Self::Data(cursor) => cursor.seek(position),
            Self::Meta(cursor) => cursor.seek(position),
            Self::Padding(cursor) => cursor.seek(position),
        }
    }

    fn advance(&mut self, offset: u32) -> Result<(), Self::Error> {
        match self {
            Self::Code(cursor) => cursor.advance(offset),
            Self::Data(cursor) => cursor.advance(offset),
            Self::Meta(cursor) => cursor.advance(offset),
            Self::Padding(cursor) => cursor.advance(offset),
        }
    }

    fn next(&mut self) -> Result<Option<Position>, Self::Error> {
        match self {
            Self::Code(cursor) => cursor.next(),
            Self::Data(cursor) => cursor.next(),
            Self::Meta(cursor) => cursor.next(),
            Self::Padding(cursor) => cursor.next(),
        }
    }

    fn step(&mut self) -> Result<Option<Position>, Self::Error> {
        match self {
            Self::Code(cursor) => cursor.step(),
            Self::Data(cursor) => cursor.step(),
            Self::Meta(cursor) => cursor.step(),
            Self::Padding(cursor) => cursor.step(),
        }
    }
}

impl Inspect for BlockCursor<'_> {
    fn inspect(&self, inspector: &mut dyn Inspector) {
        match self {
            Self::Code(code) => code.inspect(inspector),
            Self::Data(_) => (),
            Self::Meta(meta) => meta.inspect(inspector),
            Self::Padding(_) => (),
        }
    }
}

impl Debug for BlockCursor<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Code(cursor) => Debug::fmt(cursor, f),
            Self::Data(cursor) => Debug::fmt(cursor, f),
            Self::Meta(cursor) => Debug::fmt(cursor, f),
            Self::Padding(cursor) => Debug::fmt(cursor, f),
        }
    }
}
