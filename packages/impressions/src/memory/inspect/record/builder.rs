use std::fmt::Display;

use crate::memory::address::AddressSpace;
use crate::memory::inspect::{InspectionValue, Inspector, Status};

use super::Record;

/// A region inspection record builder.
pub struct RecordBuilder<'a> {
    inspector: &'a mut dyn Inspector,
    address_space: AddressSpace,
    status: Option<Status>,
    label: Option<&'a dyn Display>,
    value: Option<&'a dyn InspectionValue>,
}

impl<'a> RecordBuilder<'a> {
    /// Constructs a new region inspection record builder.
    pub const fn new(inspector: &'a mut dyn Inspector, address_space: AddressSpace) -> Self {
        Self {
            inspector,
            address_space,
            status: None,
            label: None,
            value: None,
        }
    }
}

impl RecordBuilder<'_> {
    /// Sets the record status.
    pub const fn status(&mut self, status: Status) -> &mut Self {
        self.status = Some(status);
        self
    }

    /// Sets the record status to identified.
    pub const fn identified(&mut self) -> &mut Self {
        self.status(Status::Identified)
    }

    /// Sets the record status to unidentified.
    pub const fn unidentified(&mut self) -> &mut Self {
        self.status(Status::Unidentified)
    }

    /// Sets the record status to vacant.
    pub const fn vacant(&mut self) -> &mut Self {
        self.status(Status::Vacant)
    }
}

impl<'a> RecordBuilder<'a> {
    /// Sets the record label.
    pub const fn label(&mut self, label: &'a dyn Display) -> &mut Self {
        self.label = Some(label);
        self
    }

    /// Sets the record value.
    pub const fn value(&mut self, value: &'a dyn InspectionValue) -> &mut Self {
        self.value = Some(value);
        self
    }
}

impl<'a> RecordBuilder<'a> {
    /// Emits an identified record for a labelled field value.
    pub fn field(&mut self, label: &'a dyn Display, value: &'a dyn InspectionValue) {
        self.identified().label(label).value(value).finish();
    }

    /// Emits the completed record.
    pub fn finish(&mut self) {
        self.inspector.emit(Record {
            address_space: self.address_space,
            status: self.status,
            label: self.label,
            value: self.value,
        });
    }
}
