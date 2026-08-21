mod analyse;
mod inspect;

use clap::Subcommand;

use self::analyse::Analyse;
use self::inspect::Inspect;

#[derive(Subcommand)]
pub enum Command {
    /// Analyse a binary.
    #[command(visible_alias = "analyze")]
    Analyse(Analyse),

    /// Inspect a binary analysis.
    Inspect(Inspect),
}
