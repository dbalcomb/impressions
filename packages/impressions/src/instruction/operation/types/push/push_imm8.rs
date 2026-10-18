use std::fmt::{self, Display};

use crate::instruction::Mnemonic;
use crate::instruction::operand::immediate::Immediate8;
use crate::memory::extent::{Extent, Size};
use crate::memory::region::ops::encode::{Encode, Encoder, Error as EncodeError};

/// Push an 8-bit immediate onto the stack, sign-extended to 32 bits.
///
/// `6A ib => push imm8`
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PushImm8 {
    immediate: Immediate8,
}

impl PushImm8 {
    /// Creates the operation from its encoded operands.
    pub(in crate::instruction::operation) const fn new(immediate: Immediate8) -> Self {
        Self { immediate }
    }
}

impl PushImm8 {
    /// Gets the immediate operand.
    pub const fn immediate(&self) -> Immediate8 {
        self.immediate
    }

    /// Gets the mnemonic.
    pub const fn mnemonic(&self) -> Mnemonic {
        Mnemonic::Push
    }
}

impl Extent for PushImm8 {
    fn size(&self) -> Size {
        Size::new_valid(2)
    }
}

impl Encode for PushImm8 {
    fn encode(&self, encoder: &mut dyn Encoder) -> Result<(), EncodeError> {
        encoder.write_u8(0x6a)?;
        self.immediate.encode(encoder)
    }
}

impl Display for PushImm8 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.mnemonic(), self.immediate)
    }
}

#[cfg(test)]
mod tests {
    use crate::instruction::Mnemonic;
    use crate::instruction::operand::immediate::Immediate8;
    use crate::instruction::operation::Operation;
    use crate::instruction::prefix::Prefixes;
    use crate::memory::extent::Extent;
    use crate::memory::region::ops::decode::Decode;
    use crate::memory::region::ops::encode::Encode;

    use super::PushImm8;

    #[test]
    fn supports_all_byte_values() {
        for value in i8::MIN..=i8::MAX {
            let bytes = [0x6a, value as u8];
            let mut decoder = bytes.as_slice();
            let operation = Operation::decode_with(&mut decoder, &Prefixes::new()).unwrap();
            let mut encoder = Vec::new();

            operation.encode(&mut encoder).unwrap();

            assert_eq!(
                operation,
                Operation::PushImm8(PushImm8::new(Immediate8::new(value)))
            );
            assert_eq!(operation.mnemonic(), Mnemonic::Push);
            assert_eq!(operation.to_string(), format!("push {value}"));
            assert_eq!(operation.size(), 2);
            assert_eq!(encoder, bytes);
            assert!(decoder.is_empty());

            let Operation::PushImm8(immediate) = operation else {
                panic!("expected immediate push");
            };

            assert_eq!(immediate.immediate().value(), value);
        }
    }
}
