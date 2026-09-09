mod error;
mod position;
mod simple;

pub use self::error::Error;
pub use self::position::Position;
pub use self::simple::SimpleCursor;

/// A cursor over a memory region.
pub trait Cursor {
    /// The associated error type for the cursor.
    type Error: From<Error>;

    /// Gets the current position of the cursor.
    fn position(&self) -> Position;

    /// Seeks the cursor to the specified position.
    fn seek(&mut self, position: Position) -> Result<(), Self::Error>;

    /// Advances the cursor by the specified offset.
    fn advance(&mut self, offset: u32) -> Result<(), Self::Error>;

    /// Advances the cursor to the next top-level segment.
    fn next(&mut self) -> Result<Option<Position>, Self::Error>;

    /// Advances the cursor to the next bottom-level segment.
    fn step(&mut self) -> Result<Option<Position>, Self::Error>;

    /// Builds the cursor at the given position.
    fn at(mut self, position: Position) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        self.seek(position)?;

        Ok(self)
    }
}

/// Provides the ability to create a cursor over a memory region.
pub trait AsCursor {
    /// The associated cursor type for this region.
    type Cursor<'a>: Cursor
    where
        Self: 'a;

    fn cursor(&self) -> Self::Cursor<'_>;
}
