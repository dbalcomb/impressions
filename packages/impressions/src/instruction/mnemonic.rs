use std::fmt::{self, Display};

/// An x86 instruction mnemonic.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Mnemonic {
    /// Push onto the stack.
    Push,
}

impl Display for Mnemonic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Push => write!(f, "push"),
        }
    }
}
