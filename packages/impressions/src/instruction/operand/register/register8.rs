use std::fmt::{self, Display};

/// An 8-bit byte register.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum Register8 {
    /// The accumulator low register.
    Al = 0,

    /// The counter low register.
    Cl = 1,

    /// The data low register.
    Dl = 2,

    /// The base low register.
    Bl = 3,

    /// The accumulator high register.
    Ah = 4,

    /// The counter high register.
    Ch = 5,

    /// The data high register.
    Dh = 6,

    /// The base high register.
    Bh = 7,
}

impl Register8 {
    /// Constructs an 8-bit register from its index.
    ///
    /// # Panics
    ///
    /// Panics if the index is not in the range 0..8.
    pub const fn from_index(index: u8) -> Self {
        match index {
            0 => Self::Al,
            1 => Self::Cl,
            2 => Self::Dl,
            3 => Self::Bl,
            4 => Self::Ah,
            5 => Self::Ch,
            6 => Self::Dh,
            7 => Self::Bh,
            _ => panic!("invalid register index"),
        }
    }

    /// Returns the index of the 8-bit register.
    pub const fn index(self) -> u8 {
        self as u8
    }
}

impl Display for Register8 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Al => "al",
            Self::Cl => "cl",
            Self::Dl => "dl",
            Self::Bl => "bl",
            Self::Ah => "ah",
            Self::Ch => "ch",
            Self::Dh => "dh",
            Self::Bh => "bh",
        })
    }
}

#[cfg(test)]
mod tests {
    use super::Register8;

    #[test]
    fn supports_all_registers() {
        for (index, register, display) in [
            (0, Register8::Al, "al"),
            (1, Register8::Cl, "cl"),
            (2, Register8::Dl, "dl"),
            (3, Register8::Bl, "bl"),
            (4, Register8::Ah, "ah"),
            (5, Register8::Ch, "ch"),
            (6, Register8::Dh, "dh"),
            (7, Register8::Bh, "bh"),
        ] {
            assert_eq!(Register8::from_index(index), register);
            assert_eq!(register.index(), index);
            assert_eq!(register.to_string(), display);
        }
    }
}
