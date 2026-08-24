use bytes::Buf;
use serde::{Deserialize, Serialize};

use crate::data::parse::Parse;
use crate::memory::address::Address;

use super::Error;

/// The signature of a 32-bit PE image file.
const OPTIONAL_SIGNATURE: u16 = 0x10b;

/// The image file Optional header.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct OptionalHeader {
    /// The image file type.
    ///
    /// This indicates whether the image is 32-bit (PE32) or 64-bit (PE32+).
    magic: u16,

    /// The major version number of the linker.
    major_linker_version: u8,

    /// The minor version number of the linker.
    minor_linker_version: u8,

    /// The sum of the size of the code sections.
    size_of_code: u32,

    /// The sum of the size of the initialized data sections.
    size_of_initialized_data: u32,

    /// The sum of the size of the uninitialized data sections.
    size_of_uninitialized_data: u32,

    /// The address of the entry function, relative to the base address.
    address_of_entry_point: Address,

    /// The address of the code section, relative to the image base.
    base_of_code: u32,

    /// The address of the data section, relative to the image base.
    base_of_data: u32,

    /// The preferred address of the image in memory.
    image_base: Address,

    /// The alignment of sections in memory.
    section_alignment: u32,

    /// The alignment of sections in the image file.
    file_alignment: u32,

    /// The major version number of the required operating system.
    major_operating_system_version: u16,

    /// The minor version number of the required operating system.
    minor_operating_system_version: u16,

    /// The major version number of the image.
    major_image_version: u16,

    /// The minor version number of the image.
    minor_image_version: u16,

    /// The major version number of the subsystem.
    major_subsystem_version: u16,

    /// The minor version number of the subsystem.
    minor_subsystem_version: u16,

    /// This member is reserved and must be 0.
    win32_version_value: u32,

    /// The size of the image, including all headers.
    size_of_image: u32,

    /// The combined size of the following items, rounded to a multiple of the
    /// file alignment.
    ///
    /// * PE Headers offset
    /// * PE Signature
    /// * Size of COFF header
    /// * Size of Optional header
    /// * Size of all section headers
    size_of_headers: u32,

    /// The image file checksum.
    check_sum: u32,

    /// The subsystem required to run the image.
    subsystem: u16,

    /// The DLL characteristics of the image.
    dll_characteristics: u16,

    /// The number of bytes to reserve for the stack.
    size_of_stack_reserve: u32,

    /// The number of bytes to commit for the stack.
    size_of_stack_commit: u32,

    /// The number of bytes to reserve for the local heap.
    size_of_heap_reserve: u32,

    /// The number of bytes to commit for the local heap.
    size_of_heap_commit: u32,

    /// This member is obsolete.
    loader_flags: u32,

    /// The data directories.
    data_directories: DataDirectoryTable,
}

impl OptionalHeader {
    /// The base size of the header without data directories, in bytes.
    pub const BASE_SIZE: usize = 96;
}

impl OptionalHeader {
    /// Gets the base address of the image.
    pub fn image_address(&self) -> Address {
        self.image_base
    }

    /// Gets the total size of the image.
    pub fn image_size(&self) -> u64 {
        self.size_of_image as u64
    }

    /// Gets the total size of the headers.
    pub fn headers_size(&self) -> u64 {
        self.size_of_headers as u64
    }

    /// Gets the section alignment.
    pub fn section_alignment(&self) -> u32 {
        self.section_alignment
    }

    /// Gets the data directories.
    pub fn data_directories(&self) -> &DataDirectoryTable {
        &self.data_directories
    }
}

impl Parse for OptionalHeader {
    type Context<'a> = ();
    type Error = Error;

    fn parse_with(mut buffer: impl Buf, _: Self::Context<'_>) -> Result<Self, Self::Error> {
        let magic = buffer.try_get_u16_le()?;

        if magic != OPTIONAL_SIGNATURE {
            return Err(Error::UnsupportedArchitecture);
        }

        Ok(Self {
            magic,
            major_linker_version: buffer.try_get_u8()?,
            minor_linker_version: buffer.try_get_u8()?,
            size_of_code: buffer.try_get_u32_le()?,
            size_of_initialized_data: buffer.try_get_u32_le()?,
            size_of_uninitialized_data: buffer.try_get_u32_le()?,
            address_of_entry_point: Address::parse(&mut buffer)?,
            base_of_code: buffer.try_get_u32_le()?,
            base_of_data: buffer.try_get_u32_le()?,
            image_base: Address::parse(&mut buffer)?,
            section_alignment: buffer.try_get_u32_le()?,
            file_alignment: buffer.try_get_u32_le()?,
            major_operating_system_version: buffer.try_get_u16_le()?,
            minor_operating_system_version: buffer.try_get_u16_le()?,
            major_image_version: buffer.try_get_u16_le()?,
            minor_image_version: buffer.try_get_u16_le()?,
            major_subsystem_version: buffer.try_get_u16_le()?,
            minor_subsystem_version: buffer.try_get_u16_le()?,
            win32_version_value: buffer.try_get_u32_le()?,
            size_of_image: buffer.try_get_u32_le()?,
            size_of_headers: buffer.try_get_u32_le()?,
            check_sum: buffer.try_get_u32_le()?,
            subsystem: buffer.try_get_u16_le()?,
            dll_characteristics: buffer.try_get_u16_le()?,
            size_of_stack_reserve: buffer.try_get_u32_le()?,
            size_of_stack_commit: buffer.try_get_u32_le()?,
            size_of_heap_reserve: buffer.try_get_u32_le()?,
            size_of_heap_commit: buffer.try_get_u32_le()?,
            loader_flags: buffer.try_get_u32_le()?,
            data_directories: DataDirectoryTable::parse(&mut buffer)?,
        })
    }
}

