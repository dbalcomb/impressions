use std::mem;

use smallvec::SmallVec;

use super::record::StoredRecord;

/// The memory region inspection record history.
///
/// This retains records beyond the current length to ensure that allocations
/// are reused.
pub struct History {
    records: SmallVec<[StoredRecord; 8]>,
    len: usize,
}

impl History {
    /// Constructs a new record history.
    pub fn new() -> Self {
        Self {
            records: SmallVec::new(),
            len: 0,
        }
    }
}

impl History {
    /// Finds the position for the next record in the active hierarchy.
    pub fn position_for(&self, record: &StoredRecord) -> usize {
        let active = &self.records[..self.len];

        active
            .iter()
            .rposition(|existing| existing == record)
            .or_else(|| {
                active
                    .iter()
                    .rposition(|existing| existing.address_space.includes(record.address_space))
                    .map(|position| position + 1)
            })
            .unwrap_or(0)
    }

    /// Gets the record at a position in the active hierarchy.
    pub fn get(&self, position: usize) -> Option<&StoredRecord> {
        self.records.get(position)
    }

    /// Stores a record at a hierarchy position, retaining replaced allocations.
    pub fn store(&mut self, position: usize, buffer: &mut StoredRecord) -> bool {
        if position == self.records.len() {
            self.records.push(StoredRecord::default());
        }

        let changed = *buffer != self.records[position];

        if changed {
            mem::swap(buffer, &mut self.records[position]);
        }

        self.len = position + 1;

        changed
    }
}
