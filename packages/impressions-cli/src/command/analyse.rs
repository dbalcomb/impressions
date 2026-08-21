use std::path::PathBuf;

use clap::Args;
use impressions::analysis::{Analysis, Error};

#[derive(Args)]
pub struct Analyse {
    /// The PE32 image file (.exe / .dll) to analyse.
    image: PathBuf,

    /// The path of the binary analysis file.
    #[arg(long)]
    analysis: Option<PathBuf>,
}

impl Analyse {
    pub fn exec(self) -> Result<(), Error> {
        let path = match self.analysis {
            Some(path) => path,
            None => self.image.with_extension("impressions"),
        };

        let analysis = Analysis::build(&path).with_binary_path(&self.image)?;

        eprintln!("Binary:     {}", self.image.display());
        eprintln!("Analysis:   {}", path.display());
        eprintln!("Completion: {:.2}%", analysis.completion());

        analysis.save()?;

        Ok(())
    }
}
