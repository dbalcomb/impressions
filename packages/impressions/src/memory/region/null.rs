/// Defines a region with a null value.
pub trait Null: PartialEq + Sized {
    /// Returns the null value for the region.
    fn null() -> Self;

    /// Checks whether the region is null.
    fn is_null(&self) -> bool {
        *self == Self::null()
    }
}
