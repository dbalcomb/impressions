//! Parse data from buffers.

mod error;

use array_init::try_array_init;

pub use bytes::{Buf, TryGetError};

pub use self::error::ArrayParseError;

/// Defines the ability to parse a data structure from a buffer.
pub trait Parse: Sized {
    /// The associated context.
    type Context<'a>;

    /// The associated parse error.
    type Error;

    /// Parses the data from the given buffer with the provided context.
    fn parse_with(buffer: impl Buf, context: Self::Context<'_>) -> Result<Self, Self::Error>;

    /// Parses the data from the given buffer.
    fn parse(buffer: impl Buf) -> Result<Self, Self::Error>
    where
        Self: for<'a> Parse<Context<'a> = ()>,
    {
        Self::parse_with(buffer, ())
    }
}

impl Parse for u8 {
    type Context<'a> = ();
    type Error = TryGetError;

    fn parse_with(mut buffer: impl Buf, _: Self::Context<'_>) -> Result<Self, Self::Error> {
        buffer.try_get_u8()
    }
}

impl<T, const N: usize> Parse for [T; N]
where
    T: for<'a> Parse<Context<'a>: Copy>,
{
    type Context<'a> = T::Context<'a>;
    type Error = ArrayParseError<T::Error, N>;

    fn parse_with(mut buffer: impl Buf, context: Self::Context<'_>) -> Result<Self, Self::Error> {
        try_array_init(|index| {
            T::parse_with(&mut buffer, context).map_err(|error| ArrayParseError { error, index })
        })
    }
}

#[cfg(test)]
mod tests {
    use bytes::TryGetError;

    use super::{ArrayParseError, Parse};

    #[test]
    fn test_parse_byte_array() {
        let mut buffer: &[u8] = &[0, 1, 2, 3, 4, 5, 6, 7, 8];

        let arr4 = <[u8; 4]>::parse(&mut buffer).unwrap();
        let arr3 = <[u8; 3]>::parse(&mut buffer).unwrap();

        assert_eq!(arr4, [0, 1, 2, 3]);
        assert_eq!(arr3, [4, 5, 6]);
        assert_eq!(buffer, [7, 8]);

        <[u8; 0]>::parse(&mut buffer).unwrap();

        assert_eq!(buffer, [7, 8]);
        assert_eq!(
            <[u8; 3]>::parse(&mut buffer),
            Err(ArrayParseError {
                index: 2,
                error: TryGetError {
                    requested: 1,
                    available: 0,
                }
            })
        );
        assert!(buffer.is_empty());
    }

    #[test]
    fn test_parse_byte_array_array() {
        let mut buffer: &[u8] = &[0, 1, 2, 3, 4, 5, 6, 7, 8];

        let arr = <[[u8; 4]; 2]>::parse(&mut buffer).unwrap();

        assert_eq!(arr, [[0, 1, 2, 3], [4, 5, 6, 7]]);
        assert_eq!(buffer, [8]);
        assert_eq!(
            <[[u8; 3]; 2]>::parse(&mut buffer),
            Err(ArrayParseError {
                index: 0,
                error: ArrayParseError {
                    index: 1,
                    error: TryGetError {
                        requested: 1,
                        available: 0,
                    },
                }
            })
        );
        assert!(buffer.is_empty());
    }

    #[test]
    fn test_parse_array_error_into() {
        let err = <[u8; 4]>::parse([].as_slice()).unwrap_err();

        assert_eq!(
            err,
            ArrayParseError {
                index: 0,
                error: TryGetError {
                    requested: 1,
                    available: 0,
                }
            },
        );
        assert_eq!(
            err.into_buffer_error(),
            TryGetError {
                requested: 4,
                available: 0,
            }
        );

        let err = <[u8; 4]>::parse([1].as_slice()).unwrap_err();

        assert_eq!(
            err,
            ArrayParseError {
                index: 1,
                error: TryGetError {
                    requested: 1,
                    available: 0,
                }
            },
        );
        assert_eq!(
            err.into_buffer_error(),
            TryGetError {
                requested: 4,
                available: 1,
            }
        );

        let err = <[u8; 4]>::parse([1, 2].as_slice()).unwrap_err();

        assert_eq!(
            err,
            ArrayParseError {
                index: 2,
                error: TryGetError {
                    requested: 1,
                    available: 0,
                }
            },
        );
        assert_eq!(
            err.into_buffer_error(),
            TryGetError {
                requested: 4,
                available: 2,
            }
        );

        let err = <[u8; 4]>::parse([1, 2, 3].as_slice()).unwrap_err();

        assert_eq!(
            err,
            ArrayParseError {
                index: 3,
                error: TryGetError {
                    requested: 1,
                    available: 0,
                }
            },
        );
        assert_eq!(
            err.into_buffer_error(),
            TryGetError {
                requested: 4,
                available: 3,
            }
        );
    }
}
