/// The binary analysis error.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// A problem was encountered with the image.
    #[error("Image error")]
    Image(#[from] super::image::Error),

    /// A problem was encountered reading the analysis.
    #[error("I/O error")]
    Io(#[from] std::io::Error),

    /// A problem was encountered encoding the analysis.
    #[error("Encode error")]
    Encode(#[from] rmp_serde::encode::Error),

    /// A problem was encountered decoding the analysis.
    #[error("Decode error")]
    Decode(#[from] rmp_serde::decode::Error),
}
