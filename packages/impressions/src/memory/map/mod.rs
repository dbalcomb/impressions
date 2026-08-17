//! Mapping address spaces to memory regions.

mod error;
mod iter;
mod offset;

use std::collections::btree_map::BTreeMap;
use std::fmt::{self, Debug};

pub use self::error::Error;
pub use self::iter::{IntoIter, Iter};
pub use self::offset::Offset;

use super::address::{Address, AddressSpace};
use super::region::Region;

/// A map of address spaces to memory regions.
///
/// # Invariants
///
/// The address space of each region in the map must be **immutable**. Any
/// modifications to the address space may corrupt the map and potentially cause
/// items to overlap or escape the bounds of the map.
#[derive(Clone, PartialEq, Eq)]
pub struct Map<T> {
    address_space: AddressSpace,
    inner: BTreeMap<Address, T>,
}

impl<T> Map<T> {
    /// Constructs a new memory map.
    pub const fn new(address_space: AddressSpace) -> Self {
        Self {
            address_space,
            inner: BTreeMap::new(),
        }
    }
}

impl<T> Map<T>
where
    T: Region,
{
    /// Gets a region for the given address.
    ///
    /// This method returns an optional [`Offset`] which wraps the region with
    /// an offset as the address may be inside the region's address space.
    pub fn get(&self, address: Address) -> Option<Offset<'_, T>> {
        match self.inner.range(..=address).last() {
            Some((_, region)) if region.address_space().contains(address) => {
                Some(Offset::new(region, address.offset(region.address())))
            }
            _ => None,
        }
    }

    /// Gets a region for the given offset.
    ///
    /// This method is similar to [`Self::get`] except that it takes a relative
    /// offset instead of an address.
    pub fn get_relative(&self, offset: u32) -> Option<Offset<'_, T>> {
        self.get(self.address() + offset)
    }

    /// Inserts a region into the map.
    ///
    /// This method inserts a new memory region into the map. The region's
    /// address space must be within the map's address space and must not
    /// overlap with any existing regions.
    ///
    /// # Errors
    ///
    /// This method returns an error if the region's address space is already
    /// occupied by another region or if the region's address space is outside
    /// the map's address space.
    pub fn insert(&mut self, region: impl Into<T>) -> Result<(), Error> {
        let region = region.into();
        let address_space = region.address_space();

        if !self.address_space.includes(address_space) {
            return Err(Error::OutOfBounds(address_space, self.address_space));
        }

        if let Some((_, prev)) = self.inner.range(..=address_space.first()).last()
            && prev.address_space().intersects(address_space)
        {
            return Err(Error::Intersect(address_space, prev.address_space()));
        }

        if let Some((_, next)) = self.inner.range(address_space.first()..).next()
            && next.address_space().intersects(address_space)
        {
            return Err(Error::Intersect(address_space, next.address_space()));
        }

        self.inner.insert(address_space.first(), region);

        Ok(())
    }
}

impl<T> Map<T> {
    /// Gets an iterator over the regions.
    pub fn iter(&self) -> Iter<'_, T> {
        self.into_iter()
    }
}

impl<T> Region for Map<T>
where
    T: Region,
{
    fn address_space(&self) -> AddressSpace {
        self.address_space
    }
}

impl<T> Default for Map<T> {
    fn default() -> Self {
        Self {
            address_space: AddressSpace::default(),
            inner: BTreeMap::default(),
        }
    }
}

