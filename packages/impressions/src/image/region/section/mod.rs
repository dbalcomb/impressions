//! The image file section.

pub mod block;

mod cursor;
mod error;

use std::cmp::Ordering;
use std::fmt::{self, Debug};

use serde::{Deserialize, Serialize};

use crate::analysis::Completion;
use crate::data::types::array_string::ArrayString;
use crate::memory::address::Address;
use crate::memory::cursor::AsCursor;
use crate::memory::extent::{Error as SizeError, Extent, Size};
use crate::memory::inspect::{Inspect, Inspector};
use crate::memory::region::ops::decode::{Decode, Decoder, Error as DecodeError};
use crate::memory::region::ops::encode::{self, Encode};
use crate::memory::region::ops::insert::Insert;
use crate::memory::region::types::contiguous::{Contiguous, Segment};
use crate::memory::region::types::segmented::{Segmented, Segments};
use crate::memory::region::types::unidentified::Unidentified;

use self::block::Block;

pub use self::cursor::SectionCursor;
pub use self::error::Error;

use super::headers::header::section::{SectionCharacteristics, SectionHeader};

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
            .into_iter()
            .flat_map(|segment| segment.segment().as_identified())
    }
}

impl Insert<Block> for Section {
    type Error = Error;

    fn insert(&mut self, address: Address, region: Block) -> Result<(), Self::Error> {
        self.blocks.insert(address, region)?;

        Ok(())
    }
}

impl Extent for Section {
    fn size(&self) -> Size {
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

impl Encode for Section {
    fn encode(&self, encoder: &mut dyn encode::Encoder) -> Result<(), encode::Error> {
        self.blocks.encode(encoder)
    }
}

impl Decode for Section {
    type Context<'a> = &'a SectionHeader;
    type Error = Error;

    fn decode_with(
        decoder: &mut dyn Decoder,
        section: Self::Context<'_>,
    ) -> Result<Self, Self::Error> {
        if section.section_size() == 0 {
            return Err(Error::Size(SizeError::Zero));
        }

        if decoder.remaining() < section.file_size() {
            return Err(Error::Decode(DecodeError::InsufficientBytes {
                requested: section.file_size(),
                available: decoder.remaining(),
            }));
        }

        let name = *section.name();
        let characteristics = section.characteristics();
        let blocks = match section.section_size().cmp(&(section.file_size() as u64)) {
            Ordering::Less => {
                let bytes = decoder.read_bytes(section.section_size() as usize)?;
                let padding = section.file_size() as u64 - section.section_size();

                decoder.skip(padding as usize)?;

                Contiguous::unidentified(Unidentified::try_from_initialized_bytes(bytes)?)
            }
            Ordering::Equal => {
                let bytes = decoder.read_bytes(section.file_size())?;

                Contiguous::unidentified(Unidentified::try_from_initialized_bytes(bytes)?)
            }
            Ordering::Greater => {
                let bytes = decoder.read_bytes(section.file_size())?;

                Contiguous::unidentified(
                    Unidentified::try_from_initialized_bytes(bytes)?.with_uninitialized_size(
                        section.section_size() - section.file_size() as u64,
                    )?,
                )
            }
        };

        Ok(Self {
            name,
            characteristics,
            blocks,
        })
    }
}

impl Inspect for Section {
    fn inspect(&self, inspector: &mut dyn Inspector) {
        inspector
            .record(self.address_space())
            .label(&format_args!(
                "Section {} {}",
                self.characteristics, self.name
            ))
            .finish()
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

impl AsCursor for Section {
    type Cursor<'a> = SectionCursor<'a>;

    fn cursor(&self) -> Self::Cursor<'_> {
        SectionCursor::new(self)
    }
}
