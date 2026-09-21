use strum::{Display, EnumIter};

use crate::memory::address::{Address, AddressSpace};
use crate::memory::extent::{Extent, Size};

/// A Windows-specific field in an Optional header.
#[derive(Clone, Copy, Debug, Display, PartialEq, Eq, EnumIter)]
pub enum Field {
    #[strum(to_string = "image_base")]
    ImageBase,
    #[strum(to_string = "section_alignment")]
    SectionAlignment,
    #[strum(to_string = "file_alignment")]
    FileAlignment,
    #[strum(to_string = "major_operating_system_version")]
    MajorOperatingSystemVersion,
    #[strum(to_string = "minor_operating_system_version")]
    MinorOperatingSystemVersion,
    #[strum(to_string = "major_image_version")]
    MajorImageVersion,
    #[strum(to_string = "minor_image_version")]
    MinorImageVersion,
    #[strum(to_string = "major_subsystem_version")]
    MajorSubsystemVersion,
    #[strum(to_string = "minor_subsystem_version")]
    MinorSubsystemVersion,
    #[strum(to_string = "win32_version_value")]
    Win32VersionValue,
    #[strum(to_string = "size_of_image")]
    SizeOfImage,
    #[strum(to_string = "size_of_headers")]
    SizeOfHeaders,
    #[strum(to_string = "check_sum")]
    CheckSum,
    #[strum(to_string = "subsystem")]
    Subsystem,
    #[strum(to_string = "dll_characteristics")]
    DllCharacteristics,
    #[strum(to_string = "size_of_stack_reserve")]
    SizeOfStackReserve,
    #[strum(to_string = "size_of_stack_commit")]
    SizeOfStackCommit,
    #[strum(to_string = "size_of_heap_reserve")]
    SizeOfHeapReserve,
    #[strum(to_string = "size_of_heap_commit")]
    SizeOfHeapCommit,
    #[strum(to_string = "loader_flags")]
    LoaderFlags,
    #[strum(to_string = "number_of_rva_and_sizes")]
    NumberOfRvaAndSizes,
}

impl Field {
    /// Gets the address of the field in the Windows-specific fields.
    const fn address(self) -> Address {
        Address::new(match self {
            Self::ImageBase => 0,
            Self::SectionAlignment => 4,
            Self::FileAlignment => 8,
            Self::MajorOperatingSystemVersion => 12,
            Self::MinorOperatingSystemVersion => 14,
            Self::MajorImageVersion => 16,
            Self::MinorImageVersion => 18,
            Self::MajorSubsystemVersion => 20,
            Self::MinorSubsystemVersion => 22,
            Self::Win32VersionValue => 24,
            Self::SizeOfImage => 28,
            Self::SizeOfHeaders => 32,
            Self::CheckSum => 36,
            Self::Subsystem => 40,
            Self::DllCharacteristics => 42,
            Self::SizeOfStackReserve => 44,
            Self::SizeOfStackCommit => 48,
            Self::SizeOfHeapReserve => 52,
            Self::SizeOfHeapCommit => 56,
            Self::LoaderFlags => 60,
            Self::NumberOfRvaAndSizes => 64,
        })
    }
}

impl Extent for Field {
    fn size(&self) -> Size {
        Size::new_valid(match self {
            Self::ImageBase => 4,
            Self::SectionAlignment => 4,
            Self::FileAlignment => 4,
            Self::MajorOperatingSystemVersion => 2,
            Self::MinorOperatingSystemVersion => 2,
            Self::MajorImageVersion => 2,
            Self::MinorImageVersion => 2,
            Self::MajorSubsystemVersion => 2,
            Self::MinorSubsystemVersion => 2,
            Self::Win32VersionValue => 4,
            Self::SizeOfImage => 4,
            Self::SizeOfHeaders => 4,
            Self::CheckSum => 4,
            Self::Subsystem => 2,
            Self::DllCharacteristics => 2,
            Self::SizeOfStackReserve => 4,
            Self::SizeOfStackCommit => 4,
            Self::SizeOfHeapReserve => 4,
            Self::SizeOfHeapCommit => 4,
            Self::LoaderFlags => 4,
            Self::NumberOfRvaAndSizes => 4,
        })
    }

    fn address_space(&self) -> AddressSpace {
        self.address()
            .to_space(self.size())
            .expect("valid field address space")
    }
}
