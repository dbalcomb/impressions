use std::fmt::{self, Debug};

use bytes::Buf;
use serde::{Deserialize, Serialize};

use crate::data::parse::Parse;
use crate::data::types::array_string::ArrayString;
use crate::memory::address::Address;

use super::Error;

/// An image file Section header.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SectionHeader {
    /// The section name.
    ///
    /// This is an 8-byte, null-padded UTF-8 string. There is no terminating
    /// null character if the string is exactly eight characters long.
    name: ArrayString<8>,

    /// The total size of the section when loaded into memory, in bytes.
    ///
    /// If this value is greater than the size of raw data then the section is
    /// filled with zeroes.
    virtual_size: u32,

    /// The address of the section when loaded into memory, relative to the
    /// image base.
    virtual_address: Address,

    /// The size of the initialized data on disk, in bytes.
    ///
    /// This value must be a multiple of the file alignment. If this value is
    /// less than the virtual size then the remainder of the section is filled
    /// with zeroes. If the section contains only uninitialized data then this
    /// value is zero.
    size_of_raw_data: u32,

    /// A file pointer to the first page within the file.
    ///
    /// This value must be a multiple of the file alignment. If a section
    /// contains only uninitialized data then this value is zero.
    pointer_to_raw_data: u32,

    /// A file pointer to the beginning of the relocation entries for the
    /// section.
    ///
    /// If there are no relocations then this value is zero.
    pointer_to_relocations: u32,

    /// A file pointer to the beginning of the line-number entries for the
    /// section.
    ///
    /// If there are no COFF line numbers then this value is zero.
    pointer_to_linenumbers: u32,

    /// The number of relocation entries for the section.
    ///
    /// This value is zero for executable images.
    number_of_relocations: u16,

    /// The number of line-number entries for the section.
    number_of_linenumbers: u16,

    /// The characteristics of the section.
    characteristics: SectionCharacteristics,
}

impl SectionHeader {
    /// The size of the section header.
    pub const SIZE: usize = 40;
}

impl SectionHeader {
    /// Gets the section name.
    pub fn name(&self) -> &ArrayString<8> {
        &self.name
    }

    /// Gets the address of the section.
    pub fn address(&self) -> Address {
        self.virtual_address
    }

    /// Gets the size of the section.
    pub fn size(&self) -> u64 {
        self.virtual_size as u64
    }

    /// Gets the file offset of the data.
    pub fn file_offset(&self) -> usize {
        self.pointer_to_raw_data as usize
    }

    /// Gets the size of the data.
    pub fn file_size(&self) -> usize {
        self.size_of_raw_data as usize
    }

    /// Gets the section characteristics.
    pub fn characteristics(&self) -> SectionCharacteristics {
        self.characteristics
    }
}

impl Parse for SectionHeader {
    type Error = Error;

    fn parse(mut buffer: impl Buf) -> Result<Self, Self::Error> {
        Ok(Self {
            name: ArrayString::parse(&mut buffer)?,
            virtual_size: buffer.try_get_u32_le()?,
            virtual_address: Address::parse(&mut buffer)?,
            size_of_raw_data: buffer.try_get_u32_le()?,
            pointer_to_raw_data: buffer.try_get_u32_le()?,
            pointer_to_relocations: buffer.try_get_u32_le()?,
            pointer_to_linenumbers: buffer.try_get_u32_le()?,
            number_of_relocations: buffer.try_get_u16_le()?,
            number_of_linenumbers: buffer.try_get_u16_le()?,
            characteristics: SectionCharacteristics::parse(&mut buffer)?,
        })
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
#[repr(transparent)]
pub struct SectionCharacteristics(u32);

impl SectionCharacteristics {
    /// The section contains executable code.
    const CNT_CODE: u32 = 0x00000020;

    /// The section contains initialized data.
    const CNT_INITIALIZED_DATA: u32 = 0x00000040;

    /// The section contains uninitialized data.
    const CNT_UNINITIALIZED_DATA: u32 = 0x00000080;

    /// The section can be discarded as needed.
    const MEM_DISCARDABLE: u32 = 0x02000000;

    /// The section cannot be cached.
    const MEM_NOT_CACHED: u32 = 0x04000000;

    /// The section is not pageable.
    const MEM_NOT_PAGED: u32 = 0x08000000;

    /// The section can be shared in memory.
    const MEM_SHARED: u32 = 0x10000000;

    /// The section can be executed as code.
    const MEM_EXECUTE: u32 = 0x20000000;

    /// The section can be read.
    const MEM_READ: u32 = 0x40000000;

    /// The section can be written to.
    const MEM_WRITE: u32 = 0x80000000;
}

impl SectionCharacteristics {
    /// Checks if the section is readable.
    pub const fn read(self) -> bool {
        self.0 & Self::MEM_READ != 0
    }

    /// Checks if the section is writable.
    pub const fn write(self) -> bool {
        self.0 & Self::MEM_WRITE != 0
    }

    /// Checks if the section is executable.
    pub const fn execute(self) -> bool {
        self.0 & Self::MEM_EXECUTE != 0
    }

    /// Checks if the section is shareable.
    pub const fn share(self) -> bool {
        self.0 & Self::MEM_SHARED != 0
    }

    /// Checks if the section is pageable.
    pub const fn page(self) -> bool {
        self.0 & Self::MEM_NOT_PAGED == 0
    }

    /// Checks if the section is cacheable.
    pub const fn cache(self) -> bool {
        self.0 & Self::MEM_NOT_CACHED == 0
    }

    /// Checks if the section is discardable.
    pub const fn discard(self) -> bool {
        self.0 & Self::MEM_DISCARDABLE != 0
    }

    /// Checks if the section contains initialised data.
    pub const fn initialised(self) -> bool {
        self.0 & Self::CNT_INITIALIZED_DATA != 0
    }

    /// Checks if the section contains uninitialised data.
    pub const fn uninitialised(self) -> bool {
        self.0 & Self::CNT_UNINITIALIZED_DATA != 0
    }

    /// Checks if the section contains code.
    pub const fn code(self) -> bool {
        self.0 & Self::CNT_CODE != 0
    }
}

impl Parse for SectionCharacteristics {
    type Error = Error;

    fn parse(mut buffer: impl Buf) -> Result<Self, Self::Error> {
        Ok(Self(buffer.try_get_u32_le()?))
    }
}

impl Debug for SectionCharacteristics {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SectionCharacteristics")
            .field("read", &self.read())
            .field("write", &self.write())
            .field("execute", &self.execute())
            .field("share", &self.share())
            .field("page", &self.page())
            .field("cache", &self.cache())
            .field("discard", &self.discard())
            .field("initialised", &self.initialised())
            .field("uninitialised", &self.uninitialised())
            .field("code", &self.code())
            .finish()
    }
}
