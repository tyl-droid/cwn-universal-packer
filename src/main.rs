mod commands;
mod container;
mod filesystem;
mod security;

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "cwnpack",
    version,
    author = "Community Watch Network",
    about = "CWN Universal Packer",
    long_about = "Pack, inspect, verify, test and extract arbitrary files and directories using the CWN container format."
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Pack files and folders into a CWN container.
    Pack {
        /// Files and directories to include.
        #[arg(required = true)]
        inputs: Vec<PathBuf>,

        /// Output .CWN file.
        #[arg(short, long)]
        output: PathBuf,

        /// Zstandard compression level.
        #[arg(short, long, default_value_t = 10)]
        level: i32,
    },

    /// Extract a CWN container.
    Unpack {
        /// CWN container.
        input: PathBuf,

        /// Destination directory.
        #[arg(short, long, default_value = "CWN-Extracted")]
        output: PathBuf,
    },

    /// List container contents.
    List { input: PathBuf },

    /// Show container metadata.
    Info { input: PathBuf },

    /// Verify every file using SHA-256.
    Verify { input: PathBuf },

    /// Test CWN container structure without extracting it.
    Test { input: PathBuf },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Pack {
            inputs,
            output,
            level,
        } => commands::pack::run(inputs, output, level),

        Commands::Unpack { input, output } => commands::unpack::run(input, output),

        Commands::List { input } => commands::list::run(input),

        Commands::Info { input } => commands::info::run(input),

        Commands::Verify { input } => commands::verify::run(input),

        Commands::Test { input } => commands::test::run(input),
    }
}
