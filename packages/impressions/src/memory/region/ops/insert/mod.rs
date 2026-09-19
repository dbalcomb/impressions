//! Memory region insertion.

mod error;

use crate::memory::address::Address;

pub use self::error::Error;

/// Insert a memory region into another memory region.
pub trait Insert<T> {
    /// The associated error type for insertion operations.
    type Error;

    /// Inserts a memory region at the specified address.
    fn insert(&mut self, address: Address, region: T) -> Result<(), Self::Error>;
}
