//! A data directory entry within the Optional header.

use bytes::Buf;
use serde::{Deserialize, Serialize};

use crate::data::parse::Parse;
use crate::image::region::headers::Error;
use crate::memory::address::Address;
use crate::memory::extent::{Extent, Size};

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

impl Extent for DataDirectory {
    fn size(&self) -> Size {
        Size::new(8).expect("valid size")
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
