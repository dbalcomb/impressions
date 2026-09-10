//! A region of unidentified bytes.

mod cursor;
mod error;
mod segment;

use std::fmt::{self, Debug};

use bytes::Bytes;
use serde::de::Error as _;
use serde::{Deserialize, Deserializer, Serialize};

use crate::analysis::Completion;
use crate::memory::address::{Address, AddressSpace};
use crate::memory::cursor::AsCursor;
use crate::memory::extent::{Extent, Size};
use crate::memory::inspect::{self, Inspect};
use crate::memory::segmented::{Segmented, Segments, SegmentsCursor};
use crate::memory::slice::{Error as SliceError, Slice};

use super::initialized::Initialized;
use super::uninitialized::Uninitialized;

pub use self::cursor::SegmentCursor;
pub use self::error::Error;
pub use self::segment::Segment;

/// A region of unidentified bytes.
///
/// This region is composed of zero or more initialized segments followed by at
/// most one uninitialized segment. The uninitialized segment, if present, must
/// be the last segment in the region.
#[derive(Clone, PartialEq, Eq, Serialize)]
#[serde(transparent)]
#[repr(transparent)]
pub struct Unidentified(Vec<Segment>);

impl Unidentified {
    /// Constructs a new unidentified region from an initialized region.
    pub fn from_initialized(initialized: Initialized) -> Self {
        Self(vec![Segment::Initialized(initialized)])
    }

    /// Constructs a new unidentified region from an uninitialized region.
    pub fn from_uninitialized(uninitialized: Uninitialized) -> Self {
        Self(vec![Segment::Uninitialized(uninitialized)])
    }
}

impl Unidentified {
    /// Constructs a new unidentified region from the given initialized bytes.
    pub fn try_from_initialized_bytes(bytes: Bytes) -> Result<Self, Error> {
        Ok(Self::from_initialized(Initialized::new(bytes)?))
    }

    /// Constructs a new unidentified region from the given uninitialized size.
    pub fn try_from_uninitialized_size(size: u64) -> Result<Self, Error> {
        Ok(Self::from_uninitialized(Uninitialized::new(
            size.try_into()?,
        )))
    }

    /// Constructs a new unidentified region from an iterator of segments.
    pub fn try_from_iterator(segments: impl IntoIterator<Item = Segment>) -> Result<Self, Error> {
        Self::try_from(segments.into_iter().collect::<Vec<Segment>>())
    }
}

impl Unidentified {
    /// Builds the unidentified region with the given uninitialized region.
    pub fn with_uninitialized(mut self, uninitialized: Uninitialized) -> Result<Self, Error> {
        if self.uninitialized().is_some() {
            return Err(Error::UninitializedAlreadyPresent);
        }

        self.size().checked_add(uninitialized.size())?;
        self.0.push(Segment::Uninitialized(uninitialized));

        Ok(self)
    }

    /// Builds the unidentified region with the given uninitialized size.
    pub fn with_uninitialized_size(self, size: u64) -> Result<Self, Error> {
        self.with_uninitialized(Uninitialized::new(size.try_into()?))
    }
}

impl Unidentified {
    /// Gets the initialized regions.
    pub fn initialized(&self) -> impl Iterator<Item = &Initialized> {
        self.segments()
            .into_iter()
            .filter_map(|segment| segment.segment().as_initialized())
    }

    /// Gets the uninitialized region.
    pub fn uninitialized(&self) -> Option<&Uninitialized> {
        self.segments()
            .into_iter()
            .find_map(|segment| segment.segment().as_uninitialized())
    }
}

impl Slice for Unidentified {
    type Error = Error;

    fn slice(&self, address_space: AddressSpace) -> Result<Self, Self::Error> {
        let region_address_space = self.address_space();

        if !region_address_space.includes(address_space) {
            return Err(Error::Slice(SliceError::OutOfBounds(
                address_space,
                region_address_space,
            )));
        }

        let segments = self
            .segments()
            .into_iter()
            .select(address_space)
            .map(|segment| {
                let segment_address_space = segment.address_space();

                let slice_address_space = segment_address_space
                    .intersection(address_space)
                    .expect("selected segments intersect the requested address space");

                let local_start = Address::new(
                    segment_address_space
                        .get_offset_at(slice_address_space.first())
                        .expect("intersection begins within its segment"),
                );

                let local_address_space = slice_address_space
                    .rebase(local_start)
                    .expect("a subspace of a valid segment fits in its local address space");

                segment.segment().slice(local_address_space)
            })
            .collect::<Result<Vec<_>, Error>>()?;

        Self::try_from(segments)
    }
}

impl Extent for Unidentified {
    fn size(&self) -> Size {
        Size::try_sum(self.0.iter().map(Extent::size))
            .expect("sum of sizes does not exceed maximum size")
    }
}

impl Completion for Unidentified {
    fn identified(&self) -> u64 {
        0
    }
}

impl Segmented for Unidentified {
    type Segment = Segment;

    fn segments(&self) -> Segments<'_, Self::Segment> {
        Segments::new(&self.0)
    }
}

