use std::fs;
use std::path::{Path, PathBuf};

use bytes::Bytes;

use crate::data::parse::Parse;
use crate::image::Image;

use super::{Analysis, Error};

/// The binary analysis builder.
pub struct Builder {
    path: PathBuf,
}

impl Builder {
    /// Constructs a new binary analysis builder.
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    /// Builds the binary analysis with the given binary data.
    pub fn with_binary_data(self, bytes: impl Into<Bytes>) -> Result<Analysis, Error> {
        Ok(Analysis {
            path: self.path,
            image: Image::parse(bytes.into())?,
        })
    }

    /// Builds the binary analysis with the given binary path.
    pub fn with_binary_path(self, path: impl AsRef<Path>) -> Result<Analysis, Error> {
        self.with_binary_data(fs::read(path)?)
    }
}
