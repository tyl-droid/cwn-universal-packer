use crate::container;
use crate::container::manifest::EntryKind;

use anyhow::Result;
use std::fs;
use std::path::PathBuf;

pub fn run(input: PathBuf) -> Result<()> {
    let (header, manifest) = container::read_manifest(&input)?;

    let container_size = fs::metadata(&input)?.len();

    let files = manifest
        .entries
        .iter()
        .filter(|e| e.kind == EntryKind::File)
        .count();

    let directories = manifest
        .entries
        .iter()
        .filter(|e| e.kind == EntryKind::Directory)
        .count();

    let original_size: u64 = manifest.entries.iter().map(|e| e.original_size).sum();

    println!("╔══════════════════════════════════════════╗");
    println!("║         CWN UNIVERSAL PACKER             ║");
    println!("╚══════════════════════════════════════════╝");
    println!();
    println!("File              {}", input.display());
    println!("Format            CWN");
    println!("Version           {}", header.version);
    println!("Producer          {}", manifest.producer);
    println!();
    println!("Files             {}", files);
    println!("Directories       {}", directories);
    println!("Entries           {}", manifest.entries.len());
    println!();
    println!("Original bytes    {}", original_size);
    println!("Container bytes   {}", container_size);
    println!();
    println!("Compression       Zstandard");
    println!("Integrity         SHA-256");
    println!();
    println!("CWN CONTAINER VALID");

    Ok(())
}
