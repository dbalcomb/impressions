//! The image file section block.

mod padding;
mod unknown;

use std::fmt::{self, Debug};

use bytes::Bytes;
use serde::{Deserialize, Serialize};

use crate::analysis::Completion;
use crate::memory::address::AddressSpace;
use crate::memory::region::Region;

pub use self::padding::Padding;
pub use self::unknown::Unknown;

/// A block of memory within a section.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Block {
    /// A block of unknown bytes.
    Unknown(Unknown),

    /// A block of padding.
    Padding(Padding),
}

impl Block {
    /// Constructs a new unknown block.
    pub fn unknown(address_space: AddressSpace, bytes: Bytes) -> Self {
        Self::Unknown(Unknown::new(address_space, bytes))
    }

    /// Constructs a new padding block.
    pub fn padding(address_space: AddressSpace, value: u8) -> Self {
        Self::Padding(Padding::new(address_space, value))
    }
}

impl Block {
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

impl Region for Block {
    fn address_space(&self) -> AddressSpace {
        match self {
            Self::Unknown(unknown) => unknown.address_space(),
            Self::Padding(padding) => padding.address_space(),
        }
    }
}

impl Completion for Block {
    fn identified(&self) -> u64 {
        match self {
            Self::Unknown(unknown) => unknown.identified(),
            Self::Padding(padding) => padding.identified(),
        }
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
