/// The image file section error.
#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum Error {
    /// A problem was encountered parsing the image.
    #[error("parse error")]
    Parse(#[from] bytes::TryGetError),

    /// A problem was encountered with an unidentified region.
    #[error("unidentified region error")]
    Unidentified(#[from] crate::memory::regions::unidentified::Error),

    /// An invalid size was specified for the section.
    #[error("invalid size")]
    Size(#[from] crate::memory::extent::Error),
}
