//! The image file section block.

pub mod data;
pub mod meta;

mod cursor;

use std::fmt::{self, Debug};

use serde::{Deserialize, Serialize};

use crate::analysis::Completion;
use crate::image::Padding;
use crate::memory::cursor::AsCursor;
use crate::memory::extent::{Extent, Size};
use crate::memory::inspect::{Inspect, Inspector};

pub use self::cursor::BlockCursor;

use self::data::Data;
use self::meta::Meta;

/// A block of memory within a section.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Block {
    /// A block of data.
    Data(Data),

    /// A block of metadata.
    Meta(Meta),

    /// A block of padding.
    Padding(Padding),
}

impl Block {
    /// Constructs a new padding block.
    pub fn padding(size: Size, value: u8) -> Self {
        Self::Padding(Padding::new(size, value))
    }
}

impl Block {
    /// Gets the block as data.
    pub const fn as_data(&self) -> Option<&Data> {
        match self {
            Self::Data(data) => Some(data),
            _ => None,
        }
    }

    /// Gets the block as metadata.
    pub const fn as_meta(&self) -> Option<&Meta> {
        match self {
            Self::Meta(meta) => Some(meta),
            _ => None,
        }
    }

    /// Gets the block as padding.
    pub const fn as_padding(&self) -> Option<&Padding> {
        match self {
            Self::Padding(padding) => Some(padding),
            _ => None,
        }
    }
}

impl Block {
    /// Checks whether the block is data.
    pub const fn is_data(&self) -> bool {
        matches!(self, Self::Data(_))
    }

    /// Checks whether the block is metadata.
    pub const fn is_meta(&self) -> bool {
        matches!(self, Self::Meta(_))
    }

    /// Checks whether the block is padding.
    pub const fn is_padding(&self) -> bool {
        matches!(self, Self::Padding(_))
    }
}

impl Extent for Block {
    fn size(&self) -> Size {
        match self {
            Self::Data(data) => data.size(),
            Self::Meta(meta) => meta.size(),
            Self::Padding(padding) => padding.size(),
        }
    }
}

impl Completion for Block {
    fn identified(&self) -> u64 {
        match self {
            Self::Data(data) => data.identified(),
            Self::Meta(meta) => meta.identified(),
            Self::Padding(padding) => padding.identified(),
        }
    }
}

impl Inspect for Block {
    fn inspect(&self, inspector: &mut dyn Inspector) {
        match self {
            Self::Data(data) => data.inspect(inspector),
            Self::Meta(meta) => meta.inspect(inspector),
            Self::Padding(padding) => padding.inspect(inspector),
        }
    }
}

impl Debug for Block {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Data(data) => Debug::fmt(data, f),
            Self::Meta(meta) => Debug::fmt(meta, f),
            Self::Padding(padding) => Debug::fmt(padding, f),
        }
    }
}

impl AsCursor for Block {
    type Cursor<'a> = BlockCursor<'a>;

    fn cursor(&self) -> Self::Cursor<'_> {
        BlockCursor::new(self)
    }
}
