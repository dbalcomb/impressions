//! Memory address representation and manipulation.

mod space;

use std::fmt::{self, Debug, Display};
use std::ops::{Add, Sub};

use bytes::{Buf, TryGetError};

use crate::data::parse::Parse;

pub use self::space::{AddressSpace, Error as AddressSpaceError};

/// Represents an address in memory.
///
/// An address is a 32-bit unsigned integer that can be used to access memory
/// locations. There is no distinction between relative and absolute virtual
/// memory addresses so care must be taken not to confuse the two.
#[derive(Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct Address(u32);

impl Address {
    /// The smallest address.
    pub const MIN: Address = Address::new(0);

    /// The largest address.
    pub const MAX: Address = Address::new(u32::MAX);
}

impl Address {
    /// Constructs a new memory address.
    pub const fn new(address: u32) -> Self {
        Self(address)
    }

    /// Constructs a new memory address from the given little-endian bytes.
    pub const fn from_bytes(bytes: [u8; 4]) -> Self {
        Self(u32::from_le_bytes(bytes))
    }

    /// Gets the little-endian byte representation of this address.
    pub const fn to_bytes(&self) -> [u8; 4] {
        self.0.to_le_bytes()
    }

    /// Gets the value of this address.
    pub const fn value(&self) -> u32 {
        self.0
    }
}

impl Address {
    /// Adds the given offset, returning `None` if overflow occurred.
    pub const fn checked_add(self, offset: u32) -> Option<Self> {
        match self.0.checked_add(offset) {
            Some(address) => Some(Self(address)),
            None => None,
        }
    }

    /// Subtracts the given offset, returning `None` if overflow occurred.
    pub const fn checked_sub(self, offset: u32) -> Option<Self> {
        match self.0.checked_sub(offset) {
            Some(address) => Some(Self(address)),
            None => None,
        }
    }
}

impl Parse for Address {
    type Error = TryGetError;

    fn parse(mut buffer: impl Buf) -> Result<Self, Self::Error> {
        buffer.try_get_u32_le().map(Self)
    }
}

impl Add for Address {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl Add<u32> for Address {
    type Output = Self;

    fn add(self, rhs: u32) -> Self::Output {
        Self(self.0 + rhs)
    }
}

impl Sub for Address {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0)
    }
}

impl Sub<u32> for Address {
    type Output = Self;

    fn sub(self, rhs: u32) -> Self::Output {
        Self(self.0 - rhs)
    }
}

impl Display for Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "0x{:0>8x}", self.0)
    }
}

impl Debug for Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "0x{:0>8x}", self.0)
    }
}

impl From<u32> for Address {
    fn from(address: u32) -> Self {
        Self(address)
    }
}

impl From<[u8; 4]> for Address {
    fn from(bytes: [u8; 4]) -> Self {
        Self::from_bytes(bytes)
    }
}

impl From<Address> for u32 {
    fn from(address: Address) -> Self {
        address.0
    }
}

impl From<Address> for [u8; 4] {
    fn from(address: Address) -> Self {
        address.to_bytes()
    }
}

#[cfg(test)]
mod tests {
    use crate::data::parse::Parse;

    use super::Address;

    #[test]
    fn test_arithmetic() {
        let address = Address::new(0x00400000);

        assert_eq!(address + 0x00001000, Address::new(0x00401000));
        assert_eq!(address - 0x00001000, Address::new(0x003ff000));

        assert_eq!(address + Address::new(0x00002000), Address::new(0x00402000));
        assert_eq!(address - Address::new(0x00002000), Address::new(0x003fe000));
    }

    #[test]
    fn test_parse() {
        let mut buffer = [0, 0, 64, 0, 0, 16, 64, 0].as_slice();

        assert_eq!(Address::parse(&mut buffer), Ok(Address::new(0x00400000)));
        assert_eq!(buffer, [0, 16, 64, 0]);
        assert_eq!(Address::parse(&mut buffer), Ok(Address::new(0x00401000)));
        assert_eq!(buffer, []);
    }
}
