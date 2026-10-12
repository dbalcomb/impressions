//! Displacement operands.

mod displacement32;
mod displacement8;

pub use self::displacement8::Displacement8;
pub use self::displacement32::Displacement32;

/// A signed displacement in one of several sizes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Displacement {
    /// A signed byte displacement.
    Byte(Displacement8),

    /// A signed dword displacement.
    DWord(Displacement32),
}

impl Displacement {
    /// Gets the 8-bit byte displacement.
    pub const fn displacement8(self) -> Option<Displacement8> {
        match self {
            Self::Byte(value) => Some(value),
            Self::DWord(_) => None,
        }
    }

    /// Gets the 32-bit dword displacement.
    pub const fn displacement32(self) -> Option<Displacement32> {
        match self {
            Self::Byte(_) => None,
            Self::DWord(value) => Some(value),
        }
    }
}

impl Displacement {
    /// Gets the displacement size.
    pub const fn size(self) -> DisplacementSize {
        match self {
            Self::Byte(_) => DisplacementSize::Byte,
            Self::DWord(_) => DisplacementSize::DWord,
        }
    }
}

/// The displacement size.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DisplacementSize {
    /// A byte displacement.
    Byte,

    /// A dword displacement.
    DWord,
}
