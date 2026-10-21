//! Comparison operations.

mod cmp_r32_rm32;
mod cmp_rm32_imm32;
mod cmp_rm32_imm8;

pub use cmp_r32_rm32::CmpR32Rm32;
pub use cmp_rm32_imm8::CmpRm32Imm8;
pub use cmp_rm32_imm32::CmpRm32Imm32;
