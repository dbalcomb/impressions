//! Binary analysers.

pub mod code;
pub mod imports;

use crate::image::Image;

/// Defines the ability to analyse a binary image.
pub trait Analyser {
    /// The associated error type.
    type Error;

    /// Analyse the given image.
    fn analyse(&self, image: &mut Image) -> Result<(), Self::Error>;
}
