use std::fmt::{self, Debug};

use crate::memory::address::Address;
use crate::memory::cursor::{AsCursor, Cursor, Error, Position};
use crate::memory::extent::Extent;
use crate::memory::inspect::{self, Inspect};

use super::{SegmentRef, Segments};

/// A cursor over segments in a segmented region.
pub struct SegmentsCursor<'a, T>
where
    T: AsCursor,
{
    segments: Segments<'a, T>,
    segment: SegmentRef<'a, T>,
    cursor: T::Cursor<'a>,
}

impl<'a, T> SegmentsCursor<'a, T>
where
    T: Extent + AsCursor,
{
    /// Constructs a new segmented cursor.
    pub(crate) fn new(segments: Segments<'a, T>) -> Self {
        let segment = segments
            .iter()
            .next()
            .expect("segments must contain at least one segment");

        Self {
            cursor: segment.segment().cursor(),
            segment,
            segments,
        }
    }
}

impl<'a, T> SegmentsCursor<'a, T>
where
    T: AsCursor,
{
    /// Gets the segments that this cursor is over.
    pub const fn segments(&self) -> &Segments<'a, T> {
        &self.segments
    }

    /// Gets the segment at the cursor position.
    pub const fn segment(&self) -> &'a T {
        self.segment.segment()
    }

    /// Gets the inner cursor.
    pub const fn cursor(&self) -> &T::Cursor<'a> {
        &self.cursor
    }
}

impl<'a, T> SegmentsCursor<'a, T>
where
    T: Extent + AsCursor,
{
    fn global_position(&self, position: Position) -> Position {
        match position.get_addressable() {
            Some(offset) => self
                .segment
                .address()
                .checked_add(offset)
                .map(Position::from)
                .expect("nested cursor position is within its segment"),
            None => Position::END,
        }
    }

    fn is_at_end(&self) -> bool {
        self.position() == Position::from(self.segments.size())
    }
}

impl<'a, T> Cursor for SegmentsCursor<'a, T>
where
    T: Extent + AsCursor,
{
    type Error = <T::Cursor<'a> as Cursor>::Error;

    fn position(&self) -> Position {
        self.global_position(self.cursor.position())
    }

    fn seek(&mut self, position: Position) -> Result<(), Self::Error> {
        let size = self.segments.size();

        if position.get() > size.get() {
            return Err(Error::OutOfBounds(position, size).into());
        }

        let end = Position::from(size);

        if position == end {
            let segment = self
                .segments
                .iter()
                .last()
                .expect("segmented regions contain at least one segment");

            let mut cursor = segment.segment().cursor();

            cursor.seek(Position::from(segment.size()))?;

            self.segment = segment;
            self.cursor = cursor;

            return Ok(());
        }

        let address = position
            .get_addressable()
            .expect("a position below the region end is addressable");

        let segment = self
            .segments
            .get(Address::new(address))
            .expect("an in-bounds address is contained in a segment");

        let offset = segment
            .address_space()
            .get_offset_at(Address::new(address))
            .expect("selected segment contains the address");

        if segment.index() == self.segment.index() {
            self.cursor.seek(Position::new(offset))?;
        } else {
            let mut cursor = segment.segment().cursor();

            cursor.seek(Position::new(offset))?;

            self.segment = segment;
            self.cursor = cursor;
        }

        Ok(())
    }

    fn advance(&mut self, offset: u32) -> Result<(), Self::Error> {
        if offset == 0 {
            return Ok(());
        }

        let position = self.position();

        let Some(position) = position.checked_add(offset) else {
            return Err(Error::CannotAdvance(position, offset).into());
        };

        self.seek(position)
    }

    fn next(&mut self) -> Result<Option<Position>, Self::Error> {
        if self.is_at_end() {
            return Ok(None);
        }

        let position = match self.segment.address_space().next() {
            Some(address) => Position::from(address),
            None => Position::END,
        };

        self.seek(position)?;

        if self.is_at_end() {
            Ok(None)
        } else {
            Ok(Some(position))
        }
    }

    fn step(&mut self) -> Result<Option<Position>, Self::Error> {
        if self.is_at_end() {
            return Ok(None);
        }

        match self.cursor.step()? {
            Some(position) => {
                let position = self.global_position(position);

                self.seek(position)?;

                if self.is_at_end() {
                    Ok(None)
                } else {
                    Ok(Some(position))
                }
            }
            None => self.next(),
        }
    }
}

impl<'a, T> Clone for SegmentsCursor<'a, T>
where
    T: AsCursor<Cursor<'a>: Clone>,
{
    fn clone(&self) -> Self {
        Self {
            segments: self.segments.clone(),
            segment: self.segment.clone(),
            cursor: self.cursor.clone(),
        }
    }
}

impl<'a, T> Inspect for SegmentsCursor<'a, T>
where
    T: AsCursor<Cursor<'a>: Inspect> + Extent + Inspect,
{
    fn inspect(&self, inspector: &mut inspect::Inspector<'_>) -> Result<(), inspect::Error> {
        if self.cursor().position() == Position::START {
            self.segment.inspect(inspector)?;
        }

        inspector.nest();
        self.cursor().inspect(inspector)?;

        Ok(())
    }
}

