use bytes::Buf;
use serde::{Deserialize, Serialize};

use crate::data::parse::Parse;
use crate::image::Error;
use crate::memory::Extent;

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

impl Extent for DosHeader {
    fn size(&self) -> u64 {
        64
    }
}

impl Parse for DosHeader {
    type Context<'a> = ();
    type Error = Error;

    fn parse_with(mut buffer: impl Buf, _: Self::Context<'_>) -> Result<Self, Self::Error> {
        let e_magic = buffer.try_get_u16_le()?;

        if e_magic != DOS_SIGNATURE {
            return Err(Error::InvalidSignature);
        }

        Ok(Self {
            e_magic,
            e_cblp: buffer.try_get_u16_le()?,
            e_cp: buffer.try_get_u16_le()?,
            e_crlc: buffer.try_get_u16_le()?,
            e_cparhdr: buffer.try_get_u16_le()?,
            e_minalloc: buffer.try_get_u16_le()?,
            e_maxalloc: buffer.try_get_u16_le()?,
            e_ss: buffer.try_get_u16_le()?,
            e_sp: buffer.try_get_u16_le()?,
            e_csum: buffer.try_get_u16_le()?,
            e_ip: buffer.try_get_u16_le()?,
            e_cs: buffer.try_get_u16_le()?,
            e_lfarlc: buffer.try_get_u16_le()?,
            e_ovno: buffer.try_get_u16_le()?,
            e_res: array_init::try_array_init(|_| buffer.try_get_u16_le())?,
            e_oemid: buffer.try_get_u16_le()?,
            e_oeminfo: buffer.try_get_u16_le()?,
            e_res2: array_init::try_array_init(|_| buffer.try_get_u16_le())?,
            e_lfanew: buffer.try_get_u32_le()?,
        })
    }
}
