//! The imports analyser.

mod error;

use std::collections::BTreeMap;

use crate::image::Image;
use crate::image::region::section::block::Block;
use crate::image::region::section::block::meta::Meta;
use crate::image::region::section::block::meta::hint_name_table::HintNameTable;
use crate::image::region::section::block::meta::hint_name_table::entry::HintName;
use crate::image::region::section::block::meta::import_address_table::ImportAddressTable;
use crate::image::region::section::block::meta::import_directory_table::ImportDirectoryTable;
use crate::image::region::section::block::meta::import_lookup_table::ImportLookupTable;
use crate::image::region::section::block::meta::import_name::ImportName;
use crate::memory::cursor::AsCursor;
use crate::memory::cursor::ops::read::Read;
use crate::memory::extent::Extent;
use crate::memory::ops::insert::Insert;

use super::Analyser;

pub use self::error::Error;

/// The imports analyser.
#[derive(Clone, Copy, Debug)]
pub struct Imports;

impl Analyser for Imports {
    type Error = Error;

    fn analyse(&self, image: &mut Image) -> Result<(), Error> {
        let Some(data_directories) = image.headers().optional().data_directories() else {
            return Ok(());
        };

        let Some(import_table) = data_directories.import_table() else {
            return Ok(());
        };

        let address = image.address() + import_table.target_address();
        let mut cursor = image.cursor();

        cursor.seek_address(address)?;

        let directory_table = cursor.read_with::<ImportDirectoryTable>(import_table)?;

        let mut lookup_tables = BTreeMap::new();

        for directory in directory_table.iter() {
            let address = image.address() + directory.lookup_table_address();

            if lookup_tables.contains_key(&address) {
                continue;
            }

            cursor.seek_address(address)?;
            lookup_tables.insert(address, cursor.read::<ImportLookupTable>()?);
        }

        let mut address_tables = BTreeMap::new();

        for directory in directory_table.iter() {
            let address = image.address() + directory.address_table_address();

            if address_tables.contains_key(&address) {
                continue;
            }

            cursor.seek_address(address)?;
            address_tables.insert(address, cursor.read::<ImportAddressTable>()?);
        }

        let mut import_names = BTreeMap::new();

        for directory in directory_table.iter() {
            let address = image.address() + directory.name_address();

            if import_names.contains_key(&address) {
                continue;
            }

            cursor.seek_address(address)?;
            import_names.insert(address, cursor.read::<ImportName>()?);
        }

        let mut hint_names = BTreeMap::new();

        for table in lookup_tables.values() {
            for lookup in table.iter() {
                let Some(&address) = lookup.as_name() else {
                    continue;
                };

                let address = image.address() + address;

                if hint_names.contains_key(&address) {
                    continue;
                }

                cursor.seek_address(address)?;
                hint_names.insert(address, cursor.read::<HintName>()?);
            }
        }

        image.insert(
            address,
            Block::Meta(Meta::ImportDirectoryTable(directory_table.clone())),
        )?;

        for (address, table) in lookup_tables {
            image.insert(address, Block::Meta(Meta::ImportLookupTable(table)))?;
        }

        for (address, table) in address_tables {
            image.insert(address, Block::Meta(Meta::ImportAddressTable(table)))?;
        }

        for (address, name) in import_names {
            image.insert(address, Block::Meta(Meta::ImportName(name)))?;
        }

        let mut table_address = None;
        let mut next_address = None;
        let mut entries = Vec::new();

        for (address, name) in hint_names {
            if !entries.is_empty() && next_address != Some(address) {
                image.insert(
                    table_address.expect("non-empty hint/name table"),
                    Block::Meta(Meta::HintNameTable(HintNameTable::new(entries))),
                )?;

                entries = Vec::new();
                table_address = None;
            }

            table_address.get_or_insert(address);
            next_address = Some(address + name.size().get() as u32);
            entries.push(name);
        }

        if !entries.is_empty() {
            image.insert(
                table_address.expect("non-empty hint/name table"),
                Block::Meta(Meta::HintNameTable(HintNameTable::new(entries))),
            )?;
        }

        Ok(())
    }
}
