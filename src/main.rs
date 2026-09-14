use anyhow::Result;
use clap::{Parser, Subcommand};
use cwn_universal_packer::container::manifest::PackageMetadata;
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

        /// Package name stored in the CWN manifest.
        #[arg(long)]
        package_name: Option<String>,

        /// Package version stored in the CWN manifest.
        #[arg(long)]
        package_version: Option<String>,

        /// Publisher stored in the CWN manifest.
        #[arg(long)]
        publisher: Option<String>,
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
            package_name,
            package_version,
            publisher,
        } => {
            if package_name.is_none() && package_version.is_none() && publisher.is_none() {
                cwn_universal_packer::commands::pack::run(inputs, output, level)
            } else {
                let default_name = output
                    .file_stem()
                    .and_then(|name| name.to_str())
                    .unwrap_or("CWN Package")
                    .to_string();

                let metadata = PackageMetadata::new(
                    package_name.unwrap_or(default_name),
                    package_version.unwrap_or_else(|| env!("CARGO_PKG_VERSION").to_string()),
                    publisher.unwrap_or_else(|| "Community Watch Network".to_string()),
                );

                cwn_universal_packer::commands::pack::run_with_metadata(
                    inputs, output, level, metadata,
                )
            }
        }

        Commands::Unpack { input, output } => {
            cwn_universal_packer::commands::unpack::run(input, output)
        }

        Commands::List { input } => cwn_universal_packer::commands::list::run(input),

        Commands::Info { input } => cwn_universal_packer::commands::info::run(input),

        Commands::Verify { input } => cwn_universal_packer::commands::verify::run(input),

        Commands::Test { input } => cwn_universal_packer::commands::test::run(input),
    }
}
