//! The memory region encoder.

mod fs;
mod truncating;

use super::Error;

pub use self::fs::FileEncoder;
pub use self::truncating::TruncatingEncoder;

/// Defines the behavior of a memory region encoder.
pub trait Encoder {
    /// Writes a byte slice into the encoder.
    fn write(&mut self, bytes: &[u8]) -> Result<(), Error>;

    /// Writes a `u8` value.
    fn write_u8(&mut self, n: u8) -> Result<(), Error> {
        self.write(&[n])
    }

    /// Writes a `u16` value in little-endian format.
    fn write_u16_le(&mut self, n: u16) -> Result<(), Error> {
        self.write(&n.to_le_bytes())
    }

    /// Writes a `u32` value in little-endian format.
    fn write_u32_le(&mut self, n: u32) -> Result<(), Error> {
        self.write(&n.to_le_bytes())
    }

    /// Writes a `u64` value in little-endian format.
    fn write_u64_le(&mut self, n: u64) -> Result<(), Error> {
        self.write(&n.to_le_bytes())
    }
}
