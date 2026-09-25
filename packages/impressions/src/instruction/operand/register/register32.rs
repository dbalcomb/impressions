use std::fmt::{self, Display};

use serde::{Deserialize, Serialize};

/// A 32-bit `DWord` register.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum Register32 {
    /// The extended accumulator register.
    Eax = 0,

    /// The extended counter register.
    Ecx = 1,

    /// The extended data register.
    Edx = 2,

    /// The extended base register.
    Ebx = 3,

    /// The extended stack pointer register.
    Esp = 4,

    /// The extended base pointer register.
    Ebp = 5,

    /// The extended source index register.
    Esi = 6,

    /// The extended destination index register.
    Edi = 7,
}

impl Register32 {
    /// Constructs a 32-bit register from its index.
    ///
    /// # Panics
    ///
    /// Panics if the index is not in the range 0..8.
    pub const fn from_index(index: u8) -> Self {
        match index {
            0 => Self::Eax,
            1 => Self::Ecx,
            2 => Self::Edx,
            3 => Self::Ebx,
            4 => Self::Esp,
            5 => Self::Ebp,
            6 => Self::Esi,
            7 => Self::Edi,
            _ => panic!("invalid register index"),
        }
    }

    /// Returns the index of the 32-bit register.
    pub const fn index(self) -> u8 {
        self as u8
    }
}

impl Display for Register32 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Eax => "eax",
            Self::Ecx => "ecx",
            Self::Edx => "edx",
            Self::Ebx => "ebx",
            Self::Esp => "esp",
            Self::Ebp => "ebp",
            Self::Esi => "esi",
            Self::Edi => "edi",
        })
    }
}
