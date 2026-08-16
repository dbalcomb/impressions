//! The image file section analysis.

pub mod block;

use std::collections::BTreeMap;

use bytes::Buf;

use crate::data::types::array_string::ArrayString;
use crate::memory::address::{Address, AddressSpace};
use crate::memory::region::Region;

use self::block::Block;

use super::Error;
use super::headers::{OptionalHeader, SectionHeader};

/// A 32-bit Portable Executable (PE) image file section.
///
/// Each section is divided up into blocks of memory with the ultimate goal of
/// identifying each and every byte.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Section {
    name: ArrayString<8>,
    blocks: BTreeMap<Address, Block>,
}

impl Section {
    /// Parses the buffer for the given section header.
    pub fn parse_with(
        mut buffer: impl Buf,
        optional: &OptionalHeader,
        section: &SectionHeader,
    ) -> Result<Self, Error> {
        if section.size() == 0 {
            return Err(Error::EmptySection);
        }

        let name = *section.name();
        let address = section.address() + optional.image_address();
        let alignment = optional.section_alignment() as u64;
        let bytes = buffer.copy_to_bytes(section.file_size());
        let padding = (alignment - (section.size() % alignment)) % alignment;
        let mut blocks = BTreeMap::new();

        blocks.insert(
            address,
            Block::unknown(address.to_space(section.size())?, bytes),
        );

        if padding > 0 {
            let address = address + section.size() as u32;

            blocks.insert(address, Block::padding(address.to_space(padding)?, 0));
        }

        Ok(Self { name, blocks })
    }
}

impl Section {
    /// Gets the section name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Gets an iterator over the blocks.
    pub fn blocks(&self) -> impl Iterator<Item = &Block> {
        self.blocks.values()
    }
}

impl Region for Section {
    fn address_space(&self) -> AddressSpace {
        let first = self.blocks.values().next().expect("not empty");
        let last = self.blocks.values().last().expect("not empty");

        first.address_space().union(last.address_space())
    }
}
