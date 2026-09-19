use std::fmt::{self, Debug};

use crate::memory::address::{Address, AddressSpace};
use crate::memory::cursor::{Cursor, Error, Position};
use crate::memory::extent::Extent;
use crate::memory::inspect::{Inspect, Inspector};

use super::HintName;

/// A cursor over the hint/name table entry.
#[derive(Clone)]
pub struct HintNameCursor<'a> {
    entry: &'a HintName,
    position: Position,
}

impl<'a> HintNameCursor<'a> {
    /// Constructs a cursor at the first byte of the entry.
    pub const fn new(entry: &'a HintName) -> Self {
        Self {
            entry,
            position: Position::START,
        }
    }
}

impl<'a> HintNameCursor<'a> {
    /// Gets the entry being traversed.
    pub const fn entry(&self) -> &'a HintName {
        self.entry
    }
}

impl Cursor for HintNameCursor<'_> {
    type Error = Error;

    fn position(&self) -> Position {
        self.position
    }

    fn seek(&mut self, position: Position) -> Result<(), Self::Error> {
        if position.get() > self.entry.size().get() {
            return Err(Error::OutOfBounds(position, self.entry.size()));
        }

        self.position = position;

        Ok(())
    }

    fn advance(&mut self, offset: u32) -> Result<(), Self::Error> {
        let position = self
            .position
            .checked_add(offset)
            .ok_or(Error::CannotAdvance(self.position, offset))?;

        self.seek(position)
    }

    fn next(&mut self) -> Result<Option<Position>, Self::Error> {
        self.step()
    }

    fn step(&mut self) -> Result<Option<Position>, Self::Error> {
        if self.position.get() < 2 {
            let position = Position::new(2);

            self.seek(position)?;

            return Ok(Some(position));
        }

        let name = self.entry().name();
        let size = name.size().get();

        if self.position.get() < 2 + size {
            let position = Position::new((2 + size) as u32);

            self.seek(position)?;

            if size.is_multiple_of(2) {
                return Ok(None);
            } else {
                return Ok(Some(position));
            }
        }

        if !size.is_multiple_of(2) {
            self.seek(Position::new((2 + size + 1) as u32))?;
        }

        Ok(None)
    }
}

impl Inspect for HintNameCursor<'_> {
    fn inspect(&self, inspector: &mut dyn Inspector) {
        if self.position.get() < 2 {
            inspector
                .record(
                    AddressSpace::new(Address::new(0), Address::new(1))
                        .expect("valid address space"),
                )
                .identified()
                .label(&"Hint")
                .value(&self.entry().hint())
                .finish();

            return;
        }

        let name = self.entry().name();
        let size = name.size().get();

        if self.position.get() < 2 + size {
            inspector
                .record(
                    AddressSpace::new(Address::new(2), Address::new((2 + size - 1) as u32))
                        .expect("valid address space"),
                )
                .identified()
                .label(&"Name")
                .value(self.entry().name())
                .finish();

            return;
        }

        if !size.is_multiple_of(2) && self.position.get() < 2 + size + 1 {
            inspector
                .record(
                    AddressSpace::new(
                        Address::new((2 + size) as u32),
                        Address::new((2 + size) as u32),
                    )
                    .expect("valid address space"),
                )
                .identified()
                .label(&"Pad")
                .value(self.entry().pad.as_ref().expect("pad exists"))
                .finish();
        }
    }
}

impl Debug for HintNameCursor<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("HintNameCursor")
            .field("position", &self.position())
            .finish()
    }
}
