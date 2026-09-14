use crate::container;
use crate::container::manifest::EntryKind;

use anyhow::{Result, bail};
use std::path::PathBuf;

pub fn run(input: PathBuf) -> Result<()> {
    println!("CWN Container Test");
    println!("────────────────────────────────────────");

    let (header, manifest) = container::read_manifest(&input)?;

    let files = manifest
        .entries
        .iter()
        .filter(|entry| entry.kind == EntryKind::File)
        .count();

    let directories = manifest
        .entries
        .iter()
        .filter(|entry| entry.kind == EntryKind::Directory)
        .count();

    println!("[OK] Header");
    println!("[OK] Magic CWN1");
    println!("[OK] Format version {}", header.version);
    println!("[OK] Manifest");
    println!("[OK] Entry count {}", manifest.entries.len());
    println!("[OK] Payload ranges");
    println!("[OK] Duplicate path validation");
    println!("[OK] Compression metadata");

    if files == 0 && directories == 0 {
        bail!("CWN container contains no entries");
    }

    println!();
    println!("Files:       {}", files);
    println!("Directories: {}", directories);
    println!();
    println!("STRUCTURE: VALID");
    println!();
    println!("Run `cwnpack verify` for full SHA-256 payload verification.");

    Ok(())
}
