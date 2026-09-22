use std::fmt::Display;

use crate::memory::region::ops::encode::Encode;

/// An inspection value with a data type representation.
pub trait InspectionValue: Encode + Display {
    /// Gets the data type representation.
    fn data_type(&self) -> &dyn Display;
}

impl InspectionValue for &str {
    fn data_type(&self) -> &dyn Display {
        &"string"
    }
}

macro_rules! impl_primitives {
    ($($type:ty),+ $(,)?) => {
        $(
            impl InspectionValue for $type {
                fn data_type(&self) -> &dyn Display {
                    &stringify!($type)
                }
            }
        )+
    };
}

impl_primitives!(bool, u8, u16, u32);
