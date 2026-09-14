/// The status of an inspected memory region.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Status {
    /// The region's format is known.
    Identified,

    /// The region contains data whose format is not known.
    Unidentified,

    /// The region has no stored data.
    Vacant,
}
