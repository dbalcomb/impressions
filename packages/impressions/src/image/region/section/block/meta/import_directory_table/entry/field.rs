use strum::{Display, EnumIter};

use crate::memory::address::{Address, AddressSpace};
use crate::memory::extent::{Extent, Size};

/// A field in an import directory table entry.
#[derive(Clone, Copy, Debug, Display, PartialEq, Eq, EnumIter)]
pub enum Field {
    #[strum(to_string = "lookup_table_address")]
    LookupTableAddress,
    #[strum(to_string = "timestamp")]
    Timestamp,
    #[strum(to_string = "forwarder_chain")]
    ForwarderChain,
    #[strum(to_string = "name_address")]
    NameAddress,
    #[strum(to_string = "address_table_address")]
    AddressTableAddress,
}

impl Field {
    /// Gets the address of the field in the import directory table entry.
    const fn address(self) -> Address {
        Address::new(match self {
            Self::LookupTableAddress => 0,
            Self::Timestamp => 4,
            Self::ForwarderChain => 8,
            Self::NameAddress => 12,
            Self::AddressTableAddress => 16,
        })
    }
}

impl Extent for Field {
    fn size(&self) -> Size {
        Size::new_valid(match self {
            Self::LookupTableAddress => 4,
            Self::Timestamp => 4,
            Self::ForwarderChain => 4,
            Self::NameAddress => 4,
            Self::AddressTableAddress => 4,
        })
    }

    fn address_space(&self) -> AddressSpace {
        self.address()
            .to_space(self.size())
            .expect("valid address space")
    }
}
