//! Mapping address spaces to memory regions.

mod error;
mod iter;
mod offset;

use std::collections::btree_map::BTreeMap;
use std::fmt::{self, Debug};

use serde::{Deserialize, Deserializer, Serialize, Serializer};

pub use self::error::Error;
pub use self::iter::{IntoIter, Iter};
pub use self::offset::Offset;

use super::address::{Address, AddressSpace};
use super::region::Region;

/// A map of address spaces to memory regions.
///
/// # Invariants
///
/// The size of each region in the map must be **immutable**. Any modifications
/// to the size may corrupt the map and potentially cause regions to overlap or
/// escape the bounds of the map.
#[derive(Clone, PartialEq, Eq)]
pub struct Map<T> {
    address_space: AddressSpace,
    regions: BTreeMap<Address, T>,
}

impl<T> Map<T> {
    /// Constructs a new memory map.
    pub const fn new(address_space: AddressSpace) -> Self {
        Self {
            address_space,
            regions: BTreeMap::new(),
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
        match self.regions.range(..=address).last() {
            Some((&region_address, region))
                if AddressSpace::with_size(region_address, region.size())
                    .expect("address spaces within map are always valid")
                    .contains(address) =>
            {
                Some(Offset::new(region, address.offset(region_address)))
            }
            _ => None,
        }
    }

    /// Gets a region for the given offset.
    ///
    /// This method is similar to [`Self::get`] except that it takes an offset
    /// instead of an address.
    pub fn get_by_offset(&self, offset: u32) -> Option<Offset<'_, T>> {
        self.get(self.address_space.first() + offset)
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
    pub fn insert(&mut self, address: Address, region: impl Into<T>) -> Result<(), Error> {
        let region = region.into();
        let address_space = AddressSpace::with_size(address, region.size())?;

        if !self.address_space.includes(address_space) {
            return Err(Error::OutOfBounds(address_space, self.address_space));
        }

        if let Some((&prev_address, prev)) = self.regions.range(..=address).last() {
            let prev_address_space = AddressSpace::with_size(prev_address, prev.size())?;

            if prev_address_space.intersects(address_space) {
                return Err(Error::Intersect(address_space, prev_address_space));
            }
        }

        if let Some((&next_address, next)) = self.regions.range(address..).next() {
            let next_address_space = AddressSpace::with_size(next_address, next.size())?;

            if next_address_space.intersects(address_space) {
                return Err(Error::Intersect(address_space, next_address_space));
            }
        }

        self.regions.insert(address, region);

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
    fn size(&self) -> u64 {
        self.address_space.size()
    }
}

impl<T> Default for Map<T> {
    fn default() -> Self {
        Self {
            address_space: AddressSpace::default(),
            regions: BTreeMap::default(),
        }
    }
}

impl<T> Debug for Map<T>
where
    T: Region + Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let spacer = fmt::from_fn(|f| write!(f, "..."));
        let mut regions = self.regions.iter();
        let mut dbg = f.debug_map();

        if let Some((&address, region)) = regions.next() {
            let mut space = AddressSpace::with_size(address, region.size()).expect("ok");

            if space.first() > self.address_space.first() {
                let range = self.address_space.first()..space.first();
                let space = AddressSpace::from_range(range).expect("ok");

                dbg.entry(&space, &spacer);
            }

            dbg.entry(&space, region);

            for (&region_address, region) in regions {
                let region_address_space =
                    AddressSpace::with_size(region_address, region.size()).expect("ok");

                if !space.is_adjacent_before(region_address_space) {
                    let range = space.next().expect("some")..region_address;
                    let space = AddressSpace::from_range(range).expect("ok");

                    dbg.entry(&space, &spacer);
                }

                space = region_address_space;

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
        IntoIter(self.regions.into_values())
    }
}

impl<'a, T> IntoIterator for &'a Map<T> {
    type Item = &'a T;
    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        Iter(self.regions.values())
    }
}

impl<T> Serialize for Map<T>
where
    T: Region + Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use serde::ser::{SerializeSeq, SerializeStruct};

