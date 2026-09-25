//! The code analyser.

mod error;

use crate::image::Image;
use crate::image::region::section::block::Block;
use crate::image::region::section::block::code::Code as CodeBlock;
use crate::memory::address::Address;
use crate::memory::cursor::AsCursor;
use crate::memory::cursor::ops::read::Read;
use crate::memory::region::ops::insert::Insert;

use super::Analyser;

pub use self::error::Error;

/// The code analyser.
#[derive(Clone, Copy, Debug)]
pub struct Code(Address);

impl Code {
    /// Creates a new code analyser for the given address.
    pub const fn new(address: Address) -> Self {
        Self(address)
    }
}

impl Analyser for Code {
    type Error = Error;

    fn analyse(&self, image: &mut Image) -> Result<(), Self::Error> {
        let mut cursor = image.cursor();

        cursor.seek_address(self.0)?;

        let code = cursor.read::<CodeBlock>()?;

        image.insert(self.0, Block::Code(code))?;

        Ok(())
    }
}
