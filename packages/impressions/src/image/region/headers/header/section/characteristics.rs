use std::fmt::{self, Debug, Display};

use bytes::Buf;
use serde::{Deserialize, Serialize};

use crate::data::parse::Parse;
use crate::image::region::headers::Error;

/// The image file section characteristics.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
#[repr(transparent)]
pub struct SectionCharacteristics(u32);

impl SectionCharacteristics {
    /// The section contains executable code.
    const CNT_CODE: u32 = 0x00000020;

    /// The section contains initialized data.
    const CNT_INITIALIZED_DATA: u32 = 0x00000040;

    /// The section contains uninitialized data.
    const CNT_UNINITIALIZED_DATA: u32 = 0x00000080;

    /// The section can be discarded as needed.
    const MEM_DISCARDABLE: u32 = 0x02000000;

    /// The section cannot be cached.
    const MEM_NOT_CACHED: u32 = 0x04000000;

    /// The section is not pageable.
    const MEM_NOT_PAGED: u32 = 0x08000000;

    /// The section can be shared in memory.
    const MEM_SHARED: u32 = 0x10000000;

    /// The section can be executed as code.
    const MEM_EXECUTE: u32 = 0x20000000;

    /// The section can be read.
    const MEM_READ: u32 = 0x40000000;

    /// The section can be written to.
    const MEM_WRITE: u32 = 0x80000000;
}

impl SectionCharacteristics {
    /// Checks if the section is readable.
    pub const fn read(self) -> bool {
        self.0 & Self::MEM_READ != 0
    }

    /// Checks if the section is writable.
    pub const fn write(self) -> bool {
        self.0 & Self::MEM_WRITE != 0
    }

    /// Checks if the section is executable.
    pub const fn execute(self) -> bool {
        self.0 & Self::MEM_EXECUTE != 0
    }

    /// Checks if the section is shareable.
    pub const fn share(self) -> bool {
        self.0 & Self::MEM_SHARED != 0
    }

    /// Checks if the section is pageable.
    pub const fn page(self) -> bool {
        self.0 & Self::MEM_NOT_PAGED == 0
    }

    /// Checks if the section is cacheable.
    pub const fn cache(self) -> bool {
        self.0 & Self::MEM_NOT_CACHED == 0
    }

    /// Checks if the section is discardable.
    pub const fn discard(self) -> bool {
        self.0 & Self::MEM_DISCARDABLE != 0
    }

    /// Checks if the section contains initialised data.
    pub const fn initialised(self) -> bool {
        self.0 & Self::CNT_INITIALIZED_DATA != 0
    }

    /// Checks if the section contains uninitialised data.
    pub const fn uninitialised(self) -> bool {
        self.0 & Self::CNT_UNINITIALIZED_DATA != 0
    }

    /// Checks if the section contains code.
    pub const fn code(self) -> bool {
        self.0 & Self::CNT_CODE != 0
    }
}

impl Parse for SectionCharacteristics {
    type Context<'a> = ();
    type Error = Error;

    fn parse_with(mut buffer: impl Buf, _: Self::Context<'_>) -> Result<Self, Self::Error> {
        Ok(Self(buffer.try_get_u32_le()?))
    }
}

impl Display for SectionCharacteristics {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let r = if self.read() { "r" } else { "-" };
        let w = if self.write() { "w" } else { "-" };
        let x = if self.execute() { "x" } else { "-" };

        write!(f, "{r}{w}{x}")
    }
}

impl Debug for SectionCharacteristics {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SectionCharacteristics")
            .field("read", &self.read())
            .field("write", &self.write())
            .field("execute", &self.execute())
            .field("share", &self.share())
            .field("page", &self.page())
            .field("cache", &self.cache())
            .field("discard", &self.discard())
            .field("initialised", &self.initialised())
            .field("uninitialised", &self.uninitialised())
            .field("code", &self.code())
            .finish()
    }
}
