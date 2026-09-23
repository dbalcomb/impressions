//! The memory region decoder.

use bytes::{Buf, Bytes, BytesMut};

use super::Error;

/// Defines the behavior of a memory region decoder.
pub trait Decoder {
    /// Returns the number of remaining bytes in the decoder.
    fn remaining(&self) -> usize;

    /// Returns a slice of some of the remaining bytes in the decoder.
    fn chunk(&self) -> &[u8];

    /// Skips a number of bytes in the decoder.
    fn skip(&mut self, n: usize) -> Result<(), Error>;

    /// Reads a byte slice from the decoder.
    fn read(&mut self, mut bytes: &mut [u8]) -> Result<(), Error> {
        if self.remaining() < bytes.len() {
            return Err(Error::InsufficientBytes {
                requested: bytes.len(),
                available: self.remaining(),
            });
        }

        while !bytes.is_empty() {
            let chunk = self.chunk();
            let count = usize::min(chunk.len(), bytes.len());

            bytes[..count].copy_from_slice(&chunk[..count]);
            bytes = &mut bytes[count..];

            self.skip(count)?;
        }

        Ok(())
    }

    /// Reads a `u8` value.
    fn read_u8(&mut self) -> Result<u8, Error> {
        let mut buf = [0; 1];

        self.read(&mut buf)?;

        Ok(buf[0])
    }

    /// Reads a `u16` value in little-endian format.
    fn read_u16_le(&mut self) -> Result<u16, Error> {
        let mut buf = [0; 2];

        self.read(&mut buf)?;

        Ok(u16::from_le_bytes(buf))
    }

    /// Reads a `u32` value in little-endian format.
    fn read_u32_le(&mut self) -> Result<u32, Error> {
        let mut buf = [0; 4];

        self.read(&mut buf)?;

        Ok(u32::from_le_bytes(buf))
    }

    /// Reads a `u64` value in little-endian format.
    fn read_u64_le(&mut self) -> Result<u64, Error> {
        let mut buf = [0; 8];

        self.read(&mut buf)?;

        Ok(u64::from_le_bytes(buf))
    }

    /// Reads a `Bytes` value.
    fn read_bytes(&mut self, len: usize) -> Result<Bytes, Error> {
        if self.remaining() < len {
            return Err(Error::InsufficientBytes {
                requested: len,
                available: self.remaining(),
            });
        }

        let mut bytes = BytesMut::zeroed(len);

        self.read(&mut bytes)?;

        Ok(bytes.freeze())
    }

    /// Checks whether there are remaining bytes in the decoder.
    fn has_remaining(&self) -> bool {
        self.remaining() > 0
    }
}

impl Decoder for &[u8] {
    fn remaining(&self) -> usize {
        self.len()
    }

    fn chunk(&self) -> &[u8] {
        self
    }

    fn skip(&mut self, n: usize) -> Result<(), Error> {
        if n > self.len() {
            return Err(Error::InsufficientBytes {
                requested: n,
                available: self.len(),
            });
        }

        *self = &self[n..];

        Ok(())
    }

    fn read(&mut self, bytes: &mut [u8]) -> Result<(), Error> {
        if bytes.len() > self.len() {
            return Err(Error::InsufficientBytes {
                requested: bytes.len(),
                available: self.len(),
            });
        }

        let (head, tail) = self.split_at(bytes.len());

        bytes.copy_from_slice(head);

        *self = tail;

        Ok(())
    }
}

impl Decoder for Bytes {
    fn remaining(&self) -> usize {
        self.len()
    }

    fn chunk(&self) -> &[u8] {
        self.as_ref()
    }

    fn skip(&mut self, n: usize) -> Result<(), Error> {
        if n > self.len() {
            return Err(Error::InsufficientBytes {
                requested: n,
                available: self.len(),
            });
        }

        self.advance(n);

        Ok(())
    }

    fn read_bytes(&mut self, len: usize) -> Result<Bytes, Error> {
        if len > self.len() {
            return Err(Error::InsufficientBytes {
                requested: len,
                available: self.len(),
            });
        }

        Ok(self.split_to(len))
    }
}
