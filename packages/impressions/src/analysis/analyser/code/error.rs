/// The code analyser error.
#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum Error {
    /// A problem was encountered with the cursor.
    #[error("cursor error")]
    Cursor(#[from] crate::memory::cursor::Error),

    /// A problem was encountered while decoding.
    #[error("decode error")]
    Decode(
        #[from]
        crate::memory::cursor::ops::read::Error<crate::image::region::section::block::code::Error>,
    ),

    /// A problem was encountered with the image.
    #[error("image error")]
    Image(#[from] crate::image::Error),
}