impl Inspect for Unidentified {
    fn inspect(&self, inspector: &mut inspect::Inspector<'_>) -> Result<(), inspect::Error> {
        writeln!(inspector.unidentified(), "Unidentified")
    }
}

impl Debug for Unidentified {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Unidentified")
            .field("size", &self.size())
            .field("segments", &self.0)
            .finish()
    }
}

impl<'de> Deserialize<'de> for Unidentified {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Self::try_from(<Vec<Segment>>::deserialize(deserializer)?).map_err(D::Error::custom)
    }
}

impl AsCursor for Unidentified {
    #[rustfmt::skip]
    type Cursor<'a> = SegmentsCursor<'a, Segment>
    where
        Self: 'a;

    fn cursor(&self) -> Self::Cursor<'_> {
        SegmentsCursor::new(self.segments())
    }
}

impl From<Initialized> for Unidentified {
    fn from(initialized: Initialized) -> Self {
        Self::from_initialized(initialized)
    }
}

impl From<Uninitialized> for Unidentified {
    fn from(uninitialized: Uninitialized) -> Self {
        Self::from_uninitialized(uninitialized)
    }
}

impl TryFrom<Vec<Segment>> for Unidentified {
    type Error = Error;

    fn try_from(segments: Vec<Segment>) -> Result<Self, Self::Error> {
        Size::try_sum(segments.iter().map(Extent::size))?;

        if segments
            .iter()
            .enumerate()
            .any(|(index, segment)| segment.is_uninitialized() && index + 1 != segments.len())
        {
            return Err(Error::UninitializedNotLast);
        }

        Ok(Self(segments))
    }
}

#[cfg(test)]
mod tests {
    use bytes::Bytes;

    use crate::memory::address::Address;
    use crate::memory::extent::{Error as SizeError, Extent, Size};
    use crate::memory::slice::Slice;

    use super::{Error, Initialized, Segment, Unidentified, Uninitialized};

    #[test]
    fn construction_rejects_no_segments() {
        assert_eq!(
            Unidentified::try_from(Vec::new()),
            Err(Error::Size(SizeError::Zero))
        );
    }

    #[test]
    fn construction_rejects_uninitialized_segment_before_initialized_segment() {
        let segments = [
            Segment::uninitialized(Uninitialized::new(Size::new(1).unwrap())),
            Segment::initialized(Initialized::new(Bytes::from_static(b"a")).unwrap()),
        ];

        assert_eq!(
            Unidentified::try_from_iterator(segments),
            Err(Error::UninitializedNotLast),
        );
    }

    #[test]
    fn with_uninitialized_rejects_existing_uninitialized_segment() {
        let region = Unidentified::try_from_uninitialized_size(1).unwrap();

        assert_eq!(
            region.with_uninitialized_size(1),
            Err(Error::UninitializedAlreadyPresent),
        );
    }

    #[test]
    fn slice_within_initialized_memory() {
        let region = Unidentified::try_from_initialized_bytes(Bytes::from_static(b"abcd"))
            .unwrap()
            .with_uninitialized_size(6)
            .unwrap();

        let slice = region
            .slice(Address::new(1).to_space(Size::new(2).unwrap()).unwrap())
            .unwrap();

        assert_eq!(slice.initialized().next().unwrap().bytes(), "bc");
        assert_eq!(slice.uninitialized(), None);
        assert_eq!(slice.size(), 2);
    }

    #[test]
    fn slice_within_uninitialized_memory() {
        let region = Unidentified::try_from_initialized_bytes(Bytes::from_static(b"abcd"))
            .unwrap()
            .with_uninitialized_size(6)
            .unwrap();

        let slice = region
            .slice(Address::new(5).to_space(Size::new(3).unwrap()).unwrap())
            .unwrap();

        assert_eq!(slice.initialized().next(), None);
        assert_eq!(slice.uninitialized().unwrap().size(), 3);
        assert_eq!(slice.size(), 3);
    }

    #[test]
    fn slice_crossing_initialized_and_uninitialized_memory() {
        let region = Unidentified::try_from_initialized_bytes(Bytes::from_static(b"abcd"))
            .unwrap()
            .with_uninitialized_size(6)
            .unwrap();

        let slice = region
            .slice(Address::new(2).to_space(Size::new(6).unwrap()).unwrap())
            .unwrap();

        assert_eq!(slice.initialized().next().unwrap().bytes(), "cd");
        assert_eq!(slice.uninitialized().unwrap().size(), 4);
        assert_eq!(slice.size(), 6);
    }

    #[test]
    fn serde_multiple_initialized_segments_with_trailing_uninitialized_segment() {
        let region = Unidentified::try_from_iterator([
            Segment::initialized(Initialized::new(Bytes::from_static(b"ab")).unwrap()),
            Segment::initialized(Initialized::new(Bytes::from_static(b"cd")).unwrap()),
            Segment::uninitialized(Uninitialized::new(Size::new(2).unwrap())),
        ])
        .unwrap();

        let serialized = serde_json::to_string(&region).unwrap();

        assert_eq!(
            serde_json::from_str::<Unidentified>(&serialized).unwrap(),
            region
        );
    }
}
