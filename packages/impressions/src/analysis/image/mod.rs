//! The image file analysis.

mod error;
pub mod headers;

use std::path::Path;

use bytes::{Buf, Bytes};

use crate::data::Parse;

pub use self::error::Error;
use self::headers::Headers;

/// A 32-bit Portable Executable (PE) image file analysis.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Image {
    headers: Headers,
}

impl Image {
    /// Constructs a new image file analysis from the given image file path.
    pub fn open(path: impl AsRef<Path>) -> Result<Self, Error> {
        Self::parse(Bytes::from(std::fs::read(path)?))
    }
}

impl Image {
    /// Gets the image address.
    pub fn address(&self) -> u32 {
        self.headers.address()
    }

    /// Gets the image size.
    pub fn size(&self) -> u64 {
        self.headers.optional().image_size()
    }
}

impl Parse for Image {
    type Error = Error;

    fn parse(buffer: impl Buf) -> Result<Self, Self::Error> {
        Ok(Self {
            headers: Headers::parse(buffer)?,
        })
    }
}

impl TryFrom<Bytes> for Image {
    type Error = Error;

    fn try_from(bytes: Bytes) -> Result<Self, Self::Error> {
        Self::parse(bytes)
    }
}
