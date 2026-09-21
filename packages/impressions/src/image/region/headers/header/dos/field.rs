use strum::{Display, EnumIter};

use crate::memory::address::{Address, AddressSpace};
use crate::memory::extent::{Extent, Size};

/// A field in a DOS header.
#[derive(Clone, Copy, Debug, Display, PartialEq, Eq, EnumIter)]
pub enum Field {
    #[strum(to_string = "e_magic")]
    Magic,
    #[strum(to_string = "e_cblp")]
    BytesInLastPage,
    #[strum(to_string = "e_cp")]
    Pages,
    #[strum(to_string = "e_crlc")]
    Relocations,
    #[strum(to_string = "e_cparhdr")]
    HeaderParagraphs,
    #[strum(to_string = "e_minalloc")]
    MinimumAllocation,
    #[strum(to_string = "e_maxalloc")]
    MaximumAllocation,
    #[strum(to_string = "e_ss")]
    StackSegment,
    #[strum(to_string = "e_sp")]
    StackPointer,
    #[strum(to_string = "e_csum")]
    Checksum,
    #[strum(to_string = "e_ip")]
    InstructionPointer,
    #[strum(to_string = "e_cs")]
    CodeSegment,
    #[strum(to_string = "e_lfarlc")]
    RelocationTableOffset,
    #[strum(to_string = "e_ovno")]
    OverlayNumber,
    #[strum(to_string = "e_res")]
    Reserved,
    #[strum(to_string = "e_oemid")]
    OemId,
    #[strum(to_string = "e_oeminfo")]
    OemInfo,
    #[strum(to_string = "e_res2")]
    Reserved2,
    #[strum(to_string = "e_lfanew")]
    PeHeadersOffset,
}

impl Field {
    /// Gets the address of the field in the DOS header.
    const fn address(self) -> Address {
        Address::new(match self {
            Self::Magic => 0,
            Self::BytesInLastPage => 2,
            Self::Pages => 4,
            Self::Relocations => 6,
            Self::HeaderParagraphs => 8,
            Self::MinimumAllocation => 10,
            Self::MaximumAllocation => 12,
            Self::StackSegment => 14,
            Self::StackPointer => 16,
            Self::Checksum => 18,
            Self::InstructionPointer => 20,
            Self::CodeSegment => 22,
            Self::RelocationTableOffset => 24,
            Self::OverlayNumber => 26,
            Self::Reserved => 28,
            Self::OemId => 36,
            Self::OemInfo => 38,
            Self::Reserved2 => 40,
            Self::PeHeadersOffset => 60,
        })
    }
}

impl Extent for Field {
    fn size(&self) -> Size {
        Size::new_valid(match self {
            Self::Magic => 2,
            Self::BytesInLastPage => 2,
            Self::Pages => 2,
            Self::Relocations => 2,
            Self::HeaderParagraphs => 2,
            Self::MinimumAllocation => 2,
            Self::MaximumAllocation => 2,
            Self::StackSegment => 2,
            Self::StackPointer => 2,
            Self::Checksum => 2,
            Self::InstructionPointer => 2,
            Self::CodeSegment => 2,
            Self::RelocationTableOffset => 2,
            Self::OverlayNumber => 2,
            Self::Reserved => 8,
            Self::OemId => 2,
            Self::OemInfo => 2,
            Self::Reserved2 => 20,
            Self::PeHeadersOffset => 4,
        })
    }

    fn address_space(&self) -> AddressSpace {
        self.address()
            .to_space(self.size())
            .expect("valid address space")
    }
}
