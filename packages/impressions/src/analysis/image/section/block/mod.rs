//! The image file section block analysis.

mod padding;
mod unknown;

use std::fmt::{self, Debug};

use bytes::Bytes;

pub use self::padding::Padding;
pub use self::unknown::Unknown;

/// A block of memory within a section.
#[derive(Clone, PartialEq, Eq)]
pub enum Block {
    /// A block of unknown bytes.
    Unknown(Unknown),

    /// A block of padding.
    Padding(Padding),
}

impl Block {
    /// Constructs a new unknown block.
    pub fn unknown(address: u32, size: u64, bytes: Bytes) -> Self {
        Self::Unknown(Unknown::new(address, size, bytes))
    }

    /// Constructs a new padding block.
    pub fn padding(address: u32, size: u64, value: u8) -> Self {
        Self::Padding(Padding::new(address, size, value))
    }
}

impl Block {
    /// Gets the address of the block.
    pub fn address(&self) -> u32 {
        match self {
            Block::Unknown(unknown) => unknown.address(),
            Block::Padding(padding) => padding.address(),
        }
    }

    /// Gets the size of the block.
    pub fn size(&self) -> u64 {
        match self {
            Block::Unknown(unknown) => unknown.size(),
            Block::Padding(padding) => padding.size(),
        }
    }

    /// Gets the block as unknown.
    pub fn as_unknown(&self) -> Option<&Unknown> {
        match self {
            Self::Unknown(unknown) => Some(unknown),
            _ => None,
        }
    }

    /// Checks whether the block is unknown.
    pub fn is_unknown(&self) -> bool {
        self.as_unknown().is_some()
    }

    /// Gets the block as padding.
    pub fn as_padding(&self) -> Option<&Padding> {
        match self {
            Self::Padding(padding) => Some(padding),
            _ => None,
        }
    }

    /// Checks whether the block is padding.
    pub fn is_padding(&self) -> bool {
        self.as_padding().is_some()
    }
}

impl Debug for Block {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unknown(unknown) => Debug::fmt(unknown, f),
            Self::Padding(padding) => Debug::fmt(padding, f),
        }
    }
}
