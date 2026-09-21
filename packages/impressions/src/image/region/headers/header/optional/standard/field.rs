use strum::{Display, EnumIter};

use crate::memory::address::{Address, AddressSpace};
use crate::memory::extent::{Extent, Size};

/// A standard field in an Optional header.
#[derive(Clone, Copy, Debug, Display, PartialEq, Eq, EnumIter)]
pub enum Field {
    #[strum(to_string = "magic")]
    Magic,
    #[strum(to_string = "major_linker_version")]
    MajorLinkerVersion,
    #[strum(to_string = "minor_linker_version")]
    MinorLinkerVersion,
    #[strum(to_string = "size_of_code")]
    SizeOfCode,
    #[strum(to_string = "size_of_initialized_data")]
    SizeOfInitializedData,
    #[strum(to_string = "size_of_uninitialized_data")]
    SizeOfUninitializedData,
    #[strum(to_string = "address_of_entry_point")]
    AddressOfEntryPoint,
    #[strum(to_string = "base_of_code")]
    BaseOfCode,
    #[strum(to_string = "base_of_data")]
    BaseOfData,
}

impl Field {
    /// Gets the address of the field in the standard fields.
    const fn address(self) -> Address {
        Address::new(match self {
            Self::Magic => 0,
            Self::MajorLinkerVersion => 2,
            Self::MinorLinkerVersion => 3,
            Self::SizeOfCode => 4,
            Self::SizeOfInitializedData => 8,
            Self::SizeOfUninitializedData => 12,
            Self::AddressOfEntryPoint => 16,
            Self::BaseOfCode => 20,
            Self::BaseOfData => 24,
        })
    }
}

impl Extent for Field {
    fn size(&self) -> Size {
        Size::new_valid(match self {
            Self::Magic => 2,
            Self::MajorLinkerVersion => 1,
            Self::MinorLinkerVersion => 1,
            Self::SizeOfCode => 4,
            Self::SizeOfInitializedData => 4,
            Self::SizeOfUninitializedData => 4,
            Self::AddressOfEntryPoint => 4,
            Self::BaseOfCode => 4,
            Self::BaseOfData => 4,
        })
    }

    fn address_space(&self) -> AddressSpace {
        self.address()
            .to_space(self.size())
            .expect("valid address space")
    }
}
