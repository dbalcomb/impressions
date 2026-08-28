//! The image file section.

pub mod block;

use bytes::{Buf, TryGetError};
use serde::{Deserialize, Serialize};

use crate::analysis::Completion;
use crate::data::parse::Parse;
use crate::data::types::array_string::ArrayString;
use crate::memory::Extent;
use crate::memory::regions::map::{Iter, Map};

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

        if buffer.remaining() < section.file_size() {
            return Err(Error::Parse(TryGetError {
                requested: section.file_size(),
                available: buffer.remaining(),
            }));
        }

        let name = *section.name();
        let characteristics = section.characteristics();
        let address = section.section_address() + optional.image_address();
        let mut blocks = Map::new(address.to_space(section.section_size())?);

        if section.section_size() >= section.file_size() as u64 {
            let bytes = buffer.copy_to_bytes(section.file_size());

            blocks.insert(
                address,
                Block::unidentified(bytes, section.section_size() - section.file_size() as u64)?,
            )?;
        } else {
            let bytes = buffer.copy_to_bytes(section.section_size() as usize);
            let padding = section.file_size() as u64 - section.section_size();

            buffer.advance(padding as usize);
            blocks.insert(address, Block::unidentified(bytes, 0)?)?;
        }

        Ok(Self {
            name,
            characteristics,
            blocks,
        })
    }
}
