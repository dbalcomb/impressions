use std::fmt::{self, Debug};

use serde::{Deserialize, Serialize};

use crate::analysis::Completion;
use crate::memory::Extent;

use super::headers::Headers;
use super::section::Section;

/// A mapped region in a PE image.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Region {
    Headers(Headers),
    Section(Section),
}

impl Region {
    /// Constructs a new headers region.
    pub const fn headers(headers: Headers) -> Self {
        Self::Headers(headers)
    }

    /// Constructs a new section region.
    pub const fn section(section: Section) -> Self {
        Self::Section(section)
    }
}

impl Region {
    /// Gets the region as headers.
    pub const fn as_headers(&self) -> Option<&Headers> {
        match self {
            Self::Headers(headers) => Some(headers),
            Self::Section(_) => None,
        }
    }

    /// Gets the region as a section.
    pub const fn as_section(&self) -> Option<&Section> {
        match self {
            Self::Headers(_) => None,
            Self::Section(section) => Some(section),
        }
    }
}

impl Region {
    /// Checks whether this is a headers region.
    pub const fn is_headers(&self) -> bool {
        matches!(self, Self::Headers(_))
    }

    /// Checks whether this is a section region.
    pub const fn is_section(&self) -> bool {
        matches!(self, Self::Section(_))
    }
}

impl Extent for Region {
    fn size(&self) -> u64 {
        match self {
            Self::Headers(headers) => headers.size(),
            Self::Section(section) => section.size(),
        }
    }
}

impl Completion for Region {
    fn identified(&self) -> u64 {
        match self {
            Self::Headers(headers) => headers.identified(),
            Self::Section(section) => section.identified(),
        }
    }
}

impl Debug for Region {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Headers(headers) => Debug::fmt(headers, f),
            Self::Section(section) => Debug::fmt(section, f),
        }
    }
}
