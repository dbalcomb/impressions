use strum::{Display, EnumIter};

use crate::memory::address::{Address, AddressSpace};
use crate::memory::extent::{Extent, Size};

/// A field in a data directory entry.
#[derive(Clone, Copy, Debug, Display, PartialEq, Eq, EnumIter)]
pub enum Field {
    #[strum(to_string = "virtual_address")]
    VirtualAddress,
    #[strum(to_string = "size")]
    Size,
}

impl Field {
    /// Gets the address of the field in the data directory entry.
    const fn address(self) -> Address {
        Address::new(match self {
            Self::VirtualAddress => 0,
            Self::Size => 4,
        })
    }
}

impl Extent for Field {
    fn size(&self) -> Size {
        Size::new_valid(match self {
            Self::VirtualAddress => 4,
            Self::Size => 4,
        })
    }

    fn address_space(&self) -> AddressSpace {
        self.address()
            .to_space(self.size())
            .expect("valid address space")
    }
}
