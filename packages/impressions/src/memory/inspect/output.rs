use std::fmt::{self, Write};

use super::{Error, Inspector};

/// The output writer of a memory region inspector.
pub struct Output<'a, 'b> {
    inspector: &'a mut Inspector<'b>,
    status: Status,
    start: bool,
}

impl<'a, 'b> Output<'a, 'b> {
    /// Constructs a new output writer.
    pub(super) const fn new(inspector: &'a mut Inspector<'b>, status: Status) -> Self {
        Self {
            inspector,
            status,
            start: true,
        }
    }

    /// Constructs a new output writer for an identified memory region.
    pub(super) const fn identified(inspector: &'a mut Inspector<'b>) -> Self {
        Self::new(inspector, Status::Identified)
    }

    /// Constructs a new output writer for an unidentified memory region.
    pub(super) const fn unidentified(inspector: &'a mut Inspector<'b>) -> Self {
        Self::new(inspector, Status::Unidentified)
    }

    /// Constructs a new output writer for a vacant memory region.
    pub(super) const fn vacant(inspector: &'a mut Inspector<'b>) -> Self {
        Self::new(inspector, Status::Vacant)
    }
}

impl Output<'_, '_> {
    /// Resets the depth of the output.
    pub const fn reset(&mut self) {
        self.inspector.reset();
    }

    /// Increments the depth of the output.
    pub const fn nest(&mut self) {
        self.inspector.nest();
    }
}

impl Output<'_, '_> {
    /// Writes formatted data to the output.
    pub fn write_fmt(&mut self, args: fmt::Arguments<'_>) -> Result<(), Error> {
        Write::write_fmt(self, args).map_err(Into::into)
    }
}

impl fmt::Write for Output<'_, '_> {
    fn write_str(&mut self, value: &str) -> fmt::Result {
        for line in value.split_inclusive('\n') {
            if self.start {
                let writer = &mut self.inspector.writer;

                match self.status {
                    Status::Identified if self.inspector.depth == 0 => {
                        write!(writer, "\x1b[1m{}  >\x1b[0m  ", self.inspector.address)?;
                    }
                    Status::Identified if self.inspector.depth > 1 => {
                        write!(writer, "\x1b[32m{}\x1b[0m     ", self.inspector.address)?;
                    }
                    Status::Identified => {
                        write!(writer, "{}     ", self.inspector.address)?;
                    }
                    Status::Unidentified => {
                        write!(writer, "\x1b[31m{}  ?\x1b[0m  ", self.inspector.address)?;
                    }
                    Status::Vacant => {
                        write!(writer, "\x1b[2m{}  :\x1b[0m  ", self.inspector.address)?;
                    }
                }

                for _ in 0..self.inspector.depth {
                    write!(self.inspector.writer, "    ")?;
                }
            }

            self.inspector.writer.write_str(line)?;
            self.start = line.ends_with('\n');
        }

        Ok(())
    }
}

/// The status of a memory region.
pub enum Status {
    Identified,
    Unidentified,
    Vacant,
}
