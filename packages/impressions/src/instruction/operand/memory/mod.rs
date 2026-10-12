//! Memory operands.

mod memory32;
mod scale;
mod scaled_index;

pub use self::memory32::Memory32;
pub use self::scale::Scale;
pub use self::scaled_index::ScaledIndex;
