use std::fmt::{self, Display};

use crate::instruction::Mnemonic;
use crate::instruction::operand::immediate::Immediate32;
use crate::memory::extent::{Extent, Size};
use crate::memory::region::ops::encode::{Encode, Encoder, Error as EncodeError};

/// Push a 32-bit immediate onto the stack.
///
/// `68 id => push imm32`
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PushImm32 {
    immediate: Immediate32,
}

impl PushImm32 {
    /// Creates the operation from its encoded operands.
    pub(in crate::instruction::operation) const fn new(immediate: Immediate32) -> Self {
        Self { immediate }
    }
}

impl PushImm32 {
    /// Gets the immediate operand.
    pub const fn immediate(&self) -> Immediate32 {
        self.immediate
    }

    /// Gets the mnemonic.
    pub const fn mnemonic(&self) -> Mnemonic {
        Mnemonic::Push
    }
}

impl Extent for PushImm32 {
    fn size(&self) -> Size {
        Size::new_valid(5)
    }
}

impl Encode for PushImm32 {
    fn encode(&self, encoder: &mut dyn Encoder) -> Result<(), EncodeError> {
        encoder.write_u8(0x68)?;
        self.immediate.encode(encoder)
    }
}

impl Display for PushImm32 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.mnemonic(), self.immediate)
    }
}

#[cfg(test)]
mod tests {
    use crate::instruction::Mnemonic;
    use crate::instruction::operand::immediate::Immediate32;
    use crate::instruction::operation::Operation;
    use crate::instruction::prefix::Prefixes;
    use crate::memory::extent::Extent;
    use crate::memory::region::ops::decode::Decode;
    use crate::memory::region::ops::encode::Encode;

    use super::PushImm32;

    #[test]
    fn supports_signed_values_and_little_endian_encoding() {
        for (bytes, value, display) in [
            (&b"\x68\x00\x00\x00\x00"[..], 0, "push 0"),
            (&b"\x68\x01\x00\x00\x00"[..], 1, "push 1"),
            (&b"\x68\xff\xff\xff\xff"[..], -1, "push -1"),
            (&b"\x68\x80\xff\xff\xff"[..], -128, "push -128"),
            (&b"\x68\x7f\x00\x00\x00"[..], 127, "push 127"),
            (&b"\x68\x78\x56\x34\x12"[..], 0x12345678, "push 305419896"),
            (&b"\x68\x00\x00\x00\x80"[..], i32::MIN, "push -2147483648"),
            (&b"\x68\xff\xff\xff\x7f"[..], i32::MAX, "push 2147483647"),
        ] {
            let mut decoder = bytes;
            let operation = Operation::decode_with(&mut decoder, &Prefixes::new()).unwrap();
            let mut encoder = Vec::new();

            operation.encode(&mut encoder).unwrap();

            assert_eq!(
                operation,
                Operation::PushImm32(PushImm32::new(Immediate32::new(value)))
            );
            assert_eq!(operation.mnemonic(), Mnemonic::Push);
            assert_eq!(operation.to_string(), display);
            assert_eq!(operation.size(), 5);
            assert_eq!(encoder, bytes);
            assert!(decoder.is_empty());

            let Operation::PushImm32(immediate) = operation else {
                panic!("expected immediate push");
            };

            assert_eq!(immediate.immediate().value(), value);
        }
    }
}
