//! The x86 instruction prefix.

mod error;

use crate::memory::region::ops::encode::{Encode, Encoder, Error as EncodeError};

pub use self::error::Error;

/// An x86 instruction prefix.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum Prefix {
    /// FS segment override.
    OverrideSegmentFS = 0x64,
}

impl TryFrom<u8> for Prefix {
    type Error = Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x64 => Ok(Self::OverrideSegmentFS),
            value => Err(Error::Invalid(value)),
        }
    }
}

/// A collection of instruction prefixes.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Prefixes([Option<Prefix>; 4]);

impl Prefixes {
    /// Constructs a new collection of instruction prefixes.
    pub const fn new() -> Self {
        Self([None; 4])
    }

    /// Pushes a prefix onto the collection.
    pub fn push(&mut self, prefix: Prefix) -> Result<(), Error> {
        for slot in self.0.iter_mut() {
            if slot.is_none() {
                *slot = Some(prefix);

                return Ok(());
            }
        }

        Err(Error::TooManyPrefixes)
    }

    /// Returns the number of prefixes in the collection.
    pub fn len(&self) -> usize {
        self.0.iter().filter(|slot| slot.is_some()).count()
    }

    /// Returns whether the collection of prefixes is empty.
    pub fn is_empty(&self) -> bool {
        self.0.iter().all(|slot| slot.is_none())
    }
}

impl Prefixes {
    /// Returns whether the instruction has an FS segment override.
    pub fn has_fs_override(&self) -> bool {
        self.0.contains(&Some(Prefix::OverrideSegmentFS))
    }
}

impl Encode for Prefixes {
    fn encode(&self, encoder: &mut dyn Encoder) -> Result<(), EncodeError> {
        for prefix in self.0.iter().flatten() {
            encoder.write_u8(*prefix as u8)?;
        }

        Ok(())
    }
}
