use std::path::PathBuf;

use clap::Args;
use impressions::analysis::{Analysis, Error};

#[derive(Args)]
pub struct Inspect {
    /// The path of the binary analysis file.
    analysis: PathBuf,
}

impl Inspect {
    pub fn exec(self) -> Result<(), Error> {
        let analysis = Analysis::open(&self.analysis)?;

        eprintln!("Analysis:   {}", self.analysis.display());
        eprintln!("Completion: {:.2}%", analysis.completion());
        eprintln!("\n{analysis:#?}");

        Ok(())
    }
}