/// The data directory table within the Optional header.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DataDirectoryTable {
    count: u32,
    table: [DataDirectory; Self::MAX_SIZE],
}

impl DataDirectoryTable {
    /// The maximum number of entries in the table.
    const MAX_SIZE: usize = 16;
}

impl DataDirectoryTable {
    /// Gets the export table data directory.
    pub fn export_table(&self) -> Option<&DataDirectory> {
        self.get(0)
    }

    /// Gets the import table data directory.
    pub fn import_table(&self) -> Option<&DataDirectory> {
        self.get(1)
    }

    /// Gets the resource table data directory.
    pub fn resource_table(&self) -> Option<&DataDirectory> {
        self.get(2)
    }

    /// Gets the exception table data directory.
    pub fn exception_table(&self) -> Option<&DataDirectory> {
        self.get(3)
    }

    /// Gets the certificate table data directory.
    pub fn certificate_table(&self) -> Option<&DataDirectory> {
        self.get(4)
    }

    /// Gets the base relocation table data directory.
    pub fn base_relocation_table(&self) -> Option<&DataDirectory> {
        self.get(5)
    }

    /// Gets the debug data directory.
    pub fn debug(&self) -> Option<&DataDirectory> {
        self.get(6)
    }

    /// Gets the global pointer data directory.
    pub fn global_pointer(&self) -> Option<&DataDirectory> {
        self.get(8)
    }

    /// Gets the thread local storage (TLS) table data directory.
    pub fn tls_table(&self) -> Option<&DataDirectory> {
        self.get(9)
    }

    /// Gets the load configuration table data directory.
    pub fn load_config_table(&self) -> Option<&DataDirectory> {
        self.get(10)
    }

    /// Gets the bound import table data directory.
    pub fn bound_import_table(&self) -> Option<&DataDirectory> {
        self.get(11)
    }

    /// Gets the import address table data directory.
    pub fn import_address_table(&self) -> Option<&DataDirectory> {
        self.get(12)
    }

    /// Gets the delay import descriptor data directory.
    pub fn delay_import_descriptor(&self) -> Option<&DataDirectory> {
        self.get(13)
    }

    /// Gets the data directory at the given index.
    fn get(&self, index: usize) -> Option<&DataDirectory> {
        self.table.get(index).and_then(|data_directory| {
            match data_directory.target_address().value() != 0 {
                true => Some(data_directory),
                false => None,
            }
        })
    }
}

impl DataDirectoryTable {
    /// Gets the number of data directory entries included in the image.
    pub const fn count(&self) -> usize {
        self.count as usize
    }
}

impl Parse for DataDirectoryTable {
    type Context<'a> = ();
    type Error = Error;

    fn parse_with(mut buffer: impl Buf, _: Self::Context<'_>) -> Result<Self, Self::Error> {
        let count = buffer.try_get_u32_le()?;

        assert!(count as usize <= Self::MAX_SIZE);

        Ok(Self {
            count,
            table: array_init::try_array_init(|i| match i < count as usize {
                true => DataDirectory::parse(&mut buffer),
                false => Ok(DataDirectory {
                    target_address: Address::new(0),
                    target_size: 0,
                }),
            })?,
        })
    }
}

/// A data directory entry within the Optional header.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DataDirectory {
    /// The relative virtual address of the target.
    target_address: Address,

    /// The size of the target, in bytes.
    target_size: u32,
}

impl DataDirectory {
    /// The size of the data directory entry, in bytes.
    pub const SIZE: usize = 8;
}

impl DataDirectory {
    /// Gets the relative virtual address of the target.
    pub const fn target_address(&self) -> Address {
        self.target_address
    }

    /// Gets the size of the target, in bytes.
    pub const fn target_size(&self) -> u64 {
        self.target_size as u64
    }
}

impl Parse for DataDirectory {
    type Context<'a> = ();
    type Error = Error;

    fn parse_with(mut buffer: impl Buf, _: Self::Context<'_>) -> Result<Self, Self::Error> {
        Ok(Self {
            target_address: Address::parse(&mut buffer)?,
            target_size: buffer.try_get_u32_le()?,
        })
    }
}
