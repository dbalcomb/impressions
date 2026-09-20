use std::fmt::{self, Debug};

use crate::memory::address::Address;
use crate::memory::cursor::{Cursor, Error, Position};
use crate::memory::extent::{Extent, Size};
use crate::memory::inspect::{Inspect, Inspector};

use super::ImportName;

/// A cursor over an imported DLL name and its alignment padding.
pub struct ImportNameCursor<'a> {
    name: &'a ImportName,
    position: Position,
}

impl<'a> ImportNameCursor<'a> {
    pub(super) const fn new(name: &'a ImportName) -> Self {
        Self {
            name,
            position: Position::START,
        }
    }

    fn name_size(&self) -> u64 {
        self.name.name().size().get()
    }
}

impl Cursor for ImportNameCursor<'_> {
    type Error = Error;

    fn position(&self) -> Position {
        self.position
    }

    fn seek(&mut self, position: Position) -> Result<(), Self::Error> {
        if position.get() > self.name.size().get() {
            return Err(Error::OutOfBounds(position, self.name.size()));
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
        let end = Position::from(self.name.size());

        if self.position == end {
            return Ok(None);
        }

        if self.position.get() < self.name_size() && self.name_size() < self.name.size().get() {
            let position = Position::new(self.name_size() as u32);

            self.seek(position)?;

            return Ok(Some(position));
        }

        self.seek(end)?;

        Ok(None)
    }

    fn step(&mut self) -> Result<Option<Position>, Self::Error> {
        self.next()
    }
}

impl Inspect for ImportNameCursor<'_> {
    fn inspect(&self, inspector: &mut dyn Inspector) {
        if self.position == Position::from(self.name.size()) {
            return;
        }

        if self.position.get() < self.name_size() {
            inspector
                .record(Size::new_valid(self.name_size()).to_address_space())
                .identified()
                .label(&"Name")
                .value(self.name.name())
                .finish();

            return;
        }

        inspector
            .record(
                Address::new(self.name_size() as u32)
                    .to_space(Size::new_valid(self.name.size().get() - self.name_size()))
                    .expect("padding is within the import name"),
            )
            .vacant()
            .label(&"Padding")
            .finish();
    }
}

impl Clone for ImportNameCursor<'_> {
    fn clone(&self) -> Self {
        Self {
            name: self.name,
            position: self.position,
        }
    }
}

impl Debug for ImportNameCursor<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ImportNameCursor")
            .field("position", &self.position)
            .finish()
    }
}
