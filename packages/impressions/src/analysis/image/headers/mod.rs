//! The image file headers analysis.

mod coff;
mod dos;
mod optional;
mod section;

use bytes::{Buf, TryGetError};

pub use self::coff::CoffHeader;
pub use self::dos::DosHeader;
pub use self::optional::{DataDirectory, OptionalHeader};
pub use self::section::SectionHeader;

use super::Error;

/// The signature indicating the start of the PE headers.
const PE_SIGNATURE: u32 = 0x4550;

/// The 32-bit Portable Executable (PE) image file headers.
///
/// # Structure
///
/// This region consists of the following components:
/// * DOS Header (included)
/// * DOS Stub (skipped)
/// * Rich Header (skipped)
/// * PE Signature (checked)
/// * COFF Header (included)
/// * Optional Header (included)
/// * Section Headers (included)
///
/// The DOS Stub and Rich Header are not useful for this analysis and have been
/// skipped. The former describes an application that runs under MS-DOS and the
/// latter includes compiler toolchain information.
///
/// The Optional header is only optional in object files and is always included
/// in image files so it is required here.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Headers {
    dos: DosHeader,
    coff: CoffHeader,
    optional: OptionalHeader,
    sections: Vec<SectionHeader>,
}

impl Headers {
    /// Parses the image file headers up to the section table.
    pub fn parse(mut buffer: impl Buf) -> Result<Self, Error> {
        let dos = DosHeader::parse(&mut buffer)?;
        let pe_offset = dos.pe_headers_offset() as usize - DosHeader::SIZE;

        if pe_offset > buffer.remaining() {
            return Err(Error::Parse(TryGetError {
                requested: pe_offset,
                available: buffer.remaining(),
            }));
        }

        buffer.advance(pe_offset);

        let signature = buffer.try_get_u32_le()?;

        if signature != PE_SIGNATURE {
            return Err(Error::InvalidSignature);
        }

        let coff = CoffHeader::parse(&mut buffer)?;
        let optional = OptionalHeader::parse(&mut buffer)?;
        let sections = (0..coff.number_of_sections())
            .map(|_| SectionHeader::parse(&mut buffer))
            .collect::<Result<_, _>>()?;

        let file_offset = dos.pe_headers_offset() as usize
            + 4
            + CoffHeader::SIZE
            + OptionalHeader::BASE_SIZE
            + (DataDirectory::SIZE * optional.number_of_data_directories())
            + (SectionHeader::SIZE * coff.number_of_sections());

        let remaining = buffer
            .remaining()
            .min(optional.headers_size() as usize - file_offset);

        buffer.advance(remaining);

        Ok(Self {
            dos,
            coff,
            optional,
            sections,
        })
    }
}

impl Headers {
    /// Gets the DOS header.
    pub fn dos(&self) -> &DosHeader {
        &self.dos
    }

    /// Gets the COFF header.
    pub fn coff(&self) -> &CoffHeader {
        &self.coff
    }

    /// Gets the Optional header.
    pub fn optional(&self) -> &OptionalHeader {
        &self.optional
    }

    /// Gets an iterator over the section headers.
    pub fn sections(&self) -> impl Iterator<Item = &SectionHeader> {
        self.sections.iter()
    }
}

impl Headers {
    /// Gets the address of the headers.
    pub fn address(&self) -> u32 {
        self.optional.image_address()
    }

    /// Gets the size of the headers.
    pub fn size(&self) -> u64 {
        self.optional.headers_size()
    }
}

#[cfg(test)]
mod tests {
    use bytes::{Buf, Bytes, BytesMut};

    use super::Headers;

