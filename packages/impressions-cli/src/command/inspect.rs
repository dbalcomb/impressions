use std::path::PathBuf;
use std::{fmt, io};

use clap::Args;
use impressions::analysis::{Analysis, Error};
use impressions::memory::cursor::{AsCursor, Cursor};
use impressions::memory::inspect::{Inspect as _, Inspector};

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
        let mut writer = IoWriter {
            writer: stdout.lock(),
            error: None,
        };

        let mut cursor = analysis.image().cursor();

        loop {
            let address = cursor.address().expect("addressable within loop");
            let mut inspector = Inspector::new(address, &mut writer);

            cursor.inspect(&mut inspector)?;

            if let Some(err) = writer.error.take() {
                return Err(Error::Io(err));
            }

            if cursor.step()?.is_none() {
                break;
            }
        }

        Ok(())
    }
}

struct IoWriter<W> {
    writer: W,
    error: Option<io::Error>,
}

impl<W> fmt::Write for IoWriter<W>
where
    W: io::Write,
{
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.writer.write_all(s.as_bytes()).map_err(|err| {
            self.error = Some(err);

            fmt::Error
        })
    }
}
