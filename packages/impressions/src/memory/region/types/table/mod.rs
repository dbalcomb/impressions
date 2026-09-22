//! The null-terminated table region type.

mod cursor;
mod error;

use std::slice::Iter;
use std::vec::IntoIter;

use bytes::Buf;
use serde::{Deserialize, Serialize};

use crate::data::parse::Parse;
use crate::memory::address::Address;
use crate::memory::cursor::AsCursor;
use crate::memory::extent::{Extent, FixedExtent, Size};
use crate::memory::inspect::{Inspect, Inspector};
use crate::memory::region::Null;
use crate::memory::region::ops::encode::{self, Encode};

pub use self::cursor::TableCursor;
pub use self::error::Error;

/// A null-terminated table of fixed-size entries.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
#[repr(transparent)]
pub struct Table<T>(Vec<T>);

impl<T> Table<T> {
    /// Gets a row in the table at the given index.
    pub fn get(&self, index: usize) -> Option<&T> {
        self.0.get(index)
    }

    /// Gets an iterator over the rows in the table.
    pub fn iter(&self) -> Iter<'_, T> {
        self.0.iter()
    }

    /// Gets the number of rows in the table.
    pub const fn len(&self) -> usize {
        self.0.len()
    }

    /// Checks whether the table is empty.
    pub const fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl<T> Table<T>
where
    T: FixedExtent,
{
    /// Gets the maximum number of rows that can fit in the given table size.
    fn max_rows(size: Option<Size>) -> u64 {
        size.unwrap_or(Size::MAX).get() / T::SIZE.get()
    }
}

impl<T> Extent for Table<T>
where
    T: FixedExtent,
{
    fn size(&self) -> Size {
        Size::new_valid(T::SIZE.get() * (self.0.len() as u64 + 1))
    }
}

impl<T> Inspect for Table<T>
where
    T: FixedExtent,
{
    fn inspect(&self, inspector: &mut dyn Inspector) {
        inspector
            .record(self.address_space())
            .label(&"Table")
            .finish()
    }
}

impl<T> Encode for Table<T>
where
    T: FixedExtent + Null + Encode,
{
    fn encode(&self, encoder: &mut dyn encode::Encoder) -> Result<(), encode::Error> {
        for row in &self.0 {
            row.encode(encoder)?;
        }

        T::null().encode(encoder)?;

        Ok(())
    }
}

impl<T> Parse for Table<T>
where
    T: FixedExtent + Null + for<'a> Parse<Context<'a> = ()>,
{
    type Context<'a> = Option<Size>;
    type Error = Error<T::Error>;

    fn parse_with(mut buffer: impl Buf, size: Self::Context<'_>) -> Result<Self, Self::Error> {
        if let Some(size) = size
            && !size.get().is_multiple_of(T::SIZE.get())
        {
            return Err(Error::SizeNotMultiple {
                size,
                row_size: T::SIZE,
            });
        }

        let mut table = Vec::new();

        for i in 0..Self::max_rows(size) {
            let region = T::parse(&mut buffer).map_err(|error| Error::Row {
                address: Address::new(i as u32),
                error,
            })?;

            if region.is_null() {
                if let Some(expected) = size
                    && let actual = Size::new_valid(T::SIZE.get() * (i + 1))
                    && expected != actual
                {
                    return Err(Error::SizeMismatch { expected, actual });
                }

                return Ok(Self(table));
            }

            table.push(region);
        }

        Err(Error::MissingNullTerminator)
    }
}

impl<T> AsCursor for Table<T>
where
    T: FixedExtent + AsCursor,
{
    #[rustfmt::skip]
    type Cursor<'a> = TableCursor<'a, T>
    where
        Self: 'a;

    fn cursor(&self) -> Self::Cursor<'_> {
        TableCursor::new(self)
    }
}

impl<T> IntoIterator for Table<T> {
    type Item = T;
    type IntoIter = IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a, T> IntoIterator for &'a Table<T> {
    type Item = &'a T;
    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

#[cfg(test)]
mod tests {
    use bytes::{Buf, TryGetError};
    use serde::{Deserialize, Serialize};

    use crate::data::parse::Parse;
    use crate::memory::cursor::{AsCursor, Cursor, Position, SimpleCursor};
    use crate::memory::extent::{Extent, FixedExtent, Size};
    use crate::memory::inspect::{Inspect, Inspector};
    use crate::memory::region::Null;

    use super::{Error, Table};

    #[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
    struct ByteRow(u8);

