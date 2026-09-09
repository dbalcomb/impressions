use std::fmt::{self, Debug, Display};

use crate::memory::address::Address;
use crate::memory::extent::Size;

/// A position in a memory region cursor.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Position(InnerPosition);

impl Position {
    /// The position at the start of the 32-bit address space.
    pub const START: Self = Self(InnerPosition::Address(Address::MIN));

    /// The position at the end of the 32-bit address space.
    pub const END: Self = Self(InnerPosition::End);

    /// Constructs a new memory region cursor position with the given offset.
    pub const fn new(offset: u32) -> Self {
        Self(InnerPosition::Address(Address::new(offset)))
    }
}

impl Position {
    /// Gets the position as a `u64`.
    pub const fn get(self) -> u64 {
        match self.0 {
            InnerPosition::Address(address) => address.value() as u64,
            InnerPosition::End => u32::MAX as u64 + 1,
        }
    }

    /// Attempts to get the position as a `u32`.
    pub const fn get_addressable(self) -> Option<u32> {
        match self.0 {
            InnerPosition::Address(address) => Some(address.value()),
            InnerPosition::End => None,
        }
    }
}

impl Position {
    /// Adds the given offset, returning `None` if overflow occurred.
    pub const fn checked_add(self, offset: u32) -> Option<Self> {
        let value = match self.get().checked_add(offset as u64) {
            Some(value) if value <= u32::MAX as u64 + 1 => value,
            _ => return None,
        };

        if value == u32::MAX as u64 + 1 {
            Some(Self::END)
        } else {
            Some(Self::new(value as u32))
        }
    }
}

impl Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.get())
    }
}

impl Debug for Position {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.get())
    }
}

impl From<Address> for Position {
    fn from(address: Address) -> Self {
        Self(InnerPosition::Address(address))
    }
}

impl From<Size> for Position {
    fn from(size: Size) -> Self {
        match size.get_addressable() {
            Some(value) => Self(InnerPosition::Address(Address::new(value))),
            None => Self(InnerPosition::End),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum InnerPosition {
    Address(Address),
    End,
}
