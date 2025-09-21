mod analyse;

use clap::Subcommand;

use self::analyse::Analyse;

#[derive(Subcommand)]
pub enum Command {
    /// Analyse a binary.
    #[command(visible_alias = "analyze")]
    Analyse(Analyse),
}
