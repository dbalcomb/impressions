use std::fmt::{self, Debug};

use crate::memory::cursor::{Cursor, Error, Position};
use crate::memory::inspect::{self, Inspect};
use crate::memory::segmented::{Segmented, Segments, SegmentsCursor};

use super::{DataDirectories, DataDirectory};

/// A cursor over an Optional header's data directories.
#[derive(Clone)]
pub struct DataDirectoriesCursor<'a>(SegmentsCursor<'a, DataDirectory>);

impl<'a> DataDirectoriesCursor<'a> {
    /// Constructs a new data directories cursor.
    pub(super) fn new(directories: &'a DataDirectories) -> Self {
        Self(SegmentsCursor::new(directories.segments()))
    }
}

impl<'a> DataDirectoriesCursor<'a> {
    /// Gets the data directories that this cursor is over.
    pub const fn directories(&self) -> &Segments<'a, DataDirectory> {
        self.0.segments()
    }
}

impl Cursor for DataDirectoriesCursor<'_> {
    type Error = Error;

    fn position(&self) -> Position {
        self.0.position()
    }

    fn seek(&mut self, position: Position) -> Result<(), Self::Error> {
        self.0.seek(position)
    }

    fn advance(&mut self, offset: u32) -> Result<(), Self::Error> {
        self.0.advance(offset)
    }

    fn next(&mut self) -> Result<Option<Position>, Self::Error> {
        self.0.next()
    }

    fn step(&mut self) -> Result<Option<Position>, Self::Error> {
        self.0.step()
    }
}

impl Inspect for DataDirectoriesCursor<'_> {
    fn inspect(&self, inspector: &mut inspect::Inspector<'_>) -> Result<(), inspect::Error> {
        self.0.inspect(inspector)
    }
}

impl Debug for DataDirectoriesCursor<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DataDirectoriesCursor")
            .field("position", &self.position())
            .field("cursor", self.0.cursor())
            .finish()
    }
}
