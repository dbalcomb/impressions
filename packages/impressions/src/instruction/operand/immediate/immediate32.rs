use std::fmt::{self, Display};

use crate::memory::region::ops::decode::{Decode, Decoder, Error as DecodeError};
use crate::memory::region::ops::encode::{Encode, Encoder, Error as EncodeError};

/// A signed 32-bit immediate.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Immediate32(i32);

impl Immediate32 {
    /// Constructs a dword immediate.
    pub const fn new(value: i32) -> Self {
        Self(value)
    }

    /// Gets the signed value.
    pub const fn value(self) -> i32 {
        self.0
    }
}

impl Encode for Immediate32 {
    fn encode(&self, encoder: &mut dyn Encoder) -> Result<(), EncodeError> {
        encoder.write_u32_le(self.0 as u32)
    }
}

impl Decode for Immediate32 {
    type Context<'a> = ();
    type Error = DecodeError;

    fn decode_with(decoder: &mut dyn Decoder, _: Self::Context<'_>) -> Result<Self, Self::Error> {
        decoder.read_u32_le().map(|value| Self(value as i32))
    }
}

impl Display for Immediate32 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Display::fmt(&self.0, f)
    }
}
