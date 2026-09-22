//! Memory region encoding.

pub mod encoder;

mod error;

pub use self::encoder::Encoder;
pub use self::error::Error;

/// Defines the encoding behavior for a memory region.
pub trait Encode {
    /// Encodes the region into the provided encoder.
    fn encode(&self, encoder: &mut dyn Encoder) -> Result<(), Error>;
}

impl Encode for bool {
    fn encode(&self, encoder: &mut dyn Encoder) -> Result<(), Error> {
        encoder.write_u8(if *self { 1 } else { 0 })
    }
}

impl Encode for u8 {
    fn encode(&self, encoder: &mut dyn Encoder) -> Result<(), Error> {
        encoder.write_u8(*self)
    }
}

impl Encode for u16 {
    fn encode(&self, encoder: &mut dyn Encoder) -> Result<(), Error> {
        encoder.write_u16_le(*self)
    }
}

impl Encode for u32 {
    fn encode(&self, encoder: &mut dyn Encoder) -> Result<(), Error> {
        encoder.write_u32_le(*self)
    }
}

impl Encode for &str {
    fn encode(&self, encoder: &mut dyn Encoder) -> Result<(), Error> {
        encoder.write(self.as_bytes())
    }
}
