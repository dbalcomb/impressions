use std::fmt::{self, Debug};

use serde::{Deserialize, Serialize};

use crate::analysis::Completion;
use crate::memory::address::AddressSpace;
use crate::memory::cursor::AsCursor;
use crate::memory::extent::{Extent, Size};
use crate::memory::inspect::{self, Inspect};
use crate::memory::regions::initialized::Initialized;
use crate::memory::regions::uninitialized::Uninitialized;
use crate::memory::slice::Slice;

use super::{Error, SegmentCursor};

/// A segment in an unidentified region of memory.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Segment {
    Initialized(Initialized),
    Uninitialized(Uninitialized),
}

impl Segment {
    /// Constructs a new initialized segment.
    pub const fn initialized(initialized: Initialized) -> Self {
        Self::Initialized(initialized)
    }

    /// Constructs a new uninitialized segment.
    pub const fn uninitialized(uninitialized: Uninitialized) -> Self {
        Self::Uninitialized(uninitialized)
    }
}

impl Segment {
    /// Gets the segment as initialized.
    pub const fn as_initialized(&self) -> Option<&Initialized> {
        match self {
            Self::Initialized(initialized) => Some(initialized),
            Self::Uninitialized(_) => None,
        }
    }

    /// Gets the segment as uninitialized.
    pub const fn as_uninitialized(&self) -> Option<&Uninitialized> {
        match self {
            Self::Uninitialized(uninitialized) => Some(uninitialized),
            Self::Initialized(_) => None,
        }
    }
}

impl Segment {
    /// Checks whether the segment is initialized.
    pub const fn is_initialized(&self) -> bool {
        matches!(self, Self::Initialized(_))
    }

    /// Checks whether the segment is uninitialized.
    pub const fn is_uninitialized(&self) -> bool {
        matches!(self, Self::Uninitialized(_))
    }
}

impl Slice for Segment {
    type Error = Error;

    fn slice(&self, address_space: AddressSpace) -> Result<Self, Self::Error> {
        match self {
            Self::Initialized(initialized) => {
                Ok(Self::Initialized(initialized.slice(address_space)?))
            }
            Self::Uninitialized(uninitialized) => {
                Ok(Self::Uninitialized(uninitialized.slice(address_space)?))
            }
        }
    }
}

impl Extent for Segment {
    fn size(&self) -> Size {
        match self {
            Self::Initialized(initialized) => initialized.size(),
            Self::Uninitialized(uninitialized) => uninitialized.size(),
        }
    }
}

impl Completion for Segment {
    fn identified(&self) -> u64 {
        match self {
            Self::Initialized(initialized) => initialized.identified(),
            Self::Uninitialized(uninitialized) => uninitialized.identified(),
        }
    }
}

impl Inspect for Segment {
    fn inspect(&self, inspector: &mut inspect::Inspector<'_>) -> Result<(), inspect::Error> {
        match self {
            Self::Initialized(initialized) => initialized.inspect(inspector),
            Self::Uninitialized(uninitialized) => uninitialized.inspect(inspector),
        }
    }
}

impl Debug for Segment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Initialized(initialized) => Debug::fmt(initialized, f),
            Self::Uninitialized(uninitialized) => Debug::fmt(uninitialized, f),
        }
    }
}

impl AsCursor for Segment {
    #[rustfmt::skip]
    type Cursor<'a> = SegmentCursor<'a>
    where
        Self: 'a;

    fn cursor(&self) -> Self::Cursor<'_> {
        match self {
            Self::Initialized(initialized) => SegmentCursor::Initialized(initialized.cursor()),
            Self::Uninitialized(uninitialized) => {
                SegmentCursor::Uninitialized(uninitialized.cursor())
            }
        }
    }
}

impl From<Initialized> for Segment {
    fn from(initialized: Initialized) -> Self {
        Self::Initialized(initialized)
    }
}

impl From<Uninitialized> for Segment {
    fn from(uninitialized: Uninitialized) -> Self {
        Self::Uninitialized(uninitialized)
    }
}
