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

        loop {
            cursor.inspect(&mut inspector);

            if let Some(err) = inspector.take_error() {
                return Err(Error::Io(err));
            }

            if cursor.step()?.is_none() {
                break;
            }
        }

        Ok(())
    }
}
