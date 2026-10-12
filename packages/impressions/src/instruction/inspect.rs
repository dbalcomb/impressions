use std::fmt::{self, Display};
use std::io::Error as IoError;

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
        let mut encoder = InspectionEncoder {
            writer: f,
            first: true,
        };

        self.encode(&mut encoder).map_err(|_| fmt::Error)
    }
}

impl InspectionValue for Inspection<'_> {
    fn data_type(&self) -> &dyn Display {
        &"instruction"
    }
}

/// A wrapper around a writer that implements `Encoder`.
struct InspectionEncoder<'a> {
    writer: &'a mut dyn fmt::Write,
    first: bool,
}

impl Encoder for InspectionEncoder<'_> {
    fn write(&mut self, bytes: &[u8]) -> Result<(), EncodeError> {
        for byte in bytes {
            if !self.first {
                self.writer.write_str(" ").map_err(IoError::other)?;
            }

            write!(self.writer, "{byte:02x}").map_err(IoError::other)?;

            self.first = false;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::memory::region::ops::decode::Decode;

    use super::*;

    #[test]
    fn spaces_bytes_within_and_between_writes() {
        let mut output = String::new();
        let mut encoder = InspectionEncoder {
            writer: &mut output,
            first: true,
        };

        encoder.write(&[]).unwrap();
        encoder.write(&[0x8b]).unwrap();
        encoder.write(&[]).unwrap();
        encoder.write(&[0x05]).unwrap();
        encoder.write_u32_le(0x12345678).unwrap();
        encoder.write(&[]).unwrap();

        assert_eq!(output, "8b 05 78 56 34 12");
    }

    #[test]
    fn displays_instruction_bytes_with_spaces() {
        for bytes in [
            &b"\x53"[..],
            &b"\x8b\x05\x78\x56\x34\x12"[..],
            &b"\x8b\x84\x8b\x78\x56\x34\x12"[..],
        ] {
            let instruction = Instruction::decode(&mut &bytes[..]).unwrap();
            let expected = bytes
                .iter()
                .map(|byte| format!("{byte:02x}"))
                .collect::<Vec<_>>()
                .join(" ");

            assert_eq!(Inspection(&instruction).to_string(), expected);
        }
    }
}
