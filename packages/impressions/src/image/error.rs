/// The image file error.
#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum Error {
    /// A problem was encountered with the image headers.
    #[error("headers region error")]
    Headers(#[from] crate::image::region::headers::Error),

    /// A problem was encountered with an image section.
    #[error("section region error")]
    Section(#[from] crate::image::region::section::Error),

    /// A problem was encountered with a sparse region.
    #[error("sparse region error")]
    Sparse(#[from] crate::memory::regions::sparse::Error),
}
