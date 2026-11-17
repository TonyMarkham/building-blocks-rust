mod cli;
mod error;
mod file;

// -------------------------------------------------------------------------- //

pub use error::{Error as FileError, result::Result as FileResult};

// -------------------------------------------------------------------------- //

use crate::cli::{Cli, command::Command};

use clap::Parser;
use std::{path::Path, process::ExitCode};

#[tokio::main]
async fn main() -> ExitCode {
    let cli = Cli::parse();

    match run(cli.command).await {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("{err}");
            ExitCode::FAILURE
        }
    }
}

async fn run(command: Command) -> FileResult<()> {
    match command {
        Command::Open { path } => open(&path).await,
        Command::ReadText { path } => read_text(&path).await,
        Command::ReadLines { path } => read_lines(&path).await,
    }
}

async fn open(path: &Path) -> FileResult<()> {
    file::open(path).await?;
    println!("opened {}", path.display());
    Ok(())
}

async fn read_text(path: &Path) -> FileResult<()> {
    let text = file::read_text(path).await?;
    print!("{text}");
    Ok(())
}

async fn read_lines(path: &Path) -> FileResult<()> {
    let lines = file::read_lines(path).await?;
    for (number, line) in lines.iter().enumerate() {
        println!("{:>4}: {line}", number + 1);
    }
    Ok(())
}
