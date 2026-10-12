/// The scale factor.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Scale {
    /// Scale by one.
    One = 0,

    /// Scale by two.
    Two = 1,

    /// Scale by four.
    Four = 2,

    /// Scale by eight.
    Eight = 3,
}

impl Scale {
    /// Constructs a scale from the SIB scale value.
    pub(crate) const fn from_index(index: u8) -> Self {
        match index {
            0 => Self::One,
            1 => Self::Two,
            2 => Self::Four,
            3 => Self::Eight,
            _ => panic!("invalid SIB scale"),
        }
    }
}

impl Scale {
    /// Gets the numeric scale factor.
    pub const fn value(self) -> u8 {
        1 << self as u8
    }
}
