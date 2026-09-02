//! The image file section.

pub mod block;

use std::fmt::{self, Debug};

use bytes::{Buf, TryGetError};
use serde::{Deserialize, Serialize};

use crate::analysis::Completion;
use crate::data::parse::Parse;
use crate::data::types::array_string::ArrayString;
use crate::memory::Extent;
use crate::memory::regions::contiguous::{Contiguous, Segment};
use crate::memory::regions::unidentified::Unidentified;
use crate::memory::segmented::{Segmented, Segments};

use self::block::Block;

use super::Error;
use super::headers::{SectionCharacteristics, SectionHeader};

/// A 32-bit Portable Executable (PE) image file section.
///
/// Each section is divided up into blocks of memory with the ultimate goal of
/// identifying each and every byte.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Section {
    name: ArrayString<8>,
    characteristics: SectionCharacteristics,
    blocks: Contiguous<Block>,
}

impl Section {
    /// Gets the section name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Gets an iterator over the block segments.
    pub fn blocks(&self) -> impl Iterator<Item = &Block> {
        self.blocks
            .segments()
            .flat_map(|segment| segment.segment().as_identified())
    }
}

impl Extent for Section {
    fn size(&self) -> u64 {
        self.blocks.size()
    }
}

impl Completion for Section {
    fn identified(&self) -> u64 {
        self.blocks.identified()
    }
}

impl Segmented for Section {
    type Segment = Segment<Block>;

    fn segments(&self) -> Segments<'_, Self::Segment> {
        self.blocks.segments()
    }
}

impl Parse for Section {
    type Context<'a> = &'a SectionHeader;
    type Error = Error;

    fn parse_with(mut buffer: impl Buf, section: Self::Context<'_>) -> Result<Self, Self::Error> {
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
        let blocks = if section.section_size() >= section.file_size() as u64 {
            let bytes = buffer.copy_to_bytes(section.file_size());

            Contiguous::unidentified(Unidentified::new(
                bytes,
                section.section_size() - section.file_size() as u64,
            )?)
        } else {
            let bytes = buffer.copy_to_bytes(section.section_size() as usize);
            let padding = section.file_size() as u64 - section.section_size();

            buffer.advance(padding as usize);

            Contiguous::unidentified(Unidentified::new(bytes, 0)?)
        };

        Ok(Self {
            name,
            characteristics,
            blocks,
        })
    }
}

impl Debug for Section {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let completion = std::fmt::from_fn(|f| write!(f, "{:.2}%", self.completion()));

        f.debug_struct("Section")
            .field("name", &self.name)
            .field("completion", &completion)
            .field("characteristics", &self.characteristics)
            .field("blocks", &self.blocks)
            .finish()
    }
}
