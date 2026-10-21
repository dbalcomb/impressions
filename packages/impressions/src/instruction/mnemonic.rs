use std::fmt::{self, Display};

/// An x86 instruction mnemonic.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Mnemonic {
    /// Add with carry.
    Adc,

    /// Add.
    Add,

    /// Bitwise AND.
    And,

    /// Call a procedure.
    Call,

    /// Compare.
    Cmp,

    /// Move.
    Mov,

    /// Bitwise OR.
    Or,

    /// Push onto the stack.
    Push,

    /// Rotate left through carry.
    Rcl,

    /// Rotate right through carry.
    Rcr,

    /// Rotate left.
    Rol,

    /// Rotate right.
    Ror,

    /// Shift right arithmetically.
    Sar,

    /// Subtract with borrow.
    Sbb,

    /// Shift left logically.
    Shl,

    /// Shift right logically.
    Shr,

    /// Subtract.
    Sub,

    /// Bitwise exclusive OR.
    Xor,
}

impl Display for Mnemonic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Adc => write!(f, "adc"),
            Self::Add => write!(f, "add"),
            Self::And => write!(f, "and"),
            Self::Call => write!(f, "call"),
            Self::Cmp => write!(f, "cmp"),
            Self::Mov => write!(f, "mov"),
            Self::Or => write!(f, "or"),
            Self::Push => write!(f, "push"),
            Self::Rcl => write!(f, "rcl"),
            Self::Rcr => write!(f, "rcr"),
            Self::Rol => write!(f, "rol"),
            Self::Ror => write!(f, "ror"),
            Self::Sar => write!(f, "sar"),
            Self::Sbb => write!(f, "sbb"),
            Self::Shl => write!(f, "shl"),
            Self::Shr => write!(f, "shr"),
            Self::Sub => write!(f, "sub"),
            Self::Xor => write!(f, "xor"),
        }
    }
}
