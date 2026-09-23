//! The DOS header of an image file.

mod cursor;
mod field;

use serde::{Deserialize, Serialize};

use crate::image::region::headers::Error;
use crate::memory::cursor::AsCursor;
use crate::memory::extent::{Extent, FixedExtent, Size};
use crate::memory::inspect::{Inspect, Inspector};
use crate::memory::region::ops::decode::{Decode, Decoder};
use crate::memory::region::ops::encode::{self, Encode};

pub use self::cursor::DosHeaderCursor;
pub use self::field::Field;

/// The signature indicating the start of the DOS headers.
const DOS_SIGNATURE: u16 = 0x5A4D;

/// The image file DOS header.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DosHeader {
    /// The signature that identifies the file as a DOS-compatible executable.
    e_magic: u16,

    /// The number of bytes in the last page of the file.
    e_cblp: u16,

    /// The number of pages in the file.
    e_cp: u16,

    /// The number of relocation entries in the file.
    e_crlc: u16,

    /// The size of the header in 16-byte paragraphs.
    e_cparhdr: u16,

    /// The minimum number of paragraphs allocated to the program.
    e_minalloc: u16,

    /// The maximum number of paragraphs allocated to the program.
    e_maxalloc: u16,

    /// The initial stack segment value.
    e_ss: u16,

    /// The initial stack pointer value.
    e_sp: u16,

    /// The checksum.
    e_csum: u16,

    /// The initial instruction pointer.
    e_ip: u16,

    /// The initial code segment.
    e_cs: u16,

    /// The file address of the relocation table.
    e_lfarlc: u16,

    /// The overlay number.
    e_ovno: u16,

    /// An array reserved for future use.
    e_res: [u16; 4],

    /// The OEM identifier.
    e_oemid: u16,

    /// The OEM information.
    e_oeminfo: u16,

    /// An array reserved for future use.
    e_res2: [u16; 10],

    /// The file offset of the PE headers.
    e_lfanew: u32,
}

impl DosHeader {
    /// Gets the file offset of the PE headers.
    pub fn pe_headers_offset(&self) -> u32 {
        self.e_lfanew
    }
}

impl FixedExtent for DosHeader {
    const SIZE: Size = Size::new_valid(64);
}

impl Inspect for DosHeader {
    fn inspect(&self, inspector: &mut dyn Inspector) {
        inspector
            .record(self.address_space())
            .label(&"DOS Header")
            .finish()
    }
}

impl Encode for DosHeader {
    fn encode(&self, encoder: &mut dyn encode::Encoder) -> Result<(), encode::Error> {
        encoder.write_u16_le(self.e_magic)?;
        encoder.write_u16_le(self.e_cblp)?;
        encoder.write_u16_le(self.e_cp)?;
        encoder.write_u16_le(self.e_crlc)?;
        encoder.write_u16_le(self.e_cparhdr)?;
        encoder.write_u16_le(self.e_minalloc)?;
        encoder.write_u16_le(self.e_maxalloc)?;
        encoder.write_u16_le(self.e_ss)?;
        encoder.write_u16_le(self.e_sp)?;
        encoder.write_u16_le(self.e_csum)?;
        encoder.write_u16_le(self.e_ip)?;
        encoder.write_u16_le(self.e_cs)?;
        encoder.write_u16_le(self.e_lfarlc)?;
        encoder.write_u16_le(self.e_ovno)?;

        for res in &self.e_res {
            encoder.write_u16_le(*res)?;
        }

        encoder.write_u16_le(self.e_oemid)?;
        encoder.write_u16_le(self.e_oeminfo)?;

        for res2 in &self.e_res2 {
            encoder.write_u16_le(*res2)?;
        }

        encoder.write_u32_le(self.e_lfanew)?;

        Ok(())
    }
}

impl Decode for DosHeader {
    type Context<'a> = ();
    type Error = Error;

    fn decode_with(decoder: &mut dyn Decoder, _: Self::Context<'_>) -> Result<Self, Self::Error> {
        let e_magic = decoder.read_u16_le()?;

        if e_magic != DOS_SIGNATURE {
            return Err(Error::InvalidSignature);
        }

        Ok(Self {
            e_magic,
            e_cblp: decoder.read_u16_le()?,
            e_cp: decoder.read_u16_le()?,
            e_crlc: decoder.read_u16_le()?,
            e_cparhdr: decoder.read_u16_le()?,
            e_minalloc: decoder.read_u16_le()?,
            e_maxalloc: decoder.read_u16_le()?,
            e_ss: decoder.read_u16_le()?,
            e_sp: decoder.read_u16_le()?,
            e_csum: decoder.read_u16_le()?,
            e_ip: decoder.read_u16_le()?,
            e_cs: decoder.read_u16_le()?,
            e_lfarlc: decoder.read_u16_le()?,
            e_ovno: decoder.read_u16_le()?,
            e_res: array_init::try_array_init(|_| decoder.read_u16_le())?,
            e_oemid: decoder.read_u16_le()?,
            e_oeminfo: decoder.read_u16_le()?,
            e_res2: array_init::try_array_init(|_| decoder.read_u16_le())?,
            e_lfanew: decoder.read_u32_le()?,
        })
    }
}

impl AsCursor for DosHeader {
    type Cursor<'a> = DosHeaderCursor<'a>;

    fn cursor(&self) -> Self::Cursor<'_> {
        DosHeaderCursor::new(self)
    }
}
