//! Subtraction operations.

mod sub_r32_rm32;
mod sub_rm32_imm32;
mod sub_rm32_imm8;

pub use sub_r32_rm32::SubR32Rm32;
pub use sub_rm32_imm8::SubRm32Imm8;
pub use sub_rm32_imm32::SubRm32Imm32;
