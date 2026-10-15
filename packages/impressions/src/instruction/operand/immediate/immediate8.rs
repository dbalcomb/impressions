use std::fmt::{self, Display};

use crate::memory::region::ops::decode::{Decode, Decoder, Error as DecodeError};
use crate::memory::region::ops::encode::{Encode, Encoder, Error as EncodeError};

/// A signed 8-bit immediate.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Immediate8(i8);

impl Immediate8 {
    /// Constructs a byte immediate.
    pub const fn new(value: i8) -> Self {
        Self(value)
    }

    /// Gets the signed value.
    pub const fn value(self) -> i8 {
        self.0
    }
}

impl Encode for Immediate8 {
    fn encode(&self, encoder: &mut dyn Encoder) -> Result<(), EncodeError> {
        encoder.write_u8(self.0 as u8)
    }
}

impl Decode for Immediate8 {
    type Context<'a> = ();
    type Error = DecodeError;

    fn decode_with(decoder: &mut dyn Decoder, _: Self::Context<'_>) -> Result<Self, Self::Error> {
        decoder.read_u8().map(|value| Self(value as i8))
    }
}

impl Display for Immediate8 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Display::fmt(&self.0, f)
    }
}
