use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

use crate::memory::region::ops::encode::Error;

use super::Encoder;

/// The file encoder that writes encoded data to a file.
pub struct FileEncoder {
    writer: BufWriter<File>,
}

impl FileEncoder {
    /// Constructs a new file encoder that writes to the specified file path.
    pub fn new(path: impl AsRef<Path>) -> Result<Self, Error> {
        Ok(Self {
            writer: BufWriter::new(File::create(path)?),
        })
    }

    /// Flushes the encoded bytes to the file.
    pub fn finish(mut self) -> Result<(), Error> {
        self.writer.flush()?;

        Ok(())
    }
}

impl Encoder for FileEncoder {
    fn write(&mut self, bytes: &[u8]) -> Result<(), Error> {
        self.writer.write_all(bytes)?;

        Ok(())
    }
}
