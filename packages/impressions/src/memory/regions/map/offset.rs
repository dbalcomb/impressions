use std::ops::Deref;

/// A memory region offset.
pub struct Offset<'a, T> {
    region: &'a T,
    offset: u32,
}

impl<'a, T> Offset<'a, T> {
    /// Constructs a new memory region offset.
    pub(crate) const fn new(region: &'a T, offset: u32) -> Self {
        Self { region, offset }
    }
}

impl<'a, T> Offset<'a, T> {
    /// Gets the memory region for this offset.
    pub const fn region(&self) -> &'a T {
        self.region
    }

    /// Gets the offset into the memory region.
    pub const fn offset(&self) -> u32 {
        self.offset
    }
}

impl<T> Deref for Offset<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.region
    }
}
