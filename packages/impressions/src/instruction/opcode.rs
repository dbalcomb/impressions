use super::operand::register::Register32;

/// An x86 instruction opcode.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Opcode {
    /// Add a 32-bit register or memory value to a 32-bit register.
    ///
    /// `03 /r => add r32, r/m32`
    AddR32Rm32,

    /// Bitwise OR a 32-bit register with a 32-bit register or memory value.
    ///
    /// `0B /r => or r32, r/m32`
    OrR32Rm32,

    /// Add a 32-bit register or memory value and carry to a 32-bit register.
    ///
    /// `13 /r => adc r32, r/m32`
    AdcR32Rm32,

    /// Subtract a 32-bit register or memory value and borrow from a 32-bit
    /// register.
    ///
    /// `1B /r => sbb r32, r/m32`
    SbbR32Rm32,

    /// Bitwise AND a 32-bit register with a 32-bit register or memory value.
    ///
    /// `23 /r => and r32, r/m32`
    AndR32Rm32,

    /// Subtract a 32-bit register or memory value from a 32-bit register.
    ///
    /// `2B /r => sub r32, r/m32`
    SubR32Rm32,

    /// Bitwise XOR a 32-bit register with a 32-bit register or memory value.
    ///
    /// `33 /r => xor r32, r/m32`
    XorR32Rm32,

    /// Compare a 32-bit register with a 32-bit register or memory value.
    ///
    /// `3B /r => cmp r32, r/m32`
    CmpR32Rm32,

    /// Push a 32-bit register onto the stack.
    ///
    /// `50+rd => push r32`
    PushR32(Register32),

    /// Push a 32-bit immediate onto the stack.
    ///
    /// `68 id => push imm32`
    PushImm32,

    /// Push an 8-bit immediate onto the stack, sign-extended to 32 bits.
    ///
    /// `6A ib => push imm8`
    PushImm8,

    /// Add a 32-bit immediate to r/m32.
    ///
    /// `81 /0 id => add r/m32, imm32`
    AddRm32Imm32,

    /// Bitwise OR r/m32 with a 32-bit immediate.
    ///
    /// `81 /1 id => or r/m32, imm32`
    OrRm32Imm32,

    /// Add a 32-bit immediate and carry to r/m32.
    ///
    /// `81 /2 id => adc r/m32, imm32`
    AdcRm32Imm32,

    /// Subtract a 32-bit immediate and borrow from r/m32.
    ///
    /// `81 /3 id => sbb r/m32, imm32`
    SbbRm32Imm32,

    /// Bitwise AND r/m32 with a 32-bit immediate.
    ///
    /// `81 /4 id => and r/m32, imm32`
    AndRm32Imm32,

    /// Subtract a 32-bit immediate from r/m32.
    ///
    /// `81 /5 id => sub r/m32, imm32`
    SubRm32Imm32,

    /// Bitwise XOR r/m32 with a 32-bit immediate.
    ///
    /// `81 /6 id => xor r/m32, imm32`
    XorRm32Imm32,

    /// Compare r/m32 with a 32-bit immediate.
    ///
    /// `81 /7 id => cmp r/m32, imm32`
    CmpRm32Imm32,

    /// Add a sign-extended byte immediate to r/m32.
    ///
    /// `83 /0 ib => add r/m32, imm8`
    AddRm32Imm8,

    /// Bitwise OR r/m32 with a sign-extended byte immediate.
    ///
    /// `83 /1 ib => or r/m32, imm8`
    OrRm32Imm8,

    /// Add a sign-extended byte immediate and carry to r/m32.
    ///
    /// `83 /2 ib => adc r/m32, imm8`
    AdcRm32Imm8,

    /// Subtract a sign-extended byte immediate and borrow from r/m32.
    ///
    /// `83 /3 ib => sbb r/m32, imm8`
    SbbRm32Imm8,

    /// Bitwise AND r/m32 with a sign-extended byte immediate.
    ///
    /// `83 /4 ib => and r/m32, imm8`
    AndRm32Imm8,

    /// Subtract a sign-extended byte immediate from r/m32.
    ///
    /// `83 /5 ib => sub r/m32, imm8`
    SubRm32Imm8,

    /// Bitwise XOR r/m32 with a sign-extended byte immediate.
    ///
    /// `83 /6 ib => xor r/m32, imm8`
    XorRm32Imm8,

    /// Compare r/m32 with a sign-extended byte immediate.
    ///
    /// `83 /7 ib => cmp r/m32, imm8`
    CmpRm32Imm8,

    /// Move a 32-bit register value into a 32-bit register or memory location.
    ///
    /// `89 /r => mov r/m32, r32`
    MovRm32R32,

    /// Move an 8-bit register or memory value into an 8-bit register.
    ///
    /// `8A /r => mov r8, r/m8`
    MovR8Rm8,

    /// Move a 32-bit register or memory value into a 32-bit register.
    ///
    /// `8B /r => mov r32, r/m32`
    MovR32Rm32,

    /// Move a dword at a direct 32-bit memory offset to EAX.
    ///
    /// `A1 moffs32 => mov eax, moffs32`
    MovEaxMoffs32,

    /// Move EAX to a dword at a direct 32-bit memory offset.
    ///
    /// `A3 moffs32 => mov moffs32, eax`
    MovMoffs32Eax,

    /// Rotate a 32-bit register or memory value left by an unsigned byte count.
    ///
    /// `C1 /0 ib => rol r/m32, imm8`
    RolRm32Imm8,

    /// Rotate a 32-bit register or memory value right by an unsigned byte
    /// count.
    ///
    /// `C1 /1 ib => ror r/m32, imm8`
    RorRm32Imm8,

    /// Rotate a 32-bit register or memory value left through carry by an
    /// unsigned byte count.
    ///
    /// `C1 /2 ib => rcl r/m32, imm8`
    RclRm32Imm8,

    /// Rotate a 32-bit register or memory value right through carry by an
    /// unsigned byte count.
    ///
    /// `C1 /3 ib => rcr r/m32, imm8`
    RcrRm32Imm8,

    /// Shift a 32-bit register or memory value left by an unsigned byte count.
    ///
    /// `C1 /4 ib => shl r/m32, imm8`
    ShlRm32Imm8,

    /// Shift a 32-bit register or memory value right logically by an unsigned
    /// byte count.
    ///
    /// `C1 /5 ib => shr r/m32, imm8`
    ShrRm32Imm8,

    /// Shift a 32-bit register or memory value right arithmetically by an
    /// unsigned byte count.
    ///
    /// `C1 /7 ib => sar r/m32, imm8`
    SarRm32Imm8,

    /// Call a near procedure indirectly through r/m32.
    ///
    /// `FF /2 => call r/m32`
    CallRm32,
}