impl<'a, T> Debug for SegmentsCursor<'a, T>
where
    T: Extent + AsCursor<Cursor<'a>: Debug>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SegmentsCursor")
            .field("position", &self.position())
            .field("cursor", &self.cursor)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use crate::memory::cursor::{Cursor, Error, Position};
    use crate::memory::extent::Size;
    use crate::memory::regions::uninitialized::Uninitialized;

    use super::{Segments, SegmentsCursor};

    fn size(value: u64) -> Size {
        Size::new(value).unwrap()
    }

    fn cursor<'a>(segments: &'a [Uninitialized]) -> SegmentsCursor<'a, Uninitialized> {
        SegmentsCursor::new(Segments::new(segments))
    }

    #[test]
    fn starts_at_beginning_of_first_segment() {
        let segments = [Uninitialized::new(size(2)), Uninitialized::new(size(3))];
        let cursor = cursor(&segments);

        assert_eq!(cursor.position(), Position::START);
    }

    #[test]
    fn seek_rebases_nested_cursor_to_selected_segment() {
        let segments = [
            Uninitialized::new(size(2)),
            Uninitialized::new(size(3)),
            Uninitialized::new(size(5)),
        ];
        let mut cursor = cursor(&segments);

        cursor.seek(Position::new(3)).unwrap();

        assert_eq!(cursor.position(), Position::new(3));
        assert_eq!(cursor.cursor().position(), Position::new(1));
    }

    #[test]
    fn seek_rejects_position_past_region_end() {
        let segments = [Uninitialized::new(size(2)), Uninitialized::new(size(3))];
        let mut cursor = cursor(&segments);

        assert_eq!(
            cursor.seek(Position::new(6)),
            Err(Error::OutOfBounds(Position::new(6), size(5),)),
        );

        assert_eq!(cursor.position(), Position::START);
    }

    #[test]
    fn seek_to_end_positions_nested_cursor_at_end_of_final_segment() {
        let segments = [Uninitialized::new(size(2)), Uninitialized::new(size(3))];
        let mut cursor = cursor(&segments);

        cursor.seek(Position::from(size(5))).unwrap();

        assert_eq!(cursor.position(), Position::from(size(5)));
        assert_eq!(cursor.cursor().position(), Position::from(size(3)));
        assert_eq!(cursor.step().unwrap(), None);
    }

    #[test]
    fn advance_crosses_segment_boundaries() {
        let segments = [
            Uninitialized::new(size(2)),
            Uninitialized::new(size(3)),
            Uninitialized::new(size(5)),
        ];
        let mut cursor = cursor(&segments);

        cursor.advance(4).unwrap();

        assert_eq!(cursor.position(), Position::new(4));
        assert_eq!(cursor.cursor().position(), Position::new(2));
        assert_eq!(cursor.next().unwrap(), Some(Position::new(5)));
    }

    #[test]
    fn next_visits_each_following_top_level_segment_start() {
        let segments = [
            Uninitialized::new(size(2)),
            Uninitialized::new(size(3)),
            Uninitialized::new(size(5)),
        ];
        let mut cursor = cursor(&segments);

        assert_eq!(cursor.next().unwrap(), Some(Position::new(2)));
        assert_eq!(cursor.position(), Position::new(2));

        assert_eq!(cursor.next().unwrap(), Some(Position::new(5)));
        assert_eq!(cursor.position(), Position::new(5));

        assert_eq!(cursor.next().unwrap(), None);
        assert_eq!(cursor.position(), Position::from(size(10)));
    }

    #[test]
    fn next_is_idempotent_at_end() {
        let segments = [Uninitialized::new(size(2))];
        let mut cursor = cursor(&segments);

        assert_eq!(cursor.next().unwrap(), None);
        assert_eq!(cursor.position(), Position::from(size(2)));

        assert_eq!(cursor.next().unwrap(), None);
        assert_eq!(cursor.position(), Position::from(size(2)));
    }

    #[test]
    fn step_advances_to_next_top_level_segment_when_leaf_has_no_steps() {
        let segments = [
            Uninitialized::new(size(2)),
            Uninitialized::new(size(3)),
            Uninitialized::new(size(5)),
        ];
        let mut cursor = cursor(&segments);

        assert_eq!(cursor.step().unwrap(), Some(Position::new(2)));
        assert_eq!(cursor.step().unwrap(), Some(Position::new(5)));
        assert_eq!(cursor.step().unwrap(), None);
        assert_eq!(cursor.position(), Position::from(size(10)));
    }

    #[test]
    fn step_supports_do_while_style_traversal_from_initial_position() {
        let segments = [
            Uninitialized::new(size(2)),
            Uninitialized::new(size(3)),
            Uninitialized::new(size(5)),
        ];
        let mut cursor = cursor(&segments);
        let mut positions = Vec::new();

        loop {
            positions.push(cursor.position());

            if cursor.step().unwrap().is_none() {
                break;
            }
        }

        assert_eq!(
            positions,
            [Position::START, Position::new(2), Position::new(5)],
        );
        assert_eq!(cursor.position(), Position::from(size(10)));
    }
}
