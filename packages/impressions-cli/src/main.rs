mod command;

use clap::Parser;
use impressions::analysis::Error;

use self::command::Command;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

fn main() -> Result<(), Error> {
    let cli = Cli::parse();

    match cli.command {
        Command::Analyse(command) => command.exec()?,
        Command::Inspect(command) => command.exec()?,
    }

    Ok(())
}
