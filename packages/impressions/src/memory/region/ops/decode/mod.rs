//! Memory region decoding.

pub mod decoder;

mod error;

use array_init::try_array_init;

pub use self::decoder::Decoder;
pub use self::error::{ArrayDecodeError, Error};

/// Defines the decoding behavior for a memory region.
pub trait Decode: Sized {
    /// The associated context.
    type Context<'a>;

    /// The associated error.
    type Error;

    /// Decodes the region from the provided decoder with the given context.
    fn decode_with<'a>(
        decoder: &mut dyn Decoder,
        context: Self::Context<'a>,
    ) -> Result<Self, Self::Error>;

    /// Decodes the region from the provided decoder.
    fn decode(decoder: &mut dyn Decoder) -> Result<Self, Self::Error>
    where
        Self: for<'a> Decode<Context<'a> = ()>,
    {
        Self::decode_with(decoder, ())
    }
}

impl Decode for u8 {
    type Context<'a> = ();
    type Error = Error;

    fn decode_with(decoder: &mut dyn Decoder, _: Self::Context<'_>) -> Result<Self, Self::Error> {
        decoder.read_u8()
    }
}

impl Decode for u16 {
    type Context<'a> = ();
    type Error = Error;

    fn decode_with(decoder: &mut dyn Decoder, _: Self::Context<'_>) -> Result<Self, Self::Error> {
        decoder.read_u16_le()
    }
}

impl Decode for u32 {
    type Context<'a> = ();
    type Error = Error;

    fn decode_with(decoder: &mut dyn Decoder, _: Self::Context<'_>) -> Result<Self, Self::Error> {
        decoder.read_u32_le()
    }
}

impl<T, const N: usize> Decode for [T; N]
where
    T: for<'a> Decode<Context<'a>: Copy>,
{
    type Context<'a> = T::Context<'a>;
    type Error = ArrayDecodeError<T::Error, N>;

    fn decode_with(
        decoder: &mut dyn Decoder,
        context: Self::Context<'_>,
    ) -> Result<Self, Self::Error> {
        try_array_init(|index| {
            T::decode_with(decoder, context).map_err(|error| ArrayDecodeError { error, index })
        })
    }
}
