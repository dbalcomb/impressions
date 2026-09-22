use super::Encoder;
use crate::memory::region::ops::encode::Error;

/// An encoder that discards bytes after reaching a configured limit.
pub struct TruncatingEncoder<'a> {
    encoder: &'a mut dyn Encoder,
    limit: usize,
    position: usize,
}

impl<'a> TruncatingEncoder<'a> {
    /// Constructs an encoder that retains at most `limit` bytes.
    pub fn new(encoder: &'a mut dyn Encoder, limit: usize) -> Self {
        Self {
            encoder,
            limit,
            position: 0,
        }
    }

    /// Gets the logical position, including discarded bytes.
    pub const fn position(&self) -> usize {
        self.position
    }
}

impl Encoder for TruncatingEncoder<'_> {
    fn write(&mut self, bytes: &[u8]) -> Result<(), Error> {
        let available = self.limit.saturating_sub(self.position);
        let written = bytes.len().min(available);

        if written > 0 {
            self.encoder.write(&bytes[..written])?;
        }

        self.position = self.position.saturating_add(bytes.len());

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{Encoder, Error, TruncatingEncoder};

    #[derive(Default)]
    struct VecEncoder(Vec<u8>);

    impl Encoder for VecEncoder {
        fn write(&mut self, bytes: &[u8]) -> Result<(), Error> {
            self.0.extend_from_slice(bytes);

            Ok(())
        }
    }

    #[test]
    fn retains_the_prefix_and_tracks_discarded_bytes() {
        let mut inner = VecEncoder::default();
        let mut encoder = TruncatingEncoder::new(&mut inner, 3);

        encoder.write(&[1, 2]).unwrap();
        encoder.write(&[3, 4]).unwrap();
        encoder.write(&[5]).unwrap();

        assert_eq!(encoder.position(), 5);
        assert_eq!(inner.0, [1, 2, 3]);
    }
}
