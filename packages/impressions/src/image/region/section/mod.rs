//! The image file section.

pub mod block;

mod cursor;
mod error;

use std::cmp::Ordering;
use std::fmt::{self, Debug};

use bytes::{Buf, TryGetError};
use serde::{Deserialize, Serialize};

use crate::analysis::Completion;
use crate::data::parse::Parse;
use crate::data::types::array_string::ArrayString;
use crate::memory::cursor::AsCursor;
use crate::memory::extent::{Error as SizeError, Extent, Size};
use crate::memory::inspect::{self, Inspect};
use crate::memory::regions::contiguous::{Contiguous, Segment};
use crate::memory::regions::unidentified::Unidentified;
use crate::memory::segmented::{Segmented, Segments};

use self::block::Block;

pub use self::cursor::SectionCursor;
pub use self::error::Error;

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
            .into_iter()
            .flat_map(|segment| segment.segment().as_identified())
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

impl Parse for Section {
    type Context<'a> = &'a SectionHeader;
    type Error = Error;

    fn parse_with(mut buffer: impl Buf, section: Self::Context<'_>) -> Result<Self, Self::Error> {
        if section.section_size() == 0 {
            return Err(Error::Size(SizeError::Zero));
        }

        if buffer.remaining() < section.file_size() {
            return Err(Error::Parse(TryGetError {
                requested: section.file_size(),
                available: buffer.remaining(),
            }));
        }

        let name = *section.name();
        let characteristics = section.characteristics();
        let blocks = match section.section_size().cmp(&(section.file_size() as u64)) {
            Ordering::Less => {
                let bytes = buffer.copy_to_bytes(section.section_size() as usize);
                let padding = section.file_size() as u64 - section.section_size();

                buffer.advance(padding as usize);

                Contiguous::unidentified(Unidentified::try_from_initialized_bytes(bytes)?)
            }
            Ordering::Equal => {
                let bytes = buffer.copy_to_bytes(section.file_size());

                Contiguous::unidentified(Unidentified::try_from_initialized_bytes(bytes)?)
            }
            Ordering::Greater => {
                let bytes = buffer.copy_to_bytes(section.file_size());

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
    fn inspect(&self, inspector: &mut inspect::Inspector<'_>) -> Result<(), inspect::Error> {
        let name = self.name();
        let c = self.characteristics;
        let r = if c.read() { "r" } else { "-" };
        let w = if c.write() { "w" } else { "-" };
        let x = if c.execute() { "x" } else { "-" };

        writeln!(inspector.identified(), "Section {name:<8} [{r}{w}{x}]")
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
    #[rustfmt::skip]
    type Cursor<'a> = SectionCursor<'a>
    where
        Self: 'a;

    fn cursor(&self) -> Self::Cursor<'_> {
        SectionCursor::new(self)
    }
}
