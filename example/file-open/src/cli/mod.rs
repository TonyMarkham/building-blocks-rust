pub(crate) mod command;

// -------------------------------------------------------------------------- //

use crate::cli::command::Command;

use clap::Parser;

#[derive(Debug, Parser)]
#[command(version, about)]
pub(crate) struct Cli {
    #[command(subcommand)]
    pub(crate) command: Command,
}
