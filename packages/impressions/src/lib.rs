//! An experimental binary analysis utility inspired by the [City Building][1]
//! series from [Impressions Games][2].
//!
//! This library supports analysing 32-bit Portable Executable (PE) image files
//! and is not a general purpose utility. It does not aim to support anything
//! other than a very small set of Windows games from a particular time period
//! and will make assumptions accordingly.
//!
//! [1]: https://en.wikipedia.org/wiki/City_Building_(series)
//! [2]: https://en.wikipedia.org/wiki/Impressions_Games

pub mod analysis;
pub mod data;