        struct Regions<'a, T>(&'a BTreeMap<Address, T>);

        impl<'a, T> Serialize for Regions<'a, T>
        where
            T: Serialize,
        {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                let mut seq = serializer.serialize_seq(Some(self.0.len()))?;

                for (address, region) in self.0 {
                    seq.serialize_element(&(*address, region))?;
                }

                seq.end()
            }
        }

        let mut map = serializer.serialize_struct("Map", 2)?;

        map.serialize_field("address_space", &self.address_space)?;
        map.serialize_field("regions", &Regions(&self.regions))?;
        map.end()
    }
}

impl<'de, T> Deserialize<'de> for Map<T>
where
    T: Region + Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde::de::{DeserializeSeed, Error, MapAccess, SeqAccess, Visitor};
        use std::marker::PhantomData;

        #[derive(Deserialize)]
        #[serde(field_identifier, rename_all = "snake_case")]
        enum Field {
            AddressSpace,
            Regions,
        }

        struct RegionsSeed<T>(Map<T>);

        impl<'de, T> DeserializeSeed<'de> for RegionsSeed<T>
        where
            T: Region + Deserialize<'de>,
        {
            type Value = Map<T>;

            fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
            where
                D: Deserializer<'de>,
            {
                deserializer.deserialize_seq(RegionsVisitor(self.0))
            }
        }

        struct RegionsVisitor<T>(Map<T>);

        impl<'de, T> Visitor<'de> for RegionsVisitor<T>
        where
            T: Region + Deserialize<'de>,
        {
            type Value = Map<T>;

            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, "a sequence of regions")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let mut map = self.0;

                while let Some((address, region)) = seq.next_element::<(Address, T)>()? {
                    map.insert(address, region).map_err(A::Error::custom)?;
                }

                Ok(map)
            }
        }

        struct MapVisitor<T>(PhantomData<T>);

        impl<'de, T> Visitor<'de> for MapVisitor<T>
        where
            T: Region + Deserialize<'de>,
        {
            type Value = Map<T>;

            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, "struct Map")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Map<T>, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let address_space = seq
                    .next_element()?
                    .ok_or_else(|| A::Error::invalid_length(0, &self))?;

                let regions = seq
                    .next_element_seed(RegionsSeed(Map::new(address_space)))?
                    .ok_or_else(|| A::Error::invalid_length(1, &self))?;

                Ok(regions)
            }

            fn visit_map<A>(self, mut map: A) -> Result<Map<T>, A::Error>
            where
                A: MapAccess<'de>,
            {
                let mut address_space: Option<AddressSpace> = None;
                let mut regions: Option<Map<T>> = None;

                while let Some(key) = map.next_key()? {
                    match key {
                        Field::AddressSpace => {
                            if address_space.is_some() {
                                return Err(A::Error::duplicate_field("address_space"));
                            }

                            let space: AddressSpace = map.next_value()?;

                            if let Some(regions) = &mut regions {
                                let iter = regions
                                    .regions
                                    .iter()
                                    .next()
                                    .into_iter()
                                    .chain(regions.regions.iter().last());

                                for (address, region) in iter {
                                    let region_space =
                                        AddressSpace::with_size(*address, region.size())
                                            .map_err(A::Error::custom)?;

                                    if !space.includes(region_space) {
                                        return Err(A::Error::custom(self::Error::OutOfBounds(
                                            region_space,
                                            space,
                                        )));
                                    }
                                }

                                regions.address_space = space;
                            }

                            address_space = Some(space);
                        }
                        Field::Regions => {
                            if regions.is_some() {
                                return Err(A::Error::duplicate_field("regions"));
                            }

                            let seed = match address_space {
                                Some(address_space) => Map::new(address_space),
                                None => Map::default(),
                            };

                            regions = Some(map.next_value_seed(RegionsSeed(seed))?);
                        }
                    }
                }

                if address_space.is_none() {
                    return Err(A::Error::missing_field("address_space"));
                }

                regions.ok_or_else(|| A::Error::missing_field("regions"))
            }
        }

        static FIELDS: &[&str] = &["address_space", "regions"];

        deserializer.deserialize_struct("Map", FIELDS, MapVisitor::<T>(PhantomData))
    }
}

