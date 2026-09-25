//! The image file section block.

pub mod code;
pub mod data;
pub mod meta;

mod cursor;
mod error;

use std::fmt::{self, Debug};

use serde::{Deserialize, Serialize};

use crate::analysis::Completion;
use crate::image::Padding;
use crate::memory::cursor::AsCursor;
use crate::memory::extent::{Extent, Size};
use crate::memory::inspect::{Inspect, Inspector};
use crate::memory::region::ops::encode::{self, Encode};

pub use self::cursor::BlockCursor;
pub use self::error::Error;

use self::code::Code;
use self::data::Data;
use self::meta::Meta;

/// A block of memory within a section.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Block {
    /// A block of code.
    Code(Code),

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
    /// Gets the block as code.
    pub const fn as_code(&self) -> Option<&Code> {
        match self {
            Self::Code(code) => Some(code),
            _ => None,
        }
    }

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
    /// Checks whether the block is code.
    pub const fn is_code(&self) -> bool {
        matches!(self, Self::Data(_))
    }

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
            Self::Code(code) => code.size(),
            Self::Data(data) => data.size(),
            Self::Meta(meta) => meta.size(),
            Self::Padding(padding) => padding.size(),
        }
    }
}

impl Completion for Block {
    fn identified(&self) -> u64 {
        match self {
            Self::Code(code) => code.identified(),
            Self::Data(data) => data.identified(),
            Self::Meta(meta) => meta.identified(),
            Self::Padding(padding) => padding.identified(),
        }
    }
}

impl Inspect for Block {
    fn inspect(&self, inspector: &mut dyn Inspector) {
        match self {
            Self::Code(code) => code.inspect(inspector),
            Self::Data(data) => data.inspect(inspector),
            Self::Meta(meta) => meta.inspect(inspector),
            Self::Padding(padding) => padding.inspect(inspector),
        }
    }
}

impl Encode for Block {
    fn encode(&self, encoder: &mut dyn encode::Encoder) -> Result<(), encode::Error> {
        match self {
            Self::Code(code) => code.encode(encoder),
            Self::Data(data) => data.encode(encoder),
            Self::Meta(meta) => meta.encode(encoder),
            Self::Padding(padding) => padding.encode(encoder),
        }
    }
}

impl Debug for Block {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Code(code) => Debug::fmt(code, f),
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
