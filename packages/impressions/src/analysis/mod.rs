//! The binary analysis engine.

mod error;
pub mod image;

use std::path::Path;

pub use self::error::Error;
use self::image::Image;

/// A 32-bit Portable Executable (PE) binary analysis.
#[derive(Debug)]
pub struct Analysis {
    image: Image,
}

impl Analysis {
    /// Constructs a new binary analysis from the given image file path.
    pub fn new(path: impl AsRef<Path>) -> Result<Self, Error> {
        Ok(Self {
            image: Image::open(path)?,
        })
    }
}

impl Analysis {
    /// Gets the image file analysis.
    pub fn image(&self) -> &Image {
        &self.image
    }
}
