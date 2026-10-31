/// The three-bit ModR/M `reg` field, interpreted as a register or opcode
/// extension.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum RegField {
    Zero = 0,
    One = 1,
    Two = 2,
    Three = 3,
    Four = 4,
    Five = 5,
    Six = 6,
    Seven = 7,
}

impl RegField {
    /// Constructs a field from the low three bits, ignoring the remaining bits.
    pub const fn from_bits(bits: u8) -> Self {
        match bits & 7 {
            0 => Self::Zero,
            1 => Self::One,
            2 => Self::Two,
            3 => Self::Three,
            4 => Self::Four,
            5 => Self::Five,
            6 => Self::Six,
            _ => Self::Seven,
        }
    }

    /// Gets the three encoded bits.
    pub const fn bits(self) -> u8 {
        self as u8
    }
}

#[cfg(test)]
mod tests {
    use super::RegField;

    #[test]
    fn interprets_only_the_low_three_bits() {
        let fields = [
            RegField::Zero,
            RegField::One,
            RegField::Two,
            RegField::Three,
            RegField::Four,
            RegField::Five,
            RegField::Six,
            RegField::Seven,
        ];

        for bits in 0..=u8::MAX {
            let field = RegField::from_bits(bits);

            assert_eq!(field, fields[usize::from(bits & 7)]);
            assert_eq!(field.bits(), bits & 7);
        }
    }
}
