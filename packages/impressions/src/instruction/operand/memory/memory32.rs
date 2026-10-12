use std::fmt::{self, Display};

use crate::instruction::operand::displacement::Displacement32;
use crate::instruction::operand::register::Register32;

use super::ScaledIndex;

/// A memory operand with base, scaled index, and displacement.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Memory32 {
    base: Option<Register32>,
    index: Option<ScaledIndex>,
    displacement: Displacement32,
}

impl Memory32 {
    /// Constructs a new memory operand, returning `None` for unsupported forms.
    pub(in crate::instruction) const fn new(
        base: Option<Register32>,
        index: Option<ScaledIndex>,
        displacement: Displacement32,
    ) -> Option<Self> {
        if let Some(index) = index
            && matches!(index.register(), Register32::Esp)
        {
            return None;
        }

        Some(Self {
            base,
            index,
            displacement,
        })
    }
}

impl Memory32 {
    /// Gets the base register.
    pub const fn base(&self) -> Option<Register32> {
        self.base
    }

    /// Gets the index register and scale.
    pub const fn index(&self) -> Option<ScaledIndex> {
        self.index
    }

    /// Gets the address offset, sign-extended for byte displacements.
    pub const fn displacement(&self) -> Displacement32 {
        self.displacement
    }
}

impl Display for Memory32 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let offset = self.displacement.value();

        if self.base.is_none() && self.index.is_none() {
            return write!(f, "[{:#010x}]", offset as u32);
        }

        f.write_str("[")?;

        if let Some(base) = self.base {
            Display::fmt(&base, f)?;
        }

        if let Some(index) = self.index {
            if self.base.is_some() {
                f.write_str("+")?;
            }

            Display::fmt(&index, f)?;
        }

        if offset > 0 {
            write!(f, "+{offset}")?;
        } else if offset < 0 {
            write!(f, "{offset}")?;
        }

        f.write_str("]")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::instruction::operand::memory::Scale;

    #[test]
    fn rejects_esp_index_at_every_scale() {
        for scale in [Scale::One, Scale::Two, Scale::Four, Scale::Eight] {
            assert_eq!(
                Memory32::new(
                    Some(Register32::Ebx),
                    Some(ScaledIndex::new(Register32::Esp, scale)),
                    Displacement32::default(),
                ),
                None,
            );
        }
    }

    #[test]
    fn accepts_other_indices_and_esp_base() {
        for index in 0..8 {
            let register = Register32::from_index(index);

            if register == Register32::Esp {
                continue;
            }

            let memory = Memory32::new(
                Some(Register32::Esp),
                Some(ScaledIndex::new(register, Scale::Four)),
                Displacement32::new(-16),
            )
            .unwrap();

            assert_eq!(memory.base(), Some(Register32::Esp));
            assert_eq!(
                memory.index(),
                Some(ScaledIndex::new(register, Scale::Four))
            );
            assert_eq!(memory.displacement(), Displacement32::new(-16));
        }

        assert!(Memory32::new(None, None, Displacement32::new(16)).is_some());
    }
}
