/// The inspection error.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum Error {
    /// Indicates that there was a problem writing to the writer.
    #[error("write error")]
    Write(#[from] std::fmt::Error),
}
