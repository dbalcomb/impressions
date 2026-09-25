/// A code block error.
#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum Error {
    /// The code block is empty.
    #[error("code block is empty")]
    Empty,

    /// An instruction error.
    #[error("instruction error")]
    Instruction(#[from] crate::instruction::Error),
}
