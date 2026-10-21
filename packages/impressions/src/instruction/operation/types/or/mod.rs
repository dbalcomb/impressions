//! Bitwise OR operations.

mod or_r32_rm32;
mod or_rm32_imm32;
mod or_rm32_imm8;

pub use or_r32_rm32::OrR32Rm32;
pub use or_rm32_imm8::OrRm32Imm8;
pub use or_rm32_imm32::OrRm32Imm32;
