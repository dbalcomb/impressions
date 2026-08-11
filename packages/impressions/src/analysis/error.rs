/// The binary analysis error.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// A problem was encountered with the image.
    #[error("Image error")]
    Image(#[from] super::image::Error),
}
