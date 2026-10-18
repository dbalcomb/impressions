//! Stack push operations.

mod push_imm32;
mod push_imm8;
mod push_r32;

pub use push_imm8::PushImm8;
pub use push_imm32::PushImm32;
pub use push_r32::PushR32;
