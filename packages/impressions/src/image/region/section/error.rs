/// The image file section error.
#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum Error {
    /// An empty section was detected.
    #[error("empty section")]
    EmptySection,

    /// A problem was encountered parsing the image.
    #[error("parse error")]
    Parse(#[from] bytes::TryGetError),

    /// A problem was encountered with an unidentified region.
    #[error("unidentified region error")]
    Unidentified(#[from] crate::memory::regions::unidentified::Error),
}
