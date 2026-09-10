mod coff;
mod cursor;
mod dos;
mod optional;
mod section;

use std::fmt::{self, Debug};

use serde::{Deserialize, Serialize};

use crate::analysis::Completion;
use crate::image::Padding;
use crate::memory::cursor::AsCursor;
use crate::memory::extent::{Extent, Size};
use crate::memory::inspect::{self, Inspect};

pub use self::coff::CoffHeader;
pub use self::cursor::HeaderCursor;
pub use self::dos::DosHeader;
pub use self::optional::{DataDirectory, DataDirectoryTable, OptionalHeader};
pub use self::section::{SectionCharacteristics, SectionHeader};

/// A header in a PE image file.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Header {
    Dos(DosHeader),
    Signature,
    Coff(CoffHeader),
    Optional(OptionalHeader),
    Section(SectionHeader),
    Padding(Padding),
}

impl Header {
    /// Gets the header as a DOS header.
    pub const fn as_dos(&self) -> Option<&DosHeader> {
        match self {
            Self::Dos(dos) => Some(dos),
            _ => None,
        }
    }

    /// Gets the header as a COFF header.
    pub const fn as_coff(&self) -> Option<&CoffHeader> {
        match self {
            Self::Coff(coff) => Some(coff),
            _ => None,
        }
    }

    /// Gets the header as an Optional header.
    pub const fn as_optional(&self) -> Option<&OptionalHeader> {
        match self {
            Self::Optional(optional) => Some(optional),
            _ => None,
        }
    }

    /// Gets the header as a Section header.
    pub const fn as_section(&self) -> Option<&SectionHeader> {
        match self {
            Self::Section(section) => Some(section),
            _ => None,
        }
    }
}

impl Header {
    /// Checks whether the header is a DOS header.
    pub const fn is_dos(&self) -> bool {
        matches!(self, Self::Dos(_))
    }

    /// Checks whether the header is a COFF header.
    pub const fn is_coff(&self) -> bool {
        matches!(self, Self::Coff(_))
    }

    /// Checks whether the header is the PE signature.
    pub const fn is_signature(&self) -> bool {
        matches!(self, Self::Signature)
    }

    /// Checks whether the header is an Optional header.
    pub const fn is_optional(&self) -> bool {
        matches!(self, Self::Optional(_))
    }

    /// Checks whether the header is a Section header.
    pub const fn is_section(&self) -> bool {
        matches!(self, Self::Section(_))
    }

    /// Checks whether the header is padding.
    pub const fn is_padding(&self) -> bool {
        matches!(self, Self::Padding(_))
    }
}

impl Extent for Header {
    fn size(&self) -> Size {
        match self {
            Self::Dos(dos) => dos.size(),
            Self::Signature => Size::new(4).expect("valid size"),
            Self::Coff(coff) => coff.size(),
            Self::Optional(optional) => optional.size(),
            Self::Section(section) => section.size(),
            Self::Padding(padding) => padding.size(),
        }
    }
}

impl Completion for Header {
    fn identified(&self) -> u64 {
        self.size().get()
    }
}

impl Inspect for Header {
    fn inspect(&self, inspector: &mut inspect::Inspector<'_>) -> Result<(), inspect::Error> {
        match self {
            Self::Dos(dos) => dos.inspect(inspector),
            Self::Signature => writeln!(inspector.identified(), "Signature"),
            Self::Coff(coff) => coff.inspect(inspector),
            Self::Optional(optional) => optional.inspect(inspector),
            Self::Section(section) => section.inspect(inspector),
            Self::Padding(padding) => padding.inspect(inspector),
        }
    }
}

impl Debug for Header {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Dos(dos) => Debug::fmt(dos, f),
            Self::Signature => f.write_str("Signature"),
            Self::Coff(coff) => Debug::fmt(coff, f),
            Self::Optional(optional) => Debug::fmt(optional, f),
            Self::Section(section) => Debug::fmt(section, f),
            Self::Padding(padding) => Debug::fmt(padding, f),
        }
    }
}

impl AsCursor for Header {
    #[rustfmt::skip]
    type Cursor<'a> = HeaderCursor<'a>
    where
        Self: 'a;

    fn cursor(&self) -> Self::Cursor<'_> {
        HeaderCursor::new(self)
    }
}
