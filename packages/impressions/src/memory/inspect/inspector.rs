use std::fmt::Write;

use crate::memory::address::Address;

use super::Output;

/// The memory region inspector.
pub struct Inspector<'a> {
    pub(super) writer: &'a mut (dyn Write + 'a),
    pub(super) address: Address,
    pub(super) depth: u8,
}

impl<'a> Inspector<'a> {
    /// Constructs a new memory region inspector.
    pub const fn new(address: Address, writer: &'a mut (dyn Write + 'a)) -> Self {
        Self {
            address,
            writer,
            depth: 0,
        }
    }
}

impl<'a> Inspector<'a> {
    /// Gets an output writer for an identified memory region.
    pub const fn identified(&mut self) -> Output<'_, 'a> {
        Output::identified(self)
    }

    /// Gets an output writer for an unidentified memory region.
    pub const fn unidentified(&mut self) -> Output<'_, 'a> {
        Output::unidentified(self)
    }

    /// Gets an output writer for a vacant memory region.
    pub const fn vacant(&mut self) -> Output<'_, 'a> {
        Output::vacant(self)
    }
}

impl Inspector<'_> {
    /// Resets the depth of the inspector.
    pub const fn reset(&mut self) {
        self.depth = 0;
    }

    /// Increments the depth of the inspector.
    pub const fn nest(&mut self) {
        self.depth = self.depth.saturating_add(1);
    }
}
