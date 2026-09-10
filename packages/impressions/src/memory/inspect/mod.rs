//! Memory region inspection utilities.

mod error;
mod inspector;
mod output;

pub use self::error::Error;
pub use self::inspector::Inspector;
pub use self::output::Output;

/// Defines the ability to inspect a memory region.
pub trait Inspect {
    /// Inspect the region by writing a human-readable representation.
    ///
    /// Implementors of this method should only include information about the
    /// region itself, and not any of its children.
    fn inspect(&self, inspector: &mut Inspector<'_>) -> Result<(), Error>;
}
