//! The image file headers.

mod header;

use std::fmt::{self, Debug};
use std::iter::once;

use bytes::{Buf, TryGetError};
use serde::{Deserialize, Serialize};

use crate::analysis::Completion;
use crate::data::parse::Parse;
use crate::memory::Extent;
use crate::memory::regions::contiguous::{Contiguous, Segment};
use crate::memory::regions::unidentified::Unidentified;

pub use self::header::{
    CoffHeader, DataDirectory, DataDirectoryTable, DosHeader, Header, OptionalHeader,
    SectionCharacteristics, SectionHeader,
};

use super::{Error, Padding};

/// The signature indicating the start of the PE headers.
const PE_SIGNATURE: u32 = 0x4550;

/// The 32-bit Portable Executable (PE) image file headers.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Headers(Contiguous<Header>);

impl Headers {
    /// Gets the DOS header.
    pub fn dos(&self) -> &DosHeader {
        self.0
            .segments()
            .flat_map(|entry| entry.segment().as_identified())
            .flat_map(Header::as_dos)
            .next()
            .expect("DOS header not found")
    }

    /// Gets the COFF header.
    pub fn coff(&self) -> &CoffHeader {
        self.0
            .segments()
            .flat_map(|entry| entry.segment().as_identified())
            .flat_map(Header::as_coff)
            .next()
            .expect("COFF header not found")
    }

    /// Gets the Optional header.
    pub fn optional(&self) -> &OptionalHeader {
        self.0
            .segments()
            .flat_map(|entry| entry.segment().as_identified())
            .flat_map(Header::as_optional)
            .next()
            .expect("Optional header not found")
    }

    /// Gets an iterator over the section headers.
    pub fn sections(&self) -> impl Iterator<Item = &SectionHeader> {
        self.0
            .segments()
            .flat_map(|entry| entry.segment().as_identified())
            .flat_map(Header::as_section)
    }
}

impl Extent for Headers {
    fn size(&self) -> u64 {
        self.optional().headers_size()
    }
}

impl Completion for Headers {
    fn identified(&self) -> u64 {
        self.0.identified()
    }
}

impl Parse for Headers {
    type Context<'a> = ();
    type Error = Error;

    fn parse_with(mut buffer: impl Buf, _: Self::Context<'_>) -> Result<Self, Self::Error> {
        let dos = DosHeader::parse(&mut buffer)?;
        let offset = dos.pe_headers_offset() as usize - dos.size() as usize;

        if offset > buffer.remaining() {
            return Err(Error::Parse(TryGetError {
                requested: offset,
                available: buffer.remaining(),
            }));
        }

        let stub = if offset > 0 {
            Some(Unidentified::new(buffer.copy_to_bytes(offset), 0)?)
        } else {
            None
        };

        let signature = buffer.try_get_u32_le()?;

        if signature != PE_SIGNATURE {
            return Err(Error::InvalidSignature);
        }

        let coff = CoffHeader::parse(&mut buffer)?;
        let optional = OptionalHeader::parse(&mut buffer)?;
        let sections = (0..coff.number_of_sections())
            .map(|_| SectionHeader::parse(&mut buffer))
            .collect::<Result<Vec<_>, _>>()?;

        let file_offset = dos.pe_headers_offset() as usize
            + 4
            + coff.size() as usize
            + optional.size() as usize
            + sections.iter().map(Extent::size).sum::<u64>() as usize;

        let remaining = buffer
            .remaining()
            .min(optional.headers_size() as usize - file_offset);

        buffer.advance(remaining);

        let padding = if remaining > 0 {
            Some(Header::Padding(Padding::new(remaining as u64, 0)))
        } else {
            None
        };

        let headers = once(Segment::Identified(Header::Dos(dos)))
            .chain(stub.map(Segment::Unidentified))
            .chain(once(Segment::Identified(Header::Signature)))
            .chain(once(Segment::Identified(Header::Coff(coff))))
            .chain(once(Segment::Identified(Header::Optional(optional))))
            .chain(
                sections
                    .into_iter()
                    .map(Header::Section)
                    .map(Segment::Identified),
            )
            .chain(padding.map(Segment::Identified));

        Ok(Self(Contiguous::try_from_iterator(headers)?))
    }
}

impl Debug for Headers {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Debug::fmt(&self.0, f)
    }
}

#[cfg(test)]
mod tests {
    use bytes::{Buf, Bytes, BytesMut};

    use crate::data::parse::Parse;
    use crate::memory::Extent;
    use crate::memory::address::Address;

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

