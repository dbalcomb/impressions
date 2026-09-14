use std::io;
use std::path::PathBuf;

use clap::Args;
use impressions::analysis::{Analysis, Error};
use impressions::memory::cursor::{AsCursor, Cursor};
use impressions::memory::inspect::Inspect as _;

use crate::inspector::table::TableInspector;

#[derive(Args)]
pub struct Inspect {
    /// The path of the binary analysis file.
    analysis: PathBuf,
}

impl Inspect {
    pub fn exec(self) -> Result<(), Error> {
        let analysis = Analysis::open(&self.analysis)?;

        eprintln!("Analysis: {}", self.analysis.display());
        eprintln!("Completion: {:.2}%", analysis.completion());
        eprintln!();

        let stdout = io::stdout();
        let address = analysis.image().headers().optional().image_address();

        let mut table = TableInspector::new(stdout.lock());
        let mut cursor = analysis.image().cursor();

        loop {
            cursor.inspect(&mut table.at(address));

            if let Some(err) = table.take_error() {
                return Err(Error::Io(err));
            }

            if cursor.step()?.is_none() {
                break;
            }
        }

        Ok(())
    }
}
