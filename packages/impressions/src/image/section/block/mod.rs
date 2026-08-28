//! The image file section block.

mod padding;

use std::fmt::{self, Debug};

use bytes::Bytes;
use serde::{Deserialize, Serialize};

use crate::analysis::Completion;
use crate::memory::Extent;
use crate::memory::regions::unidentified::{Error as UnidentifiedError, Unidentified};

pub use self::padding::Padding;

/// A block of memory within a section.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Block {
    /// A block of unidentified bytes.
    Unidentified(Unidentified),

    /// A block of padding.
    Padding(Padding),
}

impl Block {
    /// Constructs a new unidentified block.
    pub fn unidentified(bytes: Bytes, uninitialized: u64) -> Result<Self, UnidentifiedError> {
        Ok(Self::Unidentified(Unidentified::new(bytes, uninitialized)?))
    }

    /// Constructs a new padding block.
    pub fn padding(size: u64, value: u8) -> Self {
        Self::Padding(Padding::new(size, value))
    }
}

impl Block {
    /// Gets the block as unidentified.
    pub fn as_unidentified(&self) -> Option<&Unidentified> {
        match self {
            Self::Unidentified(unidentified) => Some(unidentified),
            _ => None,
        }
    }

    /// Checks whether the block is unidentified.
    pub fn is_unidentified(&self) -> bool {
        self.as_unidentified().is_some()
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

impl Extent for Block {
    fn size(&self) -> u64 {
        match self {
            Self::Unidentified(unidentified) => unidentified.size(),
            Self::Padding(padding) => padding.size(),
        }
    }
}

impl Completion for Block {
    fn identified(&self) -> u64 {
        match self {
            Self::Unidentified(unidentified) => unidentified.identified(),
            Self::Padding(padding) => padding.identified(),
        }
    }
}

impl Debug for Block {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unidentified(unidentified) => Debug::fmt(unidentified, f),
            Self::Padding(padding) => Debug::fmt(padding, f),
        }
    }
}
