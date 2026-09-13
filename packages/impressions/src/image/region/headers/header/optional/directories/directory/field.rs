use strum::{Display, EnumIter};

use crate::memory::address::{Address, AddressSpace};
use crate::memory::cursor::Position;
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
    /// Gets the position of the field in the data directory entry.
    const fn position(self) -> Position {
        Position::new(match self {
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
        Address::new(self.position().get_addressable().expect("valid position"))
            .to_space(self.size())
            .expect("valid address space")
    }
}