    impl FixedExtent for ByteRow {
        const SIZE: Size = Size::MIN;
    }

    impl Inspect for ByteRow {
        fn inspect(&self, inspector: &mut dyn Inspector) {
            inspector
                .record(self.address_space())
                .label(&"Byte Row")
                .finish()
        }
    }

    impl AsCursor for ByteRow {
        type Cursor<'a> = SimpleCursor<'a, Self>;

        fn cursor(&self) -> Self::Cursor<'_> {
            SimpleCursor::new(self)
        }
    }

    impl Null for ByteRow {
        fn null() -> Self {
            Self(0)
        }

        fn is_null(&self) -> bool {
            self.0 == 0
        }
    }

    impl Parse for ByteRow {
        type Context<'a> = ();
        type Error = TryGetError;

        fn parse_with(mut buffer: impl Buf, _: Self::Context<'_>) -> Result<Self, Self::Error> {
            buffer.try_get_u8().map(Self)
        }
    }

    #[derive(Clone, Debug, PartialEq, Eq)]
    struct DoubleByteRow([u8; 2]);

    impl FixedExtent for DoubleByteRow {
        const SIZE: Size = Size::new_valid(2);
    }

    impl Null for DoubleByteRow {
        fn null() -> Self {
            Self([0, 0])
        }

        fn is_null(&self) -> bool {
            self.0 == [0, 0]
        }
    }

    impl Parse for DoubleByteRow {
        type Context<'a> = ();
        type Error = TryGetError;

        fn parse_with(mut buffer: impl Buf, _: Self::Context<'_>) -> Result<Self, Self::Error> {
            Ok(Self([buffer.try_get_u8()?, buffer.try_get_u8()?]))
        }
    }

    #[test]
    fn parses_table_with_exact_size() {
        assert_eq!(
            Table::<ByteRow>::parse_with([1, 2, 0].as_slice(), Some(Size::new_valid(3))),
            Ok(Table(vec![ByteRow(1), ByteRow(2)])),
        );
    }

    #[test]
    fn rejects_terminator_before_expected_size() {
        assert_eq!(
            Table::<ByteRow>::parse_with([1, 0].as_slice(), Some(Size::new_valid(3))),
            Err(Error::SizeMismatch {
                expected: Size::new_valid(3),
                actual: Size::new_valid(2),
            }),
        );
    }

    #[test]
    fn rejects_size_that_cannot_contain_whole_rows() {
        assert_eq!(
            Table::<DoubleByteRow>::parse_with([].as_slice(), Some(Size::new_valid(3))),
            Err(Error::SizeNotMultiple {
                size: Size::new_valid(3),
                row_size: DoubleByteRow::SIZE,
            }),
        );
    }

    #[test]
    fn rejects_table_without_terminator_within_expected_size() {
        assert_eq!(
            Table::<ByteRow>::parse_with([1].as_slice(), Some(Size::MIN)),
            Err(Error::MissingNullTerminator),
        );
    }

    #[test]
    fn limits_unbounded_tables_to_maximum_representable_size() {
        assert_eq!(Table::<ByteRow>::max_rows(None), Size::MAX.get());
        assert_eq!(
            Table::<DoubleByteRow>::max_rows(None),
            Size::MAX.get() / DoubleByteRow::SIZE.get(),
        );
    }

    #[test]
    fn serializes_transparently_as_rows() {
        let table = Table(vec![ByteRow(1), ByteRow(2)]);

        assert_eq!(serde_json::to_string(&table).unwrap(), "[1,2]");
        assert_eq!(
            serde_json::from_str::<Table<ByteRow>>("[1,2]").unwrap(),
            table,
        );
    }

    #[test]
    fn cursor_reuses_segments_cursor_for_rows() {
        let table = Table(vec![ByteRow(1), ByteRow(2)]);
        let mut cursor = table.cursor();

        assert!(cursor.cursor().is_some());

        assert_eq!(cursor.next(), Ok(Some(Position::new(1))));
        assert!(cursor.cursor().is_some());

        assert_eq!(cursor.next(), Ok(Some(Position::new(2))));
        assert!(cursor.cursor().is_none());

        cursor.seek(Position::new(1)).unwrap();

        assert!(cursor.cursor().is_some());

        cursor.seek(Position::new(2)).unwrap();

        assert!(cursor.cursor().is_none());

        assert_eq!(cursor.next(), Ok(None));
        assert_eq!(cursor.position(), Position::new(3));
    }
}
