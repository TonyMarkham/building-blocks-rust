use clap::Subcommand;
use std::path::PathBuf;

#[derive(Debug, Subcommand)]
pub(crate) enum Command {
    /// Open a file and report whether it succeeded.
    Open { path: PathBuf },

    /// Print the whole file.
    ReadText { path: PathBuf },

    /// Print the file line by line, with line numbers.
    ReadLines { path: PathBuf },

    /// Show the size and colour format of a PNG image.
    PngInfo { path: PathBuf },
}
