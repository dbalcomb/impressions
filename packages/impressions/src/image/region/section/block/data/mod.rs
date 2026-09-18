//! The data block.

mod cursor;

use std::fmt::{self, Debug};

use serde::{Deserialize, Serialize};

use crate::analysis::Completion;
use crate::memory::address::Address;
use crate::memory::cursor::AsCursor;
use crate::memory::extent::{Extent, Size};
use crate::memory::inspect::{Inspect, Inspector};

pub use self::cursor::DataCursor;

/// A block of data.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Data {
    /// A 32-bit address.
    Address(Address),
}

impl Extent for Data {
    fn size(&self) -> Size {
        match self {
            Self::Address(address) => address.size(),
        }
    }
}

impl Completion for Data {
    fn identified(&self) -> u64 {
        match self {
            Self::Address(address) => address.size().get(),
        }
    }
}

impl Inspect for Data {
    fn inspect(&self, inspector: &mut dyn Inspector) {
        match self {
            Self::Address(address) => address.inspect(inspector),
        }
    }
}

impl Debug for Data {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Address(address) => Debug::fmt(address, f),
        }
    }
}

impl AsCursor for Data {
    type Cursor<'a> = DataCursor<'a>;

    fn cursor(&self) -> Self::Cursor<'_> {
        DataCursor::new(self)
    }
}
