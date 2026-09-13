use crate::container;
use crate::container::manifest::EntryKind;

use anyhow::Result;
use std::path::PathBuf;

pub fn run(input: PathBuf) -> Result<()> {
    let (_, manifest) = container::read_manifest(&input)?;

    println!("CWN Container: {}", input.display());
    println!();

    for entry in manifest.entries {
        match entry.kind {
            EntryKind::Directory => {
                println!("[DIR ] {}", entry.path);
            }

            EntryKind::File => {
                println!("[FILE] {:>12} bytes  {}", entry.original_size, entry.path);
            }
        }
    }

    Ok(())
}
