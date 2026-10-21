//! Bitwise exclusive OR operations.

mod xor_r32_rm32;
mod xor_rm32_imm32;
mod xor_rm32_imm8;

pub use xor_r32_rm32::XorR32Rm32;
pub use xor_rm32_imm8::XorRm32Imm8;
pub use xor_rm32_imm32::XorRm32Imm32;
