use std::fmt::{self, Debug};
use std::marker::PhantomData;

use strum::IntoEnumIterator;

use crate::memory::address::Address;
use crate::memory::extent::Extent;

use super::{Cursor, Error, Position};

/// A cursor over the fields of a fixed-layout structure.
///
/// # Invariants
///
/// The field `F` must implement `Extent` with the `AddressSpace` of each field
/// being relative to the start of the structure `T`.
///
/// `F::iter()` must enumerate non-empty, non-overlapping fields within `T` in
/// ascending address order. The fields may leave gaps, but cannot extend past
/// `T`.
pub struct StructCursor<'a, T, F> {
    structure: &'a T,
    position: Position,
    field: PhantomData<fn() -> F>,
}

impl<'a, T, F> StructCursor<'a, T, F> {
    /// Constructs a cursor at the first byte of the structure.
    pub const fn new(structure: &'a T) -> Self {
        Self {
            structure,
            position: Position::START,
            field: PhantomData,
        }
    }
}

impl<'a, T, F> StructCursor<'a, T, F> {
    /// Gets the structure being traversed.
    pub const fn structure(&self) -> &'a T {
        self.structure
    }
}

impl<'a, T, F> StructCursor<'a, T, F>
where
    F: Extent + IntoEnumIterator,
{
    /// Gets the field containing the current position.
    ///
    /// Returns `None` when positioned at the structure's exclusive end or in
    /// an intentionally uncovered gap.
    pub fn field(&self) -> Option<F> {
        let address = Address::new(self.position.get_addressable()?);

        F::iter().find(|field| field.address_space().contains(address))
    }

    /// Gets the next field after the given position.
    fn next_field_after(&self, position: Position) -> Option<F> {
        let address = Address::new(position.get_addressable()?);

        F::iter().find(|field| field.address_space().first() > address)
    }
}

impl<T, F> Cursor for StructCursor<'_, T, F>
where
    T: Extent,
    F: Extent + IntoEnumIterator,
{
    type Error = Error;

    fn position(&self) -> Position {
        self.position
    }

    fn seek(&mut self, position: Position) -> Result<(), Self::Error> {
        if position.get() > self.structure.size().get() {
            return Err(Error::OutOfBounds(position, self.structure.size()));
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
        let Some(field) = self.next_field_after(self.position) else {
            self.seek(self.structure.size().into())?;

            return Ok(None);
        };

        let position = Position::from(field.address_space().first());

        self.seek(position)?;

        Ok(Some(position))
    }
}

impl<T, F> Clone for StructCursor<'_, T, F> {
    fn clone(&self) -> Self {
        Self {
            structure: self.structure,
            position: self.position,
            field: PhantomData,
        }
    }
}

impl<T, F> Debug for StructCursor<'_, T, F>
where
    T: Extent,
    F: Extent + IntoEnumIterator + Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("StructCursor")
            .field("position", &self.position())
            .field("field", &self.field())
            .finish()
    }
}
