//! The binary analysis engine.

mod builder;
mod completion;
mod error;

use std::fs::File;
use std::io::{BufReader, BufWriter, Write};
use std::path::PathBuf;

use rmp_serde::{Deserializer, Serializer};
use serde::{Deserialize, Serialize};

use crate::image::Image;

pub use self::builder::Builder;
pub use self::completion::Completion;
pub use self::error::Error;

/// A 32-bit Portable Executable (PE) binary analysis.
#[derive(Debug)]
pub struct Analysis {
    path: PathBuf,
    image: Image,
}

impl Analysis {
    /// Builds a new binary analysis at the given path.
    pub fn build(path: impl Into<PathBuf>) -> Builder {
        Builder::new(path)
    }

    /// Opens a binary analysis at the given path.
    pub fn open(path: impl Into<PathBuf>) -> Result<Self, Error> {
        let path = path.into();
        let mut reader = BufReader::new(File::open(&path)?);
        let mut deserializer = Deserializer::new(&mut reader);
        let image = Image::deserialize(&mut deserializer)?;

        Ok(Self { path, image })
    }

    /// Saves the binary analysis.
    pub fn save(&self) -> Result<(), Error> {
        let mut writer = BufWriter::new(File::create(&self.path)?);
        let mut serializer = Serializer::new(&mut writer);

        self.image.serialize(&mut serializer)?;
        writer.flush()?;

        Ok(())
    }
}

impl Analysis {
    /// Gets the image file analysis.
    pub fn image(&self) -> &Image {
        &self.image
    }

    /// Gets the completion percentage of the analysis.
    pub fn completion(&self) -> f64 {
        self.image.completion()
    }
}
