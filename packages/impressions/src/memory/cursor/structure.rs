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

#[cfg(test)]
mod tests {
    use strum::EnumIter;

    use crate::memory::address::{Address, AddressSpace};
    use crate::memory::cursor::{Cursor, Position, StructCursor};
    use crate::memory::extent::{Extent, Size};

    #[derive(Debug, EnumIter, PartialEq, Eq)]
    enum Field {
        First,
        Second,
        Third,
    }

    impl Extent for Field {
        fn size(&self) -> Size {
            Size::new_valid(match self {
                Self::First => 2,
                Self::Second => 3,
                Self::Third => 1,
            })
        }

        fn address_space(&self) -> AddressSpace {
            let start = match self {
                Self::First => 0,
                Self::Second => 4,
                Self::Third => 7,
            };

            Address::new(start).to_space(self.size()).unwrap()
        }
    }

    struct Structure;

    impl Extent for Structure {
        fn size(&self) -> Size {
            Size::new_valid(8)
        }
    }

    fn cursor() -> StructCursor<'static, Structure, Field> {
        static STRUCTURE: Structure = Structure;

        StructCursor::new(&STRUCTURE)
    }

    #[test]
    fn starts_at_first_field() {
        let cursor = cursor();

        assert_eq!(cursor.position(), Position::START);
        assert_eq!(cursor.field(), Some(Field::First));
    }

    #[test]
    fn seek_identifies_field_from_any_byte_within_it() {
        let mut cursor = cursor();

        cursor.seek(Position::new(1)).unwrap();
        assert_eq!(cursor.field(), Some(Field::First));

        cursor.seek(Position::new(5)).unwrap();
        assert_eq!(cursor.field(), Some(Field::Second));

        cursor.seek(Position::new(7)).unwrap();
        assert_eq!(cursor.field(), Some(Field::Third));
    }

    #[test]
    fn seek_in_gap_has_no_active_field() {
        let mut cursor = cursor();

        cursor.seek(Position::new(2)).unwrap();
        assert_eq!(cursor.field(), None);

        cursor.seek(Position::new(3)).unwrap();
        assert_eq!(cursor.field(), None);
    }

    #[test]
    fn step_moves_to_following_field_boundary() {
        let mut cursor = cursor();

        assert_eq!(cursor.step().unwrap(), Some(Position::new(4)));
        assert_eq!(cursor.field(), Some(Field::Second));

        assert_eq!(cursor.step().unwrap(), Some(Position::new(7)));
        assert_eq!(cursor.field(), Some(Field::Third));
    }

    #[test]
    fn step_from_inside_field_skips_to_next_field() {
        let mut cursor = cursor();

        cursor.seek(Position::new(1)).unwrap();
        assert_eq!(cursor.step().unwrap(), Some(Position::new(4)));

        cursor.seek(Position::new(5)).unwrap();
        assert_eq!(cursor.step().unwrap(), Some(Position::new(7)));
    }

    #[test]
    fn step_from_gap_selects_next_field() {
        let mut cursor = cursor();

        cursor.seek(Position::new(2)).unwrap();
        assert_eq!(cursor.step().unwrap(), Some(Position::new(4)));
        assert_eq!(cursor.field(), Some(Field::Second));
    }

    #[test]
    fn step_at_final_field_moves_to_exclusive_end() {
        let mut cursor = cursor();

        cursor.seek(Position::new(7)).unwrap();

        assert_eq!(cursor.step().unwrap(), None);
        assert_eq!(cursor.position(), Position::new(8));
        assert_eq!(cursor.field(), None);
        assert_eq!(cursor.step().unwrap(), None);
        assert_eq!(cursor.position(), Position::new(8));
    }

    #[test]
    fn seek_accepts_exclusive_end_and_rejects_positions_after_it() {
        let mut cursor = cursor();

        cursor.seek(Position::new(8)).unwrap();
        assert_eq!(cursor.field(), None);

        assert!(cursor.seek(Position::new(9)).is_err());
        assert_eq!(cursor.position(), Position::new(8));
    }

    #[test]
    fn next_matches_step_for_flat_structure() {
        let mut next_cursor = cursor();
        let mut step_cursor = cursor();

        while let Some(next) = next_cursor.next().unwrap() {
            assert_eq!(step_cursor.step().unwrap(), Some(next));
            assert_eq!(next_cursor.position(), step_cursor.position());
        }

        assert_eq!(step_cursor.step().unwrap(), None);
        assert_eq!(next_cursor.position(), step_cursor.position());
    }
}
