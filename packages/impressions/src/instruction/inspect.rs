use std::fmt::{self, Display};

use crate::memory::inspect::InspectionValue;
use crate::memory::region::ops::encode::{Encode, Encoder, Error as EncodeError};

use super::Instruction;

/// A wrapper around an `Instruction` that implements `InspectionValue`.
pub struct Inspection<'a>(pub &'a Instruction);

impl Encode for Inspection<'_> {
    fn encode(&self, encoder: &mut dyn Encoder) -> Result<(), EncodeError> {
        self.0.encode(encoder)
    }
}

impl Display for Inspection<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.encode(&mut InspectionEncoder(f)).ok();

        Ok(())
    }
}

impl InspectionValue for Inspection<'_> {
    fn data_type(&self) -> &dyn Display {
        &"instruction"
    }
}

/// A wrapper around a `fmt::Formatter` that implements `Encoder`.
struct InspectionEncoder<'a, 'b>(pub &'a mut fmt::Formatter<'b>);

impl Encoder for InspectionEncoder<'_, '_> {
    fn write(&mut self, bytes: &[u8]) -> Result<(), EncodeError> {
        for (index, byte) in bytes.iter().enumerate() {
            if index > 0 {
                let _ = write!(self.0, " ");
            }

            let _ = write!(self.0, "{byte:02x}");
        }

        Ok(())
    }
}
