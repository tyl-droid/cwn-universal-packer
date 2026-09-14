use crate::container;
use crate::container::manifest::EntryKind;
use crate::filesystem::size::human_size;

use anyhow::Result;
use std::path::PathBuf;

pub fn run(input: PathBuf) -> Result<()> {
    let (_, manifest) = container::read_manifest(&input)?;

    println!("CWN Container: {}", input.display());
    println!();

    println!(
        "{:<20} {:>12} {:>12} {:<8}  PATH",
        "TYPE", "SIZE", "STORED", "METHOD"
    );

    println!("{}", "─".repeat(82));

    for entry in manifest.entries {
        match entry.kind {
            EntryKind::Directory => {
                println!(
                    "{:<20} {:>12} {:>12} {:<8}  {}",
                    "Directory", "-", "-", "-", entry.path
                );
            }

            EntryKind::File => {
                println!(
                    "{:<20} {:>12} {:>12} {:<8}  {}",
                    entry.file_type,
                    human_size(entry.original_size),
                    human_size(entry.packed_size),
                    entry.compression.to_uppercase(),
                    entry.path
                );
            }
        }
    }

    Ok(())
}
