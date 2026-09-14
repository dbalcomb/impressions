use crate::memory::address::{Address, AddressSpace};
use crate::memory::inspect::{Record, RecordBuilder};

use super::Inspector;

/// An inspector adapter that translates child-local address spaces to its
/// parent's coordinate system.
pub struct RebasedInspector<'a> {
    inspector: &'a mut dyn Inspector,
    address: Address,
}

impl<'a> RebasedInspector<'a> {
    /// Constructs an inspector that rebases child records at the given address.
    pub const fn new(inspector: &'a mut dyn Inspector, address: Address) -> Self {
        Self { inspector, address }
    }
}

impl Inspector for RebasedInspector<'_> {
    fn emit(&mut self, record: Record<'_>) {
        let address_space = record.address_space();
        let address = self
            .address
            .checked_add(address_space.first().value())
            .expect("child record address space fits in its parent");

        let address_space = address_space
            .rebase(address)
            .expect("child record address space fits in its parent");

        self.inspector
            .emit(record.with_address_space(address_space));
    }

    fn record(&mut self, address_space: AddressSpace) -> RecordBuilder<'_> {
        RecordBuilder::new(self, address_space)
    }
}
