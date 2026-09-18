//! The imports analyser.

mod error;

use crate::image::Image;
use crate::image::region::section::block::Block;
use crate::image::region::section::block::meta::Meta;
use crate::image::region::section::block::meta::import_directory_table::ImportDirectoryTable;
use crate::memory::cursor::{AsCursor, Read};
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

        image.insert(
            address,
            Block::Meta(Meta::ImportDirectoryTable(directory_table.clone())),
        )?;

        Ok(())
    }
}
