//! An imported DLL name.

mod cursor;
mod error;

use serde::{Deserialize, Serialize};

use crate::analysis::Completion;
use crate::memory::cursor::AsCursor;
use crate::memory::extent::{Extent, Size};
use crate::memory::inspect::{Inspect, Inspector};
use crate::memory::region::ops::decode::{Decode, Decoder};
use crate::memory::region::ops::encode::{self, Encode};
use crate::memory::region::types::aligned::Aligned;
use crate::memory::region::types::null_string::NullString;

pub use self::cursor::ImportNameCursor;
pub use self::error::Error;

/// An imported DLL name.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImportName {
    /// The name of the imported DLL.
    name: Aligned<NullString, 2>,
}

impl ImportName {
    /// Gets the name.
    pub const fn name(&self) -> &NullString {
        self.name.region()
    }
}

impl Extent for ImportName {
    fn size(&self) -> Size {
        self.name.size()
    }
}

impl Completion for ImportName {
    fn identified(&self) -> u64 {
        self.size().get()
    }
}

impl Inspect for ImportName {
    fn inspect(&self, inspector: &mut dyn Inspector) {
        inspector
            .record(self.address_space())
            .label(&"Import Name")
            .finish();
    }
}

impl Encode for ImportName {
    fn encode(&self, encoder: &mut dyn encode::Encoder) -> Result<(), encode::Error> {
        self.name.encode(encoder)
    }
}

impl Decode for ImportName {
    type Context<'a> = ();
    type Error = Error;

    fn decode_with(decoder: &mut dyn Decoder, _: Self::Context<'_>) -> Result<Self, Self::Error> {
        Ok(Self {
            name: Aligned::decode(decoder)?,
        })
    }
}

impl AsCursor for ImportName {
    type Cursor<'a> = ImportNameCursor<'a>;

    fn cursor(&self) -> Self::Cursor<'_> {
        ImportNameCursor::new(self)
    }
}

#[cfg(test)]
mod tests {
    use crate::memory::cursor::{AsCursor, Cursor, Position};
    use crate::memory::extent::Extent;
    use crate::memory::region::ops::decode::Decode;

    use super::ImportName;

    #[test]
    fn decode_includes_padding_after_odd_sized_name() {
        let import_name = ImportName::decode(&mut &b"ab\0\0"[..]).unwrap();

        assert_eq!(import_name.size().get(), 4);
    }

    #[test]
    fn cursor_visits_alignment_padding() {
        let import_name = ImportName::decode(&mut &b"ab\0\0"[..]).unwrap();
        let mut cursor = import_name.cursor();

        assert_eq!(cursor.next(), Ok(Some(Position::new(3))));
        assert_eq!(cursor.next(), Ok(None));
        assert_eq!(cursor.position(), Position::new(4));
    }
}
