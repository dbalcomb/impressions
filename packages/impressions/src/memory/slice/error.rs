use crate::memory::address::AddressSpace;

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum Error {
    /// The requested slice is outside the source region.
    #[error("the slice {0} is out of bounds for {1}")]
    OutOfBounds(AddressSpace, AddressSpace),
}
