//! A region of unidentified bytes.

mod error;
mod segment;

use std::fmt::{self, Debug};

use bytes::Bytes;
use serde::de::Error as _;
use serde::{Deserialize, Deserializer, Serialize};

use crate::analysis::Completion;
use crate::memory::address::Address;
use crate::memory::extent::{Extent, Size};
use crate::memory::segmented::{Segmented, Segments};
use crate::memory::{Slice, SliceBoundsError};

use super::initialized::Initialized;
use super::uninitialized::Uninitialized;

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

    fn slice(&self, address: Address, size: Size) -> Result<Self, Self::Error> {
        let start = u64::from(address.value());
        let region_size = self.size();

        if start >= region_size.get() || size.get() > region_size.get() - start {
            return Err(Error::SliceBounds(SliceBoundsError {
                address,
                size,
                region_size,
            }));
        }

        let end = start + size.get();

        let segments = self
            .segments()
            .into_iter()
            .select(address, size)
            .map(|entry| {
                let segment_start = u64::from(entry.address().value());
                let segment_end = segment_start + entry.size().get();
                let slice_start = start.max(segment_start);
                let slice_end = end.min(segment_end);
                let slice_address = Address::new((slice_start - segment_start) as u32);
                let slice_size = slice_end - slice_start;

                match entry.segment() {
                    Segment::Initialized(initialized) => initialized
                        .slice(slice_address, Size::new(slice_size).expect("valid size"))
                        .map(Segment::initialized)
                        .map_err(Error::from),
                    Segment::Uninitialized(uninitialized) => uninitialized
                        .slice(slice_address, Size::new(slice_size).expect("valid size"))
                        .map(Segment::uninitialized)
                        .map_err(Error::from),
                }
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

    use crate::memory::Slice;
    use crate::memory::address::Address;
    use crate::memory::extent::{Error as SizeError, Extent, Size};

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
            .slice(Address::new(1), Size::new(2).unwrap())
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
            .slice(Address::new(5), Size::new(3).unwrap())
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
            .slice(Address::new(2), Size::new(6).unwrap())
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
