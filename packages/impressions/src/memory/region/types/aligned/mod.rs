//! A region followed by padding to a fixed alignment.

mod cursor;
mod error;

use bytes::Buf;
use serde::{Deserialize, Serialize};

use crate::analysis::Completion;
use crate::data::parse::Parse;
use crate::memory::cursor::AsCursor;
use crate::memory::extent::{Extent, Size};
use crate::memory::inspect::{Inspect, Inspector};
use crate::memory::region::ops::encode::{self, Encode};

pub use self::cursor::AlignedCursor;
pub use self::error::Error;

/// A region followed by optional padding to `ALIGNMENT` bytes.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
#[repr(transparent)]
pub struct Aligned<T, const ALIGNMENT: u32>(T);

impl<T, const ALIGNMENT: u32> Aligned<T, ALIGNMENT> {
    /// Gets the aligned region.
    pub const fn region(&self) -> &T {
        &self.0
    }
}

impl<T, const ALIGNMENT: u32> Aligned<T, ALIGNMENT>
where
    T: Extent,
{
    /// Gets the size of the padding in bytes.
    fn padding_size(&self) -> u64 {
        let remainder = self.0.size().get() % ALIGNMENT as u64;

        if remainder == 0 {
            0
        } else {
            ALIGNMENT as u64 - remainder
        }
    }
}

impl<T, const ALIGNMENT: u32> Extent for Aligned<T, ALIGNMENT>
where
    T: Extent,
{
    fn size(&self) -> Size {
        self.0
            .size()
            .checked_add_value(self.padding_size() as u32)
            .expect("valid size")
    }
}

impl<T, const ALIGNMENT: u32> Completion for Aligned<T, ALIGNMENT>
where
    T: Completion,
{
    fn identified(&self) -> u64 {
        self.0
            .identified()
            .checked_add(self.padding_size())
            .expect("valid identified size")
    }
}

impl<T, const ALIGNMENT: u32> Inspect for Aligned<T, ALIGNMENT>
where
    T: Extent + Inspect,
{
    fn inspect(&self, _: &mut dyn Inspector) {}
}

impl<T, const ALIGNMENT: u32> Encode for Aligned<T, ALIGNMENT>
where
    T: Extent + Encode,
{
    fn encode(&self, encoder: &mut dyn encode::Encoder) -> Result<(), encode::Error> {
        self.0.encode(encoder)?;

        for _ in 0..self.padding_size() {
            encoder.write_u8(0)?;
        }

        Ok(())
    }
}

impl<T, const ALIGNMENT: u32> Parse for Aligned<T, ALIGNMENT>
where
    T: Extent + Parse,
{
    type Context<'a> = T::Context<'a>;
    type Error = Error<T::Error>;

    fn parse_with(mut buffer: impl Buf, context: Self::Context<'_>) -> Result<Self, Self::Error> {
        if ALIGNMENT == 0 {
            return Err(Error::ZeroAlignment);
        }

        let region = T::parse_with(&mut buffer, context).map_err(Error::Region)?;
        let remainder = region.size().get() % ALIGNMENT as u64;
        let padding_size = if remainder == 0 {
            0
        } else {
            ALIGNMENT as u64 - remainder
        };

        for _ in 0..padding_size {
            let value = buffer.try_get_u8()?;

            if value != 0 {
                return Err(Error::NonNullPadding(value));
            }
        }

        Ok(Self(region))
    }
}

impl<T, const ALIGNMENT: u32> AsCursor for Aligned<T, ALIGNMENT>
where
    T: Extent + AsCursor,
{
    #[rustfmt::skip]
    type Cursor<'a> = AlignedCursor<'a, T, ALIGNMENT>
    where
        Self: 'a;

    fn cursor(&self) -> Self::Cursor<'_> {
        AlignedCursor::new(self)
    }
}
