use std::fmt::{self, Debug, Display};
use std::num::NonZeroU32;

use serde::de::Error as _;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::memory::address::{Address, AddressSpace};

use super::Error;

/// Represents the size of a memory region.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Size(InnerSize);

impl Size {
    /// The minimum size of a memory region.
    ///
    /// This is equivalent to `1`.
    pub const MIN: Self = Self(InnerSize::Value(
        NonZeroU32::new(1).expect("value is non-zero"),
    ));

    /// The maximum size of a memory region.
    ///
    /// This is equivalent to `u32::MAX as u64 + 1`.
    pub const MAX: Self = Self(InnerSize::Max);

    /// Constructs a new memory region size.
    pub const fn new(size: u64) -> Result<Self, Error> {
        match size {
            0 => Err(Error::Zero),
            size if size <= u32::MAX as u64 => Ok(Self(InnerSize::Value(
                NonZeroU32::new(size as u32).expect("value is non-zero"),
            ))),
            size if size == u32::MAX as u64 + 1 => Ok(Self(InnerSize::Max)),
            size => Err(Error::TooLarge(size)),
        }
    }
}

impl Size {
    /// Gets the size as a `u64`.
    pub const fn get(self) -> u64 {
        match self.0 {
            InnerSize::Value(size) => size.get() as u64,
            InnerSize::Max => u32::MAX as u64 + 1,
        }
    }

    /// Attempts to get the size as a `u32`.
    pub const fn get_addressable(self) -> Option<u32> {
        match self.0 {
            InnerSize::Value(size) => Some(size.get()),
            InnerSize::Max => None,
        }
    }
}

impl Size {
    /// Checks whether the size is the minimum size.
    pub const fn is_min(self) -> bool {
        matches!(self.0, InnerSize::Value(n) if n.get() == 1)
    }

    /// Checks whether the size is the maximum size.
    pub const fn is_max(self) -> bool {
        matches!(self.0, InnerSize::Max)
    }
}

impl Size {
    /// Adds the given size to this size.
    pub const fn checked_add(self, other: Self) -> Result<Self, Error> {
        match self.get().checked_add(other.get()) {
            Some(total) => Self::new(total),
            None => Err(Error::TooLarge(u64::MAX)),
        }
    }

    /// Adds the given value to this size.
    pub const fn checked_add_value(self, value: u32) -> Result<Self, Error> {
        match self.get().checked_add(value as u64) {
            Some(total) => Self::new(total),
            None => Err(Error::TooLarge(u64::MAX)),
        }
    }

    /// Subtracts the given size from this size.
    pub const fn checked_sub(self, other: Self) -> Result<Self, Error> {
        match self.get().checked_sub(other.get()) {
            Some(total) => Self::new(total),
            None => Err(Error::Zero),
        }
    }

    /// Subtracts the given value from this size.
    pub const fn checked_sub_value(self, value: u32) -> Result<Self, Error> {
        match self.get().checked_sub(value as u64) {
            Some(total) => Self::new(total),
            None => Err(Error::Zero),
        }
    }

    /// Attempts to sum the given sizes.
    pub fn try_sum(sizes: impl IntoIterator<Item = Self>) -> Result<Self, Error> {
        let mut total = 0;

        for size in sizes {
            total += size.get();

            if total > Self::MAX.get() {
                return Err(Error::TooLarge(total));
            }
        }

        Self::new(total)
    }
}

impl Size {
    /// Constructs an address space from the minimum address to the size.
    pub const fn to_address_space(self) -> AddressSpace {
        match AddressSpace::with_size(Address::MIN, self) {
            Ok(address_space) => address_space,
            Err(_) => panic!("invalid address space"),
        }
    }
}

impl Display for Size {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.get())
    }
}

impl Debug for Size {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.get())
    }
}

impl PartialEq<u32> for Size {
    fn eq(&self, other: &u32) -> bool {
        self.get_addressable() == Some(*other)
    }
}

impl PartialEq<Size> for u32 {
    fn eq(&self, other: &Size) -> bool {
        other.get_addressable() == Some(*self)
    }
}

impl PartialEq<u64> for Size {
    fn eq(&self, other: &u64) -> bool {
        self.get() == *other
    }
}

impl PartialEq<Size> for u64 {
    fn eq(&self, other: &Size) -> bool {
        *self == other.get()
    }
}

impl PartialEq<i32> for Size {
    fn eq(&self, other: &i32) -> bool {
        other.is_positive() && self.get_addressable() == Some(*other as u32)
    }
}

impl PartialEq<Size> for i32 {
    fn eq(&self, other: &Size) -> bool {
        self.is_positive() && other.get_addressable() == Some(*self as u32)
    }
}

impl PartialEq<i64> for Size {
    fn eq(&self, other: &i64) -> bool {
        other.is_positive() && self.get() == *other as u64
    }
}

impl PartialEq<Size> for i64 {
    fn eq(&self, other: &Size) -> bool {
        self.is_positive() && other.get() == *self as u64
    }
}

impl TryFrom<u32> for Size {
    type Error = Error;

    fn try_from(size: u32) -> Result<Self, Self::Error> {
        NonZeroU32::new(size)
            .map(InnerSize::Value)
            .map(Self)
            .ok_or(Error::Zero)
    }
}

impl TryFrom<u64> for Size {
    type Error = Error;

    fn try_from(size: u64) -> Result<Self, Self::Error> {
        Self::new(size)
    }
}

impl From<Size> for u64 {
    fn from(value: Size) -> Self {
        value.get()
    }
}

impl Serialize for Size {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u64(self.get())
    }
}

impl<'de> Deserialize<'de> for Size {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Self::try_from(u64::deserialize(deserializer)?).map_err(D::Error::custom)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum InnerSize {
    Value(NonZeroU32),
    Max,
}
