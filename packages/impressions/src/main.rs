mod command;

use anyhow::Error;
use clap::Parser;

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
    }

    Ok(())
}