#[cfg(test)]
mod tests {
    use serde::{Deserialize, Serialize};

    use crate::memory::address::AddressSpace;
    use crate::memory::region::Region;

    use super::{Error, Map};

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct Node(u64);

    impl Region for Node {
        fn size(&self) -> u64 {
            self.0
        }
    }

    #[test]
    fn test_get() {
        let mut regions = Map::<Node>::new(AddressSpace::new(30.into(), 89.into()).unwrap());

        regions.insert(30.into(), Node(20)).unwrap();
        regions.insert(80.into(), Node(5)).unwrap();

        let a = regions.get(30.into()).unwrap();
        let b = regions.get(34.into()).unwrap();
        let c = regions.get(49.into()).unwrap();

        assert_eq!(a.offset(), 0);
        assert_eq!(b.offset(), 4);
        assert_eq!(c.offset(), 19);

        assert_eq!(a.region(), &Node(20));
        assert_eq!(b.region(), &Node(20));
        assert_eq!(c.region(), &Node(20));

        let d = regions.get(80.into()).unwrap();
        let e = regions.get(81.into()).unwrap();
        let f = regions.get(84.into()).unwrap();

        assert_eq!(d.offset(), 0);
        assert_eq!(e.offset(), 1);
        assert_eq!(f.offset(), 4);

        assert_eq!(d.region(), &Node(5));
        assert_eq!(e.region(), &Node(5));
        assert_eq!(f.region(), &Node(5));

        assert!(regions.get(0.into()).is_none());
        assert!(regions.get(29.into()).is_none());
        assert!(regions.get(50.into()).is_none());
        assert!(regions.get(79.into()).is_none());
        assert!(regions.get(85.into()).is_none());
        assert!(regions.get(99.into()).is_none());

        let g = regions.get_by_offset(0).unwrap();
        let h = regions.get_by_offset(4).unwrap();
        let i = regions.get_by_offset(19).unwrap();

        assert_eq!(g.region(), &Node(20));
        assert_eq!(h.region(), &Node(20));
        assert_eq!(i.region(), &Node(20));

        assert!(regions.get_by_offset(20).is_none());
    }

    #[test]
    fn test_insert_intersect() {
        let mut regions = Map::<Node>::default();

        regions.insert(10.into(), Node(10)).unwrap();
        regions.insert(40.into(), Node(20)).unwrap();
        regions.insert(80.into(), Node(20)).unwrap();

        assert_eq!(
            regions.insert(10.into(), Node(10)),
            Err(Error::Intersect(
                AddressSpace::new(10.into(), 19.into()).unwrap(),
                AddressSpace::new(10.into(), 19.into()).unwrap(),
            )),
            "exists",
        );

        assert_eq!(
            regions.insert(11.into(), Node(8)),
            Err(Error::Intersect(
                AddressSpace::new(11.into(), 18.into()).unwrap(),
                AddressSpace::new(10.into(), 19.into()).unwrap(),
            )),
            "inside",
        );

        assert_eq!(
            regions.insert(0.into(), Node(120)),
            Err(Error::Intersect(
                AddressSpace::new(0.into(), 119.into()).unwrap(),
                AddressSpace::new(10.into(), 19.into()).unwrap(),
            )),
            "outside",
        );

        assert_eq!(
            regions.insert(0.into(), Node(15)),
            Err(Error::Intersect(
                AddressSpace::new(0.into(), 14.into()).unwrap(),
                AddressSpace::new(10.into(), 19.into()).unwrap(),
            )),
            "top edge",
        );

        assert_eq!(
            regions.insert(15.into(), Node(10)),
            Err(Error::Intersect(
                AddressSpace::new(15.into(), 24.into()).unwrap(),
                AddressSpace::new(10.into(), 19.into()).unwrap(),
            )),
            "bottom edge",
        );

        regions.insert(20.into(), Node(20)).unwrap();
        regions.insert(61.into(), Node(18)).unwrap();
        regions.insert(1000.into(), Node(1000)).unwrap();
        regions
            .insert(2000.into(), Node(u32::MAX as u64 - 2000 + 1))
            .unwrap();
        regions.insert(0.into(), Node(1)).unwrap();
    }

