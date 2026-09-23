/// The image file section error.
#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum Error {
    /// A problem was encountered decoding the image.
    #[error("decode error")]
    Decode(#[from] crate::memory::region::ops::decode::Error),

    /// A problem was encountered with an unidentified region.
    #[error("unidentified region error")]
    Unidentified(#[from] crate::memory::region::types::unidentified::Error),

    /// An invalid size was specified for the section.
    #[error("invalid size")]
    Size(#[from] crate::memory::extent::Error),

    /// A problem was encountered with a contiguous region.
    #[error("contiguous region error")]
    Contiguous(#[from] crate::memory::region::types::contiguous::Error),

    /// A problem was encountered with a block region.
    #[error("block error")]
    Block(#[from] super::block::Error),
}
