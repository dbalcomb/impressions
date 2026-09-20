//! The hint/name table entry.

mod cursor;
mod error;

use serde::{Deserialize, Serialize};

use crate::analysis::Completion;
use crate::data::parse::Parse;
use crate::data::types::null_string::NullString;
use crate::memory::cursor::AsCursor;
use crate::memory::extent::{Extent, Size};
use crate::memory::inspect::{Inspect, Inspector};
use crate::memory::region::types::aligned::Aligned;

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
    name: Aligned<NullString, 2>,
}

impl HintName {
    /// Gets the hint.
    pub const fn hint(&self) -> u16 {
        self.hint
    }

    /// Gets the name.
    pub const fn name(&self) -> &NullString {
        self.name.region()
    }
}

impl Extent for HintName {
    fn size(&self) -> Size {
        Size::new_valid(2)
            .checked_add(self.name.size())
            .expect("valid size")
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
        Ok(Self {
            hint: buffer.try_get_u16_le()?,
            name: Aligned::parse(&mut buffer)?,
        })
    }
}

impl AsCursor for HintName {
    type Cursor<'a> = HintNameCursor<'a>;

    fn cursor(&self) -> Self::Cursor<'_> {
        HintNameCursor::new(self)
    }
}

#[cfg(test)]
mod tests {
    use crate::data::parse::Parse;
    use crate::memory::extent::Extent;

    use super::HintName;

    #[test]
    fn parse_includes_padding_after_odd_sized_name() {
        let hint_name = HintName::parse(&b"\0\0ab\0\0"[..]).unwrap();

        assert_eq!(hint_name.name(), "ab");
        assert_eq!(hint_name.size().get(), 6);
    }
}
