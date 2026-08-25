//! The image file section.

pub mod block;

use bytes::Buf;
use serde::{Deserialize, Serialize};

use crate::analysis::Completion;
use crate::data::parse::Parse;
use crate::data::types::array_string::ArrayString;
use crate::memory::Extent;
use crate::memory::map::{Iter, Map};

use self::block::Block;

use super::Error;
use super::headers::{OptionalHeader, SectionCharacteristics, SectionHeader};

/// A 32-bit Portable Executable (PE) image file section.
///
/// Each section is divided up into blocks of memory with the ultimate goal of
/// identifying each and every byte.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Section {
    name: ArrayString<8>,
    characteristics: SectionCharacteristics,
    blocks: Map<Block>,
}

impl Section {
    /// Gets the section name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Gets an iterator over the blocks.
    pub fn blocks(&self) -> Iter<'_, Block> {
        self.blocks.iter()
    }
}

impl Extent for Section {
    fn size(&self) -> u64 {
        self.blocks.size()
    }
}

impl Completion for Section {
    fn identified(&self) -> u64 {
        self.blocks().map(Block::identified).sum()
    }
}

impl Parse for Section {
    type Context<'a> = (&'a OptionalHeader, &'a SectionHeader);
    type Error = Error;

    fn parse_with(
        mut buffer: impl Buf,
        (optional, section): Self::Context<'_>,
    ) -> Result<Self, Self::Error> {
        if section.section_size() == 0 {
            return Err(Error::EmptySection);
        }

        let name = *section.name();
        let characteristics = section.characteristics();
        let address = section.section_address() + optional.image_address();
        let alignment = optional.section_alignment() as u64;
        let bytes = buffer.copy_to_bytes(section.file_size());
        let padding = (alignment - (section.section_size() % alignment)) % alignment;
        let mut blocks = Map::new(address.to_space(section.section_size() + padding)?);

        blocks.insert(address, Block::unknown(section.section_size(), bytes))?;

        if padding > 0 {
            let address = address + section.section_size() as u32;

            blocks.insert(address, Block::padding(padding, 0))?;
        }

        Ok(Self {
            name,
            characteristics,
            blocks,
        })
    }
}
