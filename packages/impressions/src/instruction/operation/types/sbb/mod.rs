//! Subtract-with-borrow operations.

mod sbb_rm32_imm32;
mod sbb_rm32_imm8;

pub use sbb_rm32_imm8::SbbRm32Imm8;
pub use sbb_rm32_imm32::SbbRm32Imm32;