    static SAMPLE_HEADERS_SIZE: usize = 4096;
    static SAMPLE_HEADERS_DATA: [u8; 696] = [
        77, 90, 144, 0, 3, 0, 0, 0, 4, 0, 0, 0, 255, 255, 0, 0, 184, 0, 0, 0, 0, 0, 0, 0, 64, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 32, 1, 0, 0, 14, 31, 186, 14, 0, 180, 9, 205, 33, 184, 1, 76, 205, 33, 84, 104,
        105, 115, 32, 112, 114, 111, 103, 114, 97, 109, 32, 99, 97, 110, 110, 111, 116, 32, 98,
        101, 32, 114, 117, 110, 32, 105, 110, 32, 68, 79, 83, 32, 109, 111, 100, 101, 32, 13, 13,
        10, 36, 0, 0, 0, 0, 0, 0, 0, 253, 10, 68, 56, 185, 107, 42, 107, 185, 107, 42, 107, 185,
        107, 42, 107, 194, 119, 38, 107, 183, 107, 42, 107, 214, 116, 33, 107, 178, 107, 42, 107,
        58, 119, 36, 107, 161, 107, 42, 107, 214, 116, 32, 107, 194, 107, 42, 107, 62, 119, 40,
        107, 191, 107, 42, 107, 185, 107, 42, 107, 191, 107, 42, 107, 219, 116, 57, 107, 183, 107,
        42, 107, 185, 107, 43, 107, 44, 107, 42, 107, 230, 73, 33, 107, 184, 107, 42, 107, 230, 73,
        32, 107, 184, 107, 42, 107, 70, 75, 46, 107, 254, 107, 42, 107, 81, 116, 33, 107, 182, 107,
        42, 107, 81, 116, 32, 107, 177, 107, 42, 107, 191, 72, 33, 107, 201, 107, 42, 107, 126,
        109, 44, 107, 184, 107, 42, 107, 82, 105, 99, 104, 185, 107, 42, 107, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 80, 69, 0, 0, 76, 1, 4, 0, 251, 139, 53, 60, 0, 0, 0, 0, 0, 0,
        0, 0, 224, 0, 15, 1, 11, 1, 6, 0, 0, 112, 29, 0, 0, 16, 9, 0, 0, 0, 0, 0, 32, 214, 28, 0,
        0, 16, 0, 0, 0, 128, 29, 0, 0, 0, 64, 0, 0, 16, 0, 0, 0, 16, 0, 0, 4, 0, 0, 0, 0, 0, 0, 0,
        4, 0, 0, 0, 0, 0, 0, 0, 0, 16, 33, 1, 0, 16, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 0, 0, 16, 0, 0,
        16, 0, 0, 0, 0, 16, 0, 0, 16, 0, 0, 0, 0, 0, 0, 16, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 144,
        95, 30, 0, 220, 0, 0, 0, 0, 224, 32, 1, 208, 40, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 128, 29, 0, 80, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 46, 116, 101, 120, 116, 0, 0, 0, 81, 99, 29, 0, 0, 16, 0, 0, 0, 112, 29, 0, 0, 16,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 32, 0, 0, 96, 46, 114, 100, 97, 116, 97, 0, 0,
        192, 243, 0, 0, 0, 128, 29, 0, 0, 0, 1, 0, 0, 128, 29, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 64, 0, 0, 64, 46, 100, 97, 116, 97, 0, 0, 0, 8, 86, 2, 1, 0, 128, 30, 0, 0, 224, 7, 0,
        0, 128, 30, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 64, 0, 0, 192, 46, 114, 115, 114, 99, 0,
        0, 0, 208, 40, 0, 0, 0, 224, 32, 1, 0, 48, 0, 0, 0, 96, 38, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 64, 0, 0, 64,
    ];

    fn sample_headers_padded() -> Bytes {
        const SECTIONS_SIZE: usize = 4;

        let mut bytes = BytesMut::zeroed(SAMPLE_HEADERS_SIZE + SECTIONS_SIZE);

        bytes[0..SAMPLE_HEADERS_DATA.len()].copy_from_slice(&SAMPLE_HEADERS_DATA);
        bytes[SAMPLE_HEADERS_SIZE..].copy_from_slice(&[1, 2, 3, 4]);
        bytes.freeze()
    }

    #[test]
    fn test_parse_bytes() {
        let mut buffer = sample_headers_padded();
        let headers = Headers::parse(&mut buffer).unwrap();

        assert_eq!(headers.address(), 0x00400000);
        assert_eq!(headers.size(), 4096);
        assert_eq!(headers.optional().image_size(), 18944000);
        assert_eq!(buffer, [1, 2, 3, 4].as_slice());

        let mut sections = headers.sections();

        let text = sections.next().unwrap();

        assert_eq!(text.name(), ".text");
        assert_eq!(text.address(), 0x00001000);

        let rdata = sections.next().unwrap();

        assert_eq!(rdata.name(), ".rdata");
        assert_eq!(rdata.address(), 0x001D8000);

        let data = sections.next().unwrap();

        assert_eq!(data.name(), ".data");
        assert_eq!(data.address(), 0x001E8000);

        let rsrc = sections.next().unwrap();

        assert_eq!(rsrc.name(), ".rsrc");
        assert_eq!(rsrc.address(), 0x0120E000);

        assert_eq!(sections.next(), None);
    }

    #[test]
    fn test_parse_slice() {
        let mut buffer = SAMPLE_HEADERS_DATA.as_slice();
        let headers = Headers::parse(&mut buffer).unwrap();

        assert_eq!(headers.address(), 0x00400000);
        assert_eq!(headers.size(), 4096);
        assert_eq!(headers.optional().image_size(), 18944000);
        assert_eq!(buffer.remaining(), 0);

        let mut sections = headers.sections();

        let text = sections.next().unwrap();

        assert_eq!(text.name(), ".text");
        assert_eq!(text.address(), 0x00001000);

        let rdata = sections.next().unwrap();

        assert_eq!(rdata.name(), ".rdata");
        assert_eq!(rdata.address(), 0x001D8000);

        let data = sections.next().unwrap();

        assert_eq!(data.name(), ".data");
        assert_eq!(data.address(), 0x001E8000);

        let rsrc = sections.next().unwrap();

        assert_eq!(rsrc.name(), ".rsrc");
        assert_eq!(rsrc.address(), 0x0120E000);

        assert_eq!(sections.next(), None);
    }
}
