use std::fmt::{self, Debug};

use crate::memory::cursor::{Cursor, Error, Position};
use crate::memory::inspect::{Inspect, Inspector};
use crate::memory::region::types::segmented::{Segmented, SegmentsCursor};

use super::{DataDirectories, DataDirectory};

/// A cursor over an Optional header's data directories.
#[derive(Clone)]
pub struct DataDirectoriesCursor<'a> {
    directories: &'a DataDirectories,
    cursor: SegmentsCursor<'a, DataDirectory>,
}

impl<'a> DataDirectoriesCursor<'a> {
    /// Constructs a new data directories cursor.
    pub(super) fn new(directories: &'a DataDirectories) -> Self {
        Self {
            directories,
            cursor: SegmentsCursor::new(directories.segments()),
        }
    }
}

impl<'a> DataDirectoriesCursor<'a> {
    /// Gets the data directories that this cursor is over.
    pub const fn directories(&self) -> &'a DataDirectories {
        self.directories
    }
}

impl Cursor for DataDirectoriesCursor<'_> {
    type Error = Error;

    fn position(&self) -> Position {
        self.cursor.position()
    }

    fn seek(&mut self, position: Position) -> Result<(), Self::Error> {
        self.cursor.seek(position)
    }

    fn advance(&mut self, offset: u32) -> Result<(), Self::Error> {
        self.cursor.advance(offset)
    }

    fn next(&mut self) -> Result<Option<Position>, Self::Error> {
        self.cursor.next()
    }

    fn step(&mut self) -> Result<Option<Position>, Self::Error> {
        self.cursor.step()
    }
}

impl Inspect for DataDirectoriesCursor<'_> {
    fn inspect(&self, inspector: &mut dyn Inspector) {
        self.directories().inspect(inspector);
        self.cursor.inspect(inspector);
    }
}

impl Debug for DataDirectoriesCursor<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DataDirectoriesCursor")
            .field("position", &self.position())
            .field("cursor", self.cursor.cursor())
            .finish()
    }
}
