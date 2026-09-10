use std::fmt::{self, Debug};

use serde::{Deserialize, Serialize};

use crate::analysis::Completion;
use crate::memory::cursor::AsCursor;
use crate::memory::extent::{Extent, Size};
use crate::memory::inspect::{self, Inspect};
use crate::memory::regions::unidentified::Unidentified;

use super::SegmentCursor;

/// A segment in a contiguous region of memory.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Segment<T> {
    Identified(T),
    Unidentified(Unidentified),
}

impl<T> Segment<T> {
    /// Constructs a new identified segment.
    pub const fn identified(identified: T) -> Self {
        Self::Identified(identified)
    }

    /// Constructs a new unidentified segment.
    pub const fn unidentified(unidentified: Unidentified) -> Self {
        Self::Unidentified(unidentified)
    }
}

impl<T> Segment<T> {
    /// Gets the segment as identified.
    pub const fn as_identified(&self) -> Option<&T> {
        match self {
            Self::Identified(identified) => Some(identified),
            Self::Unidentified(_) => None,
        }
    }

    /// Gets the segment as unidentified.
    pub const fn as_unidentified(&self) -> Option<&Unidentified> {
        match self {
            Self::Unidentified(unidentified) => Some(unidentified),
            Self::Identified(_) => None,
        }
    }
}

impl<T> Segment<T> {
    /// Checks whether the segment is identified.
    pub const fn is_identified(&self) -> bool {
        matches!(self, Self::Identified(_))
    }

    /// Checks whether the segment is unidentified.
    pub const fn is_unidentified(&self) -> bool {
        matches!(self, Self::Unidentified(_))
    }
}

impl<T> Extent for Segment<T>
where
    T: Extent,
{
    fn size(&self) -> Size {
        match self {
            Self::Identified(identified) => identified.size(),
            Self::Unidentified(unidentified) => unidentified.size(),
        }
    }
}

impl<T> Completion for Segment<T>
where
    T: Completion,
{
    fn identified(&self) -> u64 {
        match self {
            Self::Identified(identified) => identified.identified(),
            Self::Unidentified(unidentified) => unidentified.identified(),
        }
    }
}

impl<T> Inspect for Segment<T>
where
    T: Inspect,
{
    fn inspect(&self, inspector: &mut inspect::Inspector<'_>) -> Result<(), inspect::Error> {
        match self {
            Self::Identified(identified) => identified.inspect(inspector),
            Self::Unidentified(unidentified) => unidentified.inspect(inspector),
        }
    }
}

impl<T> Debug for Segment<T>
where
    T: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Identified(identified) => Debug::fmt(identified, f),
            Self::Unidentified(unidentified) => Debug::fmt(unidentified, f),
        }
    }
}

impl<T> AsCursor for Segment<T>
where
    T: AsCursor,
{
    #[rustfmt::skip]
    type Cursor<'a> = SegmentCursor<'a, T>
    where
        Self: 'a;

    fn cursor(&self) -> Self::Cursor<'_> {
        match self {
            Self::Identified(identified) => SegmentCursor::Identified(identified.cursor()),
            Self::Unidentified(unidentified) => SegmentCursor::Unidentified(unidentified.cursor()),
        }
    }
}

impl<T> From<Unidentified> for Segment<T> {
    fn from(unidentified: Unidentified) -> Self {
        Self::Unidentified(unidentified)
    }
}
