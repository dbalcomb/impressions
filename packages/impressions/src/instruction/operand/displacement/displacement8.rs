use crate::memory::region::ops::decode::{Decode, Decoder, Error as DecodeError};
use crate::memory::region::ops::encode::{Encode, Encoder, Error as EncodeError};

/// A signed 8-bit byte displacement.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Displacement8(i8);

impl Displacement8 {
    /// Constructs a byte displacement.
    pub const fn new(value: i8) -> Self {
        Self(value)
    }

    /// Gets the signed value.
    pub const fn value(self) -> i8 {
        self.0
    }
}

impl Encode for Displacement8 {
    fn encode(&self, encoder: &mut dyn Encoder) -> Result<(), EncodeError> {
        encoder.write_u8(self.0 as u8)
    }
}

impl Decode for Displacement8 {
    type Context<'a> = ();
    type Error = DecodeError;

    fn decode_with(decoder: &mut dyn Decoder, _: Self::Context<'_>) -> Result<Self, Self::Error> {
        decoder.read_u8().map(|value| Self(value as i8))
    }
}
