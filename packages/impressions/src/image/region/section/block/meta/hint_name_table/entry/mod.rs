//! The hint/name table entry.

mod cursor;
mod error;

use serde::{Deserialize, Serialize};

use crate::analysis::Completion;
use crate::data::parse::Parse;
use crate::data::types::null_string::NullString;
use crate::image::Padding;
use crate::memory::cursor::AsCursor;
use crate::memory::extent::{Extent, Size};
use crate::memory::inspect::{Inspect, Inspector};

pub use self::cursor::HintNameCursor;
pub use self::error::Error;

/// A hint/name table entry.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct HintName {
    /// The hint.
    ///
    /// An index into the export name pointer table. A match is attempted first
    /// with this value. If it fails, a binary search is performed on the DLL's
    /// export name pointer table.
    hint: u16,

    /// The name.
    ///
    /// An ASCII string that contains the name to import. This is the string
    /// that must be matched to the public name in the DLL. This string is case
    /// sensitive and terminated by a null byte.
    name: NullString,

    /// The padding.
    ///
    /// A trailing zero-pad byte that appears after the trailing null byte, if
    /// necessary, to align the next entry on an even boundary.
    pad: Option<Padding>,
}

impl HintName {
    /// Gets the hint.
    pub const fn hint(&self) -> u16 {
        self.hint
    }

    /// Gets the name.
    pub const fn name(&self) -> &NullString {
        &self.name
    }
}

impl Extent for HintName {
    fn size(&self) -> Size {
        let mut size = Size::new_valid(2)
            .checked_add(self.name.size())
            .expect("valid size");

        if let Some(pad) = &self.pad {
            size = size.checked_add(pad.size()).expect("valid size");
        }

        size
    }
}

impl Completion for HintName {
    fn identified(&self) -> u64 {
        self.size().get()
    }
}

impl Inspect for HintName {
    fn inspect(&self, inspector: &mut dyn Inspector) {
        inspector
            .record(self.address_space())
            .label(&"Hint/Name")
            .finish()
    }
}

impl Parse for HintName {
    type Context<'a> = ();
    type Error = Error;

    fn parse_with(mut buffer: impl bytes::Buf, _: Self::Context<'_>) -> Result<Self, Self::Error> {
        let hint = buffer.get_u16_le();
        let name = NullString::parse(&mut buffer)?;

        let pad = if name.size().get().is_multiple_of(2) {
            None
        } else {
            Some(Padding::new(Size::new_valid(1), buffer.try_get_u8()?))
        };

        Ok(Self { hint, name, pad })
    }
}

impl AsCursor for HintName {
    type Cursor<'a> = HintNameCursor<'a>;

    fn cursor(&self) -> Self::Cursor<'_> {
        HintNameCursor::new(self)
    }
}
