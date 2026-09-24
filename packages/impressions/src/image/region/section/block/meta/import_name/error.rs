/// An imported DLL name error.
#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum Error {
    /// The DLL name or its alignment padding could not be read.
    #[error("invalid name")]
    Name(
        #[from]
        crate::memory::region::types::aligned::Error<
            crate::memory::region::types::null_string::Error,
        >,
    ),
}
