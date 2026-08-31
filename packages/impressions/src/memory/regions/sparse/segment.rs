use std::fmt::{self, Debug};

use serde::{Deserialize, Serialize};

use crate::analysis::Completion;
use crate::memory::Extent;
use crate::memory::regions::uninitialized::Uninitialized;

/// A segment in a sparse region of memory.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Segment<T> {
    Occupied(T),
    Vacant(Uninitialized),
}

impl<T> Segment<T> {
    /// Constructs a new occupied segment.
    pub const fn occupied(region: T) -> Self {
        Self::Occupied(region)
    }

    /// Constructs a new vacant segment.
    pub const fn vacant(uninitialized: Uninitialized) -> Self {
        Self::Vacant(uninitialized)
    }
}

impl<T> Segment<T> {
    /// Gets the segment as occupied.
    pub const fn as_occupied(&self) -> Option<&T> {
        match self {
            Self::Occupied(occupied) => Some(occupied),
            Self::Vacant(_) => None,
        }
    }

    /// Gets the segment as vacant.
    pub const fn as_vacant(&self) -> Option<&Uninitialized> {
        match self {
            Self::Vacant(vacant) => Some(vacant),
            Self::Occupied(_) => None,
        }
    }
}

impl<T> Segment<T> {
    /// Checks whether the segment is occupied.
    pub const fn is_occupied(&self) -> bool {
        matches!(self, Self::Occupied(_))
    }

    /// Checks whether the segment is vacant.
    pub const fn is_vacant(&self) -> bool {
        matches!(self, Self::Vacant(_))
    }
}

impl<T> Extent for Segment<T>
where
    T: Extent,
{
    fn size(&self) -> u64 {
        match self {
            Self::Occupied(occupied) => occupied.size(),
            Self::Vacant(vacant) => vacant.size(),
        }
    }
}

impl<T> Completion for Segment<T>
where
    T: Completion,
{
    fn identified(&self) -> u64 {
        match self {
            Self::Occupied(occupied) => occupied.identified(),
            Self::Vacant(vacant) => vacant.size(),
        }
    }
}

impl<T> Debug for Segment<T>
where
    T: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Occupied(occupied) => Debug::fmt(occupied, f),
            Self::Vacant(vacant) => Debug::fmt(vacant, f),
        }
    }
}

impl<T> From<Uninitialized> for Segment<T> {
    fn from(uninitialized: Uninitialized) -> Self {
        Self::Vacant(uninitialized)
    }
}
