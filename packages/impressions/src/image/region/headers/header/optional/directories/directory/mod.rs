//! A data directory entry within the Optional header.

mod cursor;
mod field;

use bytes::Buf;
use serde::{Deserialize, Serialize};

use crate::data::parse::Parse;
use crate::image::region::headers::Error;
use crate::memory::address::Address;
use crate::memory::cursor::AsCursor;
use crate::memory::extent::{Extent, FixedExtent, Size};
use crate::memory::inspect::{Inspect, Inspector};
use crate::memory::region::ops::encode::{self, Encode};

pub use self::cursor::DataDirectoryCursor;
pub use self::field::Field;

/// A data directory entry within the Optional header.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct DataDirectory {
    /// The relative virtual address of the table.
    virtual_address: Address,

    /// The size of the table, in bytes.
    size: u32,
}

impl DataDirectory {
    /// Constructs a new data directory.
    pub const fn new(virtual_address: Address, size: u32) -> Self {
        Self {
            virtual_address,
            size,
        }
    }
}

impl DataDirectory {
    /// Gets the relative virtual address of the target.
    pub const fn target_address(&self) -> Address {
        self.virtual_address
    }

    /// Gets the size of the target, in bytes.
    pub const fn target_size(&self) -> u32 {
        self.size
    }
}

impl FixedExtent for DataDirectory {
    const SIZE: Size = Size::new_valid(8);
}

impl Inspect for DataDirectory {
    fn inspect(&self, inspector: &mut dyn Inspector) {
        inspector
            .record(self.address_space())
            .label(&"Data Directory")
            .finish()
    }
}

impl Encode for DataDirectory {
    fn encode(&self, encoder: &mut dyn encode::Encoder) -> Result<(), encode::Error> {
        self.virtual_address.encode(encoder)?;
        encoder.write_u32_le(self.size)?;

        Ok(())
    }
}

impl Parse for DataDirectory {
    type Context<'a> = ();
    type Error = Error;

    fn parse_with(mut buffer: impl Buf, _: Self::Context<'_>) -> Result<Self, Self::Error> {
        Ok(Self {
            virtual_address: Address::parse(&mut buffer)?,
            size: buffer.try_get_u32_le()?,
        })
    }
}

impl AsCursor for DataDirectory {
    type Cursor<'a> = DataDirectoryCursor<'a>;

    fn cursor(&self) -> Self::Cursor<'_> {
        DataDirectoryCursor::new(self)
    }
}
