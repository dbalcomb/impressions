use std::fmt::{self, Display};

use crate::instruction::Mnemonic;
use crate::instruction::operand::register::Register32;
use crate::memory::extent::{Extent, Size};
use crate::memory::region::ops::encode::{Encode, Encoder, Error as EncodeError};

/// Push a 32-bit register onto the stack.
///
/// `50+rd => push r32`
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PushR32 {
    register: Register32,
}

impl PushR32 {
    /// Creates the operation from its encoded operands.
    pub(in crate::instruction::operation) const fn new(register: Register32) -> Self {
        Self { register }
    }
}

impl PushR32 {
    /// Gets the register operand.
    pub const fn register(&self) -> Register32 {
        self.register
    }

    /// Gets the mnemonic.
    pub const fn mnemonic(&self) -> Mnemonic {
        Mnemonic::Push
    }
}

impl Extent for PushR32 {
    fn size(&self) -> Size {
        Size::new_valid(1)
    }
}

impl Encode for PushR32 {
    fn encode(&self, encoder: &mut dyn Encoder) -> Result<(), EncodeError> {
        encoder.write_u8(0x50 | self.register.index())
    }
}

impl Display for PushR32 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.mnemonic(), self.register)
    }
}

#[cfg(test)]
mod tests {
    use crate::instruction::Mnemonic;
    use crate::instruction::operand::register::Register32;
    use crate::instruction::operation::Operation;
    use crate::instruction::prefix::Prefixes;
    use crate::memory::extent::Extent;
    use crate::memory::region::ops::decode::Decode;
    use crate::memory::region::ops::encode::Encode;

    use super::PushR32;

    #[test]
    fn supports_all_register_forms() {
        for (opcode, register, display) in [
            (0x50, Register32::Eax, "push eax"),
            (0x51, Register32::Ecx, "push ecx"),
            (0x52, Register32::Edx, "push edx"),
            (0x53, Register32::Ebx, "push ebx"),
            (0x54, Register32::Esp, "push esp"),
            (0x55, Register32::Ebp, "push ebp"),
            (0x56, Register32::Esi, "push esi"),
            (0x57, Register32::Edi, "push edi"),
        ] {
            let bytes = [opcode];
            let mut decoder = bytes.as_slice();
            let operation = Operation::decode_with(&mut decoder, &Prefixes::new()).unwrap();
            let mut encoder = Vec::new();

            operation.encode(&mut encoder).unwrap();

            assert_eq!(operation, Operation::PushR32(PushR32::new(register)));
            assert_eq!(operation.mnemonic(), Mnemonic::Push);
            assert_eq!(operation.to_string(), display);
            assert_eq!(operation.size(), 1);
            assert_eq!(encoder, bytes);
            assert!(decoder.is_empty());
        }
    }
}
