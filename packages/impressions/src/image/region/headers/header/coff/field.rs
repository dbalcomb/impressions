use strum::{Display, EnumIter};

use crate::memory::address::{Address, AddressSpace};
use crate::memory::cursor::Position;
use crate::memory::extent::{Extent, Size};

/// A field in a COFF header.
#[derive(Clone, Copy, Debug, Display, PartialEq, Eq, EnumIter)]
pub enum Field {
    #[strum(to_string = "machine")]
    Machine,
    #[strum(to_string = "number_of_sections")]
    NumberOfSections,
    #[strum(to_string = "time_date_stamp")]
    TimeDateStamp,
    #[strum(to_string = "pointer_to_symbol_table")]
    PointerToSymbolTable,
    #[strum(to_string = "number_of_symbols")]
    NumberOfSymbols,
    #[strum(to_string = "size_of_optional_header")]
    SizeOfOptionalHeader,
    #[strum(to_string = "characteristics")]
    Characteristics,
}

impl Field {
    /// Gets the position of the field in the COFF header.
    const fn position(self) -> Position {
        Position::new(match self {
            Self::Machine => 0,
            Self::NumberOfSections => 2,
            Self::TimeDateStamp => 4,
            Self::PointerToSymbolTable => 8,
            Self::NumberOfSymbols => 12,
            Self::SizeOfOptionalHeader => 16,
            Self::Characteristics => 18,
        })
    }
}

impl Extent for Field {
    fn size(&self) -> Size {
        Size::new_valid(match self {
            Self::Machine => 2,
            Self::NumberOfSections => 2,
            Self::TimeDateStamp => 4,
            Self::PointerToSymbolTable => 4,
            Self::NumberOfSymbols => 4,
            Self::SizeOfOptionalHeader => 2,
            Self::Characteristics => 2,
        })
    }

    fn address_space(&self) -> AddressSpace {
        Address::new(self.position().get_addressable().expect("valid position"))
            .to_space(self.size())
            .expect("valid address space")
    }
}
