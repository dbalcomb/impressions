mod builder;

use std::fmt::Display;

use crate::memory::address::AddressSpace;

use super::{InspectionValue, Status};

pub use self::builder::RecordBuilder;

/// A region inspection record.
pub struct Record<'a> {
    address_space: AddressSpace,
    status: Option<Status>,
    label: Option<&'a dyn Display>,
    value: Option<&'a dyn InspectionValue>,
}

impl Record<'_> {
    /// Builds the record with the given address space.
    pub(super) const fn with_address_space(mut self, address_space: AddressSpace) -> Self {
        self.address_space = address_space;
        self
    }
}

impl Record<'_> {
    /// Gets the address space covered by this record.
    pub const fn address_space(&self) -> AddressSpace {
        self.address_space
    }

    /// Gets the record status.
    pub const fn status(&self) -> Option<Status> {
        self.status
    }

    /// Gets the record label.
    pub const fn label(&self) -> Option<&dyn Display> {
        self.label
    }

    /// Gets the record value.
    pub const fn value(&self) -> Option<&dyn InspectionValue> {
        self.value
    }
}