        assert_eq!(headers.optional().image_address(), Address::new(0x00400000));
        assert_eq!(headers.size(), 4096);
        assert_eq!(headers.optional().image_size(), 18944000);
        assert_eq!(buffer, [1, 2, 3, 4].as_slice());

        let data_directories = headers.optional().data_directories();
        let import_table = data_directories.import_table().unwrap();
        let import_address_table = data_directories.import_address_table().unwrap();
        let resource_table = data_directories.resource_table().unwrap();

        assert_eq!(import_table.target_address(), Address::new(0x001e5f90));
        assert_eq!(import_table.target_size(), 220);

        assert_eq!(
            import_address_table.target_address(),
            Address::new(0x001d8000)
        );
        assert_eq!(import_address_table.target_size(), 848);

        assert_eq!(resource_table.target_address(), Address::new(0x0120e000));
        assert_eq!(resource_table.target_size(), 10448);

        let mut sections = headers.sections();

        let text = sections.next().unwrap();

        assert_eq!(text.name(), ".text");
        assert_eq!(text.section_address(), Address::new(0x00001000));

        assert!(text.characteristics().read());
        assert!(!text.characteristics().write());
        assert!(text.characteristics().execute());
        assert!(!text.characteristics().share());
        assert!(text.characteristics().page());
        assert!(text.characteristics().cache());
        assert!(!text.characteristics().discard());
        assert!(!text.characteristics().initialised());
        assert!(!text.characteristics().uninitialised());
        assert!(text.characteristics().code());

        let rdata = sections.next().unwrap();

        assert_eq!(rdata.name(), ".rdata");
        assert_eq!(rdata.section_address(), Address::new(0x001D8000));

        assert!(rdata.characteristics().read());
        assert!(!rdata.characteristics().write());
        assert!(!rdata.characteristics().execute());
        assert!(!rdata.characteristics().share());
        assert!(rdata.characteristics().page());
        assert!(rdata.characteristics().cache());
        assert!(!rdata.characteristics().discard());
        assert!(rdata.characteristics().initialised());
        assert!(!rdata.characteristics().uninitialised());
        assert!(!rdata.characteristics().code());

        let data = sections.next().unwrap();

        assert_eq!(data.name(), ".data");
        assert_eq!(data.section_address(), Address::new(0x001E8000));

        assert!(data.characteristics().read());
        assert!(data.characteristics().write());
        assert!(!data.characteristics().execute());
        assert!(!data.characteristics().share());
        assert!(data.characteristics().page());
        assert!(data.characteristics().cache());
        assert!(!data.characteristics().discard());
        assert!(data.characteristics().initialised());
        assert!(!data.characteristics().uninitialised());
        assert!(!data.characteristics().code());

        let rsrc = sections.next().unwrap();

        assert_eq!(rsrc.name(), ".rsrc");
        assert_eq!(rsrc.section_address(), Address::new(0x0120E000));

        assert!(rsrc.characteristics().read());
        assert!(!rsrc.characteristics().write());
        assert!(!rsrc.characteristics().execute());
        assert!(!rsrc.characteristics().share());
        assert!(rsrc.characteristics().page());
        assert!(rsrc.characteristics().cache());
        assert!(!rsrc.characteristics().discard());
        assert!(rsrc.characteristics().initialised());
        assert!(!rsrc.characteristics().uninitialised());
        assert!(!rsrc.characteristics().code());

        assert_eq!(sections.next(), None);
    }

    #[test]
    fn test_parse_slice() {
        let mut buffer = SAMPLE_HEADERS_DATA.as_slice();
        let headers = Headers::parse(&mut buffer).unwrap();

        assert_eq!(headers.optional().image_address(), Address::new(0x00400000));
        assert_eq!(headers.size(), 4096);
        assert_eq!(headers.optional().image_size(), 18944000);
        assert_eq!(buffer.remaining(), 0);

        let mut sections = headers.sections();

        let text = sections.next().unwrap();

        assert_eq!(text.name(), ".text");
        assert_eq!(text.section_address(), Address::new(0x00001000));

        let rdata = sections.next().unwrap();

        assert_eq!(rdata.name(), ".rdata");
        assert_eq!(rdata.section_address(), Address::new(0x001D8000));

        let data = sections.next().unwrap();

        assert_eq!(data.name(), ".data");
        assert_eq!(data.section_address(), Address::new(0x001E8000));

        let rsrc = sections.next().unwrap();

        assert_eq!(rsrc.name(), ".rsrc");
        assert_eq!(rsrc.section_address(), Address::new(0x0120E000));

        assert_eq!(sections.next(), None);
    }
}
