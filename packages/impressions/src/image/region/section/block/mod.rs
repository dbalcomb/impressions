//! The image file section block.

mod cursor;

use std::fmt::{self, Debug};

use serde::{Deserialize, Serialize};

use crate::analysis::Completion;
use crate::image::Padding;
use crate::memory::cursor::AsCursor;
use crate::memory::extent::{Extent, Size};

pub use self::cursor::BlockCursor;

/// A block of memory within a section.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Block {
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
    /// Gets the block as padding.
    pub const fn as_padding(&self) -> Option<&Padding> {
        match self {
            Self::Padding(padding) => Some(padding),
        }
    }
}

impl Block {
    /// Checks whether the block is padding.
    pub const fn is_padding(&self) -> bool {
        matches!(self, Self::Padding(_))
    }
}

impl Extent for Block {
    fn size(&self) -> Size {
        match self {
            Self::Padding(padding) => padding.size(),
        }
    }
}

impl Completion for Block {
    fn identified(&self) -> u64 {
        match self {
            Self::Padding(padding) => padding.identified(),
        }
    }
}

impl Debug for Block {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Padding(padding) => Debug::fmt(padding, f),
        }
    }
}

impl AsCursor for Block {
    #[rustfmt::skip]
    type Cursor<'a> = BlockCursor<'a>
    where
        Self: 'a;

    fn cursor(&self) -> Self::Cursor<'_> {
        BlockCursor::new(self)
    }
}
