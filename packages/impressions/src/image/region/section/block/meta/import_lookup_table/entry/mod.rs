//! The import lookup entry.

mod error;

use std::fmt::{self, Display};

use bytes::Buf;
use serde::{Deserialize, Serialize};

use crate::data::parse::Parse;
use crate::memory::address::Address;
use crate::memory::cursor::{AsCursor, SimpleCursor};
use crate::memory::extent::{Extent, FixedExtent, Size};
use crate::memory::inspect::{Inspect, InspectionValue, Inspector};
use crate::memory::region::Null;

pub use self::error::Error;

/// A single import lookup.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ImportLookup {
    Name(Address),
    Ordinal(u16),
}

impl ImportLookup {
    /// Gets the import lookup as a name.
    pub const fn as_name(&self) -> Option<&Address> {
        match self {
            Self::Name(address) => Some(address),
            Self::Ordinal(_) => None,
        }
    }

    /// Gets the import lookup as an ordinal.
    pub const fn as_ordinal(&self) -> Option<u16> {
        match self {
            Self::Ordinal(ordinal) => Some(*ordinal),
            Self::Name(_) => None,
        }
    }
}

impl ImportLookup {
    /// Checks whether the import lookup is a name.
    pub const fn is_name(&self) -> bool {
        matches!(self, Self::Name(_))
    }

    /// Checks whether the import lookup is an ordinal.
    pub const fn is_ordinal(&self) -> bool {
        matches!(self, Self::Ordinal(_))
    }
}

impl FixedExtent for ImportLookup {
    const SIZE: Size = Size::new_valid(4);
}

impl Null for ImportLookup {
    fn null() -> Self {
        Self::Name(Address::null())
    }
}

impl Inspect for ImportLookup {
    fn inspect(&self, inspector: &mut dyn Inspector) {
        inspector
            .record(self.address_space())
            .identified()
            .label(match self {
                Self::Name(_) => &"Name",
                Self::Ordinal(_) => &"Ordinal",
            })
            .value(self)
            .finish()
    }
}

impl Display for ImportLookup {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Name(address) => Display::fmt(address, f),
            Self::Ordinal(ordinal) => Display::fmt(ordinal, f),
        }
    }
}

impl InspectionValue for ImportLookup {
    fn data_type(&self) -> &dyn Display {
        match self {
            Self::Name(_) => &"address",
            Self::Ordinal(_) => &"u16",
        }
    }
}

impl Parse for ImportLookup {
    type Context<'a> = ();
    type Error = Error;

    fn parse_with(mut buffer: impl Buf, _: Self::Context<'_>) -> Result<Self, Self::Error> {
        let value = buffer.try_get_u32_le()?;

        Ok(match value & 0x80000000 == 0 {
            true => Self::Name(Address::new(value)),
            false => Self::Ordinal(value as u16),
        })
    }
}

impl AsCursor for ImportLookup {
    type Cursor<'a> = SimpleCursor<'a, Self>;

    fn cursor(&self) -> Self::Cursor<'_> {
        SimpleCursor::new(self)
    }
}
