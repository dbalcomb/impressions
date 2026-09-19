//! The import directory entry.

mod cursor;
mod field;

use bytes::Buf;
use serde::{Deserialize, Serialize};

use crate::data::parse::Parse;
use crate::image::region::section::Error;
use crate::memory::address::Address;
use crate::memory::cursor::AsCursor;
use crate::memory::extent::{Extent, FixedExtent, Size};
use crate::memory::inspect::{Inspect, Inspector};
use crate::memory::region::Null;

pub use self::cursor::ImportDirectoryCursor;
pub use self::field::Field;

/// An import directory entry.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImportDirectory {
    /// The RVA of the import lookup table.
    ///
    /// This table contains a name or ordinal for each import.
    lookup_table_address: Address,

    /// The stamp that is set to zero until the image is bound.
    ///
    /// After the image is bound, this field is set to the time/data stamp of
    /// the DLL.
    timestamp: u32,

    /// The index of the first forwarder reference.
    forwarder_chain: u32,

    /// The address of an ASCII string that contains the name of the DLL.
    ///
    /// This address is relative to the image base.
    name_address: Address,

    /// The RVA of the import address table.
    ///
    /// The contents of this table are identical to the contents of the import
    /// lookup table until the image is bound.
    address_table_address: Address,
}

impl ImportDirectory {
    /// Gets the address of the import lookup table.
    pub const fn lookup_table_address(&self) -> Address {
        self.lookup_table_address
    }

    /// Gets the address of the DLL name.
    pub const fn name_address(&self) -> Address {
        self.name_address
    }

    /// Gets the address of address table.
    pub const fn address_table_address(&self) -> Address {
        self.address_table_address
    }
}

impl FixedExtent for ImportDirectory {
    const SIZE: Size = Size::new_valid(20);
}

impl Null for ImportDirectory {
    fn null() -> Self {
        Self {
            lookup_table_address: Address::null(),
            timestamp: 0,
            forwarder_chain: 0,
            name_address: Address::null(),
            address_table_address: Address::null(),
        }
    }

    fn is_null(&self) -> bool {
        self.lookup_table_address.is_null()
            && self.timestamp == 0
            && self.forwarder_chain == 0
            && self.name_address.is_null()
            && self.address_table_address.is_null()
    }
}

impl Inspect for ImportDirectory {
    fn inspect(&self, inspector: &mut dyn Inspector) {
        inspector
            .record(self.address_space())
            .label(&"Import Directory")
            .finish()
    }
}

impl Parse for ImportDirectory {
    type Context<'a> = ();
    type Error = Error;

    fn parse_with(mut buffer: impl Buf, _: Self::Context<'_>) -> Result<Self, Self::Error> {
        Ok(Self {
            lookup_table_address: Address::parse(&mut buffer)?,
            timestamp: buffer.try_get_u32_le()?,
            forwarder_chain: buffer.try_get_u32_le()?,
            name_address: Address::parse(&mut buffer)?,
            address_table_address: Address::parse(&mut buffer)?,
        })
    }
}

impl AsCursor for ImportDirectory {
    type Cursor<'a> = ImportDirectoryCursor<'a>;

    fn cursor(&self) -> Self::Cursor<'_> {
        ImportDirectoryCursor::new(self)
    }
}
