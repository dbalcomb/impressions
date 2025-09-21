use std::path::PathBuf;

use anyhow::Error;
use clap::Args;

#[derive(Args)]
pub struct Analyse {
    /// The path of the binary to analyse.
    path: PathBuf,
}

impl Analyse {
    pub fn exec(self) -> Result<(), Error> {
        println!("Analysing `{}`...", self.path.display());

        let metadata = std::fs::metadata(self.path)?;

        println!("Size: {} bytes", metadata.len());

        Ok(())
    }
}
