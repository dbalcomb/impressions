//! An imported DLL name.

mod error;

use serde::{Deserialize, Serialize};

use crate::analysis::Completion;
use crate::data::parse::Parse;
use crate::data::types::null_string::NullString;
use crate::image::Padding;
use crate::memory::cursor::{AsCursor, SimpleCursor};
use crate::memory::extent::{Extent, Size};
use crate::memory::inspect::{Inspect, Inspector};

pub use self::error::Error;

/// An imported DLL name and its optional alignment padding.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImportName {
    name: NullString,
    pad: Option<Padding>,
}

impl ImportName {
    /// Gets the name.
    pub const fn name(&self) -> &NullString {
        &self.name
    }
}

impl Extent for ImportName {
    fn size(&self) -> Size {
        let mut size = self.name.size();

        if let Some(pad) = &self.pad {
            size = size.checked_add(pad.size()).expect("valid size");
        }

        size
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
            .identified()
            .label(&"Import Name")
            .value(&self.name)
            .finish()
    }
}

impl Parse for ImportName {
    type Context<'a> = ();
    type Error = Error;

    fn parse_with(mut buffer: impl bytes::Buf, _: Self::Context<'_>) -> Result<Self, Self::Error> {
        let name = NullString::parse(&mut buffer)?;
        let pad = if name.size().get().is_multiple_of(2) {
            None
        } else {
            Some(Padding::new(Size::new_valid(1), buffer.try_get_u8()?))
        };

        Ok(Self { name, pad })
    }
}

impl AsCursor for ImportName {
    type Cursor<'a> = SimpleCursor<'a, Self>;

    fn cursor(&self) -> Self::Cursor<'_> {
        SimpleCursor::new(self)
    }
}

#[cfg(test)]
mod tests {
    use crate::data::parse::Parse;
    use crate::memory::extent::Extent;

    use super::ImportName;

    #[test]
    fn parse_includes_padding_after_odd_sized_name() {
        let import_name = ImportName::parse(&b"ab\0\0"[..]).unwrap();

        assert_eq!(import_name.size().get(), 4);
    }
}
