//! The data directories within the Optional header.

pub mod directory;

use bytes::Buf;
use serde::{Deserialize, Serialize};

use crate::data::parse::Parse;
use crate::image::region::headers::Error;
use crate::memory::extent::{Extent, Size};

use self::directory::DataDirectory;

/// The data directories within the Optional header.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DataDirectories {
    count: u32,
    table: [DataDirectory; Self::MAX_COUNT],
}

impl DataDirectories {
    /// The maximum number of entries.
    const MAX_COUNT: usize = 16;
}

impl DataDirectories {
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

impl Extent for DataDirectories {
    fn size(&self) -> Size {
        let sizes = self
            .table
            .iter()
            .take(self.count as usize)
            .map(Extent::size);

        Size::try_sum(sizes).expect("sum of sizes does not exceed maximum size")
    }
}

impl Parse for DataDirectories {
    type Context<'a> = u32;
    type Error = Error;

    fn parse_with(mut buffer: impl Buf, count: Self::Context<'_>) -> Result<Self, Self::Error> {
        if count == 0 || count > Self::MAX_COUNT as u32 {
            return Err(Error::UnsupportedDataDirectoryCount(count));
        }

        Ok(Self {
            count,
            table: array_init::try_array_init(|i| match i < count as usize {
                true => DataDirectory::parse(&mut buffer),
                false => Ok(DataDirectory::default()),
            })?,
        })
    }
}