    #[test]
    fn test_insert_bounds() {
        let mut regions = Map::<Node>::new(AddressSpace::new(50.into(), 99.into()).unwrap());

        assert_eq!(
            regions.insert(0.into(), Node(21)),
            Err(Error::OutOfBounds(
                AddressSpace::new(0.into(), 20.into()).unwrap(),
                AddressSpace::new(50.into(), 99.into()).unwrap(),
            )),
            "below",
        );

        assert_eq!(
            regions.insert(120.into(), Node(30)),
            Err(Error::OutOfBounds(
                AddressSpace::new(120.into(), 149.into()).unwrap(),
                AddressSpace::new(50.into(), 99.into()).unwrap(),
            )),
            "above",
        );

        assert_eq!(
            regions.insert(5.into(), Node(70)),
            Err(Error::OutOfBounds(
                AddressSpace::new(5.into(), 74.into()).unwrap(),
                AddressSpace::new(50.into(), 99.into()).unwrap(),
            )),
            "top edge",
        );

        assert_eq!(
            regions.insert(75.into(), Node(45)),
            Err(Error::OutOfBounds(
                AddressSpace::new(75.into(), 119.into()).unwrap(),
                AddressSpace::new(50.into(), 99.into()).unwrap(),
            )),
            "bottom edge",
        );

        regions.insert(50.into(), Node(50)).unwrap();
    }

    #[test]
    fn test_serde() {
        let mut map = Map::<Node>::new(AddressSpace::new(10.into(), 49.into()).unwrap());
        let map_str = serde_json::to_string(&map).unwrap();

        assert_eq!(map_str, "{\"address_space\":[10,49],\"regions\":[]}");
        assert_eq!(serde_json::from_str::<Map<Node>>(&map_str).unwrap(), map);

        map.insert(10.into(), Node(1)).unwrap();
        map.insert(20.into(), Node(26)).unwrap();

        let map_str = serde_json::to_string(&map).unwrap();

        assert_eq!(
            map_str,
            "{\"address_space\":[10,49],\"regions\":[[10,1],[20,26]]}"
        );
        assert_eq!(serde_json::from_str::<Map<Node>>(&map_str).unwrap(), map);

        assert_eq!(
            serde_json::from_str::<Map<Node>>(
                "{\"regions\":[[10,1],[20,26]],\"address_space\":[10,49]}"
            )
            .unwrap(),
            map
        );

        assert!(
            serde_json::from_str::<Map<Node>>(
                "{\"address_space\":[10,39],\"regions\":[[10,1],[20,26]]}"
            )
            .unwrap_err()
            .is_data()
        );
        assert!(
            serde_json::from_str::<Map<Node>>(
                "{\"regions\":[[10,1],[20,26]],\"address_space\":[10,39]}"
            )
            .unwrap_err()
            .is_data()
        );

        assert!(
            serde_json::from_str::<Map<Node>>(
                "{\"address_space\":[12,49],\"regions\":[[10,1],[20,26]]}"
            )
            .unwrap_err()
            .is_data()
        );
        assert!(
            serde_json::from_str::<Map<Node>>(
                "{\"regions\":[[10,1],[20,26]],\"address_space\":[12,49]}"
            )
            .unwrap_err()
            .is_data()
        );
    }
}
