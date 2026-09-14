use std::fmt::Write;

use impressions::memory::address::AddressSpace;
use impressions::memory::inspect::{Record, Status};

/// A stored record in a table inspector.
#[derive(Default, Eq, PartialEq)]
pub struct StoredRecord {
    pub address_space: AddressSpace,
    pub status: Option<Status>,
    pub label: String,
    pub value: String,
    pub data_type: String,
}

impl StoredRecord {
    /// Stores the given record, replacing any previous contents while retaining
    /// allocated capacity.
    pub fn store(&mut self, record: Record<'_>) {
        self.address_space = record.address_space();
        self.status = record.status();
        self.label.clear();
        self.value.clear();
        self.data_type.clear();

        if let Some(label) = record.label() {
            write!(self.label, "{label}").expect("formatting into a string does not fail");
        }

        if let Some(value) = record.value() {
            write!(self.value, "{value}").expect("formatting into a string does not fail");
            write!(self.data_type, "{}", value.data_type())
                .expect("formatting into a string does not fail");
        }
    }
}
