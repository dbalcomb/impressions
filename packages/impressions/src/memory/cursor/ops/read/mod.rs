//! Memory region cursor reading.

mod error;

use crate::data::parse::Parse;
use crate::memory::cursor::Cursor;
use crate::memory::extent::Extent;

pub use self::error::Error;

/// Read regions of memory from a cursor.
pub trait Read: Cursor {
    /// Reads a region of type `T` from the current position of the cursor with
    /// the given context.
    ///
    /// Implementors of this method are expected to advance the cursor by the
    /// extent of the parsed region.
    fn read_with<'a, T>(
        &mut self,
        context: T::Context<'a>,
    ) -> Result<T, Error<T::Error, Self::Error>>
    where
        T: Extent + Parse;

    /// Reads a region of type `T` from the current position of the cursor.
    ///
    /// Implementors of this method are expected to advance the cursor by the
    /// extent of the parsed region.
    fn read<'a, T>(&mut self) -> Result<T, Error<T::Error, Self::Error>>
    where
        T: Extent + Parse<Context<'a> = ()>,
    {
        self.read_with(())
    }
}
