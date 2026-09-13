use strum::{Display, EnumIter};

use crate::memory::address::{Address, AddressSpace};
use crate::memory::cursor::Position;
use crate::memory::extent::{Extent, Size};

/// A field in a section header.
#[derive(Clone, Copy, Debug, Display, PartialEq, Eq, EnumIter)]
pub enum Field {
    #[strum(to_string = "name")]
    Name,
    #[strum(to_string = "virtual_size")]
    VirtualSize,
    #[strum(to_string = "virtual_address")]
    VirtualAddress,
    #[strum(to_string = "size_of_raw_data")]
    SizeOfRawData,
    #[strum(to_string = "pointer_to_raw_data")]
    PointerToRawData,
    #[strum(to_string = "pointer_to_relocations")]
    PointerToRelocations,
    #[strum(to_string = "pointer_to_linenumbers")]
    PointerToLinenumbers,
    #[strum(to_string = "number_of_relocations")]
    NumberOfRelocations,
    #[strum(to_string = "number_of_linenumbers")]
    NumberOfLinenumbers,
    #[strum(to_string = "characteristics")]
    Characteristics,
}

impl Field {
    /// Gets the position of the field in the section header.
    const fn position(&self) -> Position {
        Position::new(match self {
            Self::Name => 0,
            Self::VirtualSize => 8,
            Self::VirtualAddress => 12,
            Self::SizeOfRawData => 16,
            Self::PointerToRawData => 20,
            Self::PointerToRelocations => 24,
            Self::PointerToLinenumbers => 28,
            Self::NumberOfRelocations => 32,
            Self::NumberOfLinenumbers => 34,
            Self::Characteristics => 36,
        })
    }
}

impl Extent for Field {
    fn size(&self) -> Size {
        Size::new_valid(match self {
            Self::Name => 8,
            Self::VirtualSize => 4,
            Self::VirtualAddress => 4,
            Self::SizeOfRawData => 4,
            Self::PointerToRawData => 4,
            Self::PointerToRelocations => 4,
            Self::PointerToLinenumbers => 4,
            Self::NumberOfRelocations => 2,
            Self::NumberOfLinenumbers => 2,
            Self::Characteristics => 4,
        })
    }

    fn address_space(&self) -> AddressSpace {
        Address::new(self.position().get_addressable().expect("valid position"))
            .to_space(self.size())
            .expect("valid address space")
    }
}
