mod rebased;

use crate::memory::address::{Address, AddressSpace};

use super::{Record, RecordBuilder};

pub use self::rebased::RebasedInspector;

/// A region inspector that emits records for a memory region.
pub trait Inspector {
    /// Emits a single record.
    fn emit(&mut self, record: Record<'_>);

    /// Builds a record to be emitted by this inspector.
    fn record(&mut self, address_space: AddressSpace) -> RecordBuilder<'_>;
}

impl dyn Inspector + '_ {
    /// Adapts this inspector to rebase child-local address spaces.
    pub fn at(&mut self, address: Address) -> RebasedInspector<'_> {
        RebasedInspector::new(self, address)
    }
}
