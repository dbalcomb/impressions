use crate::memory::region::Region;

/// Defines the ability to calculate the completion percentage of a region.
///
/// The completion percentage for a region is calculated using the number of
/// bytes identified out of the total size. Implementors need only to provide
/// the number of identified bytes and a correct region size.
pub trait Completion: Region {
    /// Gets the number of identified bytes.
    fn identified(&self) -> u64;

    /// Gets the analysis completion percentage of the region.
    fn completion(&self) -> f64 {
        self.identified() as f64 / self.size() as f64 * 100.0
    }
}