impl<T> Debug for Map<T>
where
    T: Region + Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let spacer = fmt::from_fn(|f| write!(f, "..."));
        let mut regions = self.iter();
        let mut dbg = f.debug_map();

        if let Some(region) = regions.next() {
            let mut space = region.address_space();

            if space.first() > self.address() {
                let range = self.address()..space.first();
                let space = AddressSpace::from_range(range).expect("ok");

                dbg.entry(&space, &spacer);
            }

            dbg.entry(&space, region);

            for region in regions {
                if !space.is_adjacent_before(region.address_space()) {
                    let range = space.next().expect("some")..region.address();
                    let space = AddressSpace::from_range(range).expect("ok");

                    dbg.entry(&space, &spacer);
                }

                space = region.address_space();

                dbg.entry(&space, region);
            }

            if space.last() < self.address_space.last() {
                let range = space.last()..=self.address_space.last();
                let space = AddressSpace::from_range(range).expect("ok");

                dbg.entry(&space, &spacer);
            }
        } else {
            dbg.entry(&self.address_space, &spacer);
        }

        dbg.finish()
    }
}

impl<T> IntoIterator for Map<T> {
    type Item = T;
    type IntoIter = IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter(self.inner.into_values())
    }
}

impl<'a, T> IntoIterator for &'a Map<T> {
    type Item = &'a T;
    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        Iter(self.inner.values())
    }
}

#[cfg(test)]
mod tests {
    use crate::memory::address::{Address, AddressSpace};
    use crate::memory::region::Region;

    use super::{Error, Map};

    struct Range(u32, u32);

    impl Region for Range {
        fn address_space(&self) -> AddressSpace {
            AddressSpace::new(Address::new(self.0), Address::new(self.1)).unwrap()
        }
    }

    #[test]
    fn test_insert_intersect() {
        let mut regions = Map::<Range>::default();

        regions.insert(Range(10, 19)).unwrap();
        regions.insert(Range(40, 59)).unwrap();
        regions.insert(Range(80, 99)).unwrap();

        assert_eq!(
            regions.insert(Range(10, 19)),
            Err(Error::Intersect(
                Range(10, 19).address_space(),
                Range(10, 19).address_space(),
            )),
            "exists",
        );

        assert_eq!(
            regions.insert(Range(11, 18)),
            Err(Error::Intersect(
                Range(11, 18).address_space(),
                Range(10, 19).address_space(),
            )),
            "inside",
        );

        assert_eq!(
            regions.insert(Range(0, 119)),
            Err(Error::Intersect(
                Range(0, 119).address_space(),
                Range(10, 19).address_space(),
            )),
            "outside",
        );

        assert_eq!(
            regions.insert(Range(0, 14)),
            Err(Error::Intersect(
                Range(0, 14).address_space(),
                Range(10, 19).address_space(),
            )),
            "top edge",
        );

        assert_eq!(
            regions.insert(Range(15, 24)),
            Err(Error::Intersect(
                Range(15, 24).address_space(),
                Range(10, 19).address_space(),
            )),
            "bottom edge",
        );

        regions.insert(Range(20, 39)).unwrap();
        regions.insert(Range(61, 78)).unwrap();
        regions.insert(Range(1000, 1999)).unwrap();
        regions.insert(Range(2000, u32::MAX)).unwrap();
        regions.insert(Range(0, 0)).unwrap();
    }

    #[test]
    fn test_insert_bounds() {
        let mut regions = Map::<Range>::new(AddressSpace::new(50.into(), 99.into()).unwrap());

        assert_eq!(
            regions.insert(Range(0, 20)),
            Err(Error::OutOfBounds(
                Range(0, 20).address_space(),
                Range(50, 99).address_space(),
            )),
            "below",
        );

        assert_eq!(
            regions.insert(Range(120, 149)),
            Err(Error::OutOfBounds(
                Range(120, 149).address_space(),
                Range(50, 99).address_space(),
            )),
            "above",
        );

        assert_eq!(
            regions.insert(Range(5, 74)),
            Err(Error::OutOfBounds(
                Range(5, 74).address_space(),
                Range(50, 99).address_space(),
            )),
            "top edge",
        );

        assert_eq!(
            regions.insert(Range(75, 119)),
            Err(Error::OutOfBounds(
                Range(75, 119).address_space(),
                Range(50, 99).address_space(),
            )),
            "bottom edge",
        );

        regions.insert(Range(50, 99)).unwrap();
    }
}
