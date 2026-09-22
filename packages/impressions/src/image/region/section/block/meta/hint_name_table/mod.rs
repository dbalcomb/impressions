//! The hint/name table.

pub mod entry;

mod cursor;

use serde::{Deserialize, Serialize};

use crate::analysis::Completion;
use crate::memory::cursor::AsCursor;
use crate::memory::extent::{Extent, Size};
use crate::memory::inspect::{Inspect, Inspector};
use crate::memory::region::ops::encode::{self, Encode};
use crate::memory::region::types::segmented::{Segmented, Segments};

pub use self::cursor::HintNameTableCursor;

use self::entry::HintName;

/// The hint/name table.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
#[repr(transparent)]
pub struct HintNameTable(Vec<HintName>);

impl HintNameTable {
    /// Constructs a hint/name table from its entries.
    pub const fn new(entries: Vec<HintName>) -> Self {
        assert!(
            !entries.is_empty(),
            "hint/name table must have at least one entry"
        );

        Self(entries)
    }
}

impl HintNameTable {
    /// Gets an iterator over the hint/name entries.
    pub fn iter(&self) -> impl Iterator<Item = &HintName> {
        self.0.iter()
    }
}

impl Extent for HintNameTable {
    fn size(&self) -> Size {
        Size::try_sum(self.0.iter().map(Extent::size)).expect("valid size")
    }
}

impl Completion for HintNameTable {
    fn identified(&self) -> u64 {
        self.size().get()
    }
}

impl Segmented for HintNameTable {
    type Segment = HintName;

    fn segments(&self) -> Segments<'_, Self::Segment> {
        Segments::new(&self.0)
    }
}

impl Inspect for HintNameTable {
    fn inspect(&self, inspector: &mut dyn Inspector) {
        inspector
            .record(self.address_space())
            .label(&"Hint/Name Table")
            .finish()
    }
}

impl Encode for HintNameTable {
    fn encode(&self, encoder: &mut dyn encode::Encoder) -> Result<(), encode::Error> {
        for entry in &self.0 {
            entry.encode(encoder)?;
        }

        Ok(())
    }
}

impl AsCursor for HintNameTable {
    type Cursor<'a> = HintNameTableCursor<'a>;

    fn cursor(&self) -> Self::Cursor<'_> {
        HintNameTableCursor::new(self)
    }
}
