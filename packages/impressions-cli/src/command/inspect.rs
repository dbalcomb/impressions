use std::io;
use std::path::PathBuf;

use clap::Args;
use impressions::analysis::{Analysis, Error};
use impressions::memory::address::AddressSpace;
use impressions::memory::cursor::{AsCursor, Cursor};
use impressions::memory::inspect::Inspect as _;

use crate::inspector::table::TableInspector;

#[derive(Args)]
pub struct Inspect {
    /// The path of the binary analysis file.
    analysis: PathBuf,

    /// The target address space to inspect.
    #[arg(long, short = 'a')]
    address_space: Option<AddressSpace>,

    /// Use relative addresses instead of absolute addresses.
    #[arg(long, short = 'r')]
    relative: bool,
}

impl Inspect {
    pub fn exec(self) -> Result<(), Error> {
        let analysis = Analysis::open(&self.analysis)?;

        eprintln!("Analysis: {}", self.analysis.display());
        eprintln!("Completion: {:.2}%", analysis.completion());
        eprintln!();

        let stdout = io::stdout();

        let mut inspector = TableInspector::new(stdout.lock());
        let mut cursor = analysis.image().cursor().relative(self.relative);

        if let Some(address_space) = self.address_space {
            cursor.seek_address(address_space.first())?;
        }

        loop {
            cursor.inspect(&mut inspector);

            if let Some(err) = inspector.take_error() {
                return Err(Error::Io(err));
            }

            if cursor.step()?.is_none() {
                break;
            }

            if let Some(address_space) = self.address_space
                && let Some(address) = cursor.address()
                && address > address_space.last()
            {
                break;
            }
        }

        Ok(())
    }
}
