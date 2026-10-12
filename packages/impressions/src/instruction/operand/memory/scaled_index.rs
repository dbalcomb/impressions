use std::fmt::{self, Display};

use crate::instruction::operand::register::Register32;

use super::Scale;

/// An address index register multiplied by a scale factor.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ScaledIndex {
    register: Register32,
    scale: Scale,
}

impl ScaledIndex {
    /// Constructs a scaled index.
    pub const fn new(register: Register32, scale: Scale) -> Self {
        Self { register, scale }
    }

    /// Gets the index register.
    pub const fn register(self) -> Register32 {
        self.register
    }

    /// Gets the scale.
    pub const fn scale(self) -> Scale {
        self.scale
    }
}

impl Display for ScaledIndex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}*{}", self.register, self.scale.value())
    }
}
