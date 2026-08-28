//! A region of uninitialized memory.

use serde::{Deserialize, Serialize};

use crate::analysis::Completion;
use crate::memory::Extent;

/// A region of uninitialized memory.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
#[repr(transparent)]
pub struct Uninitialized(u64);

impl Uninitialized {
    /// Constructs a new uninitialized region.
    pub const fn new(size: u64) -> Self {
        Self(size)
    }
}

impl Extent for Uninitialized {
    fn size(&self) -> u64 {
        self.0
    }
}

impl Completion for Uninitialized {
    fn identified(&self) -> u64 {
        0
    }
}
