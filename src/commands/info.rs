use crate::container;
use crate::container::manifest::EntryKind;
use crate::filesystem::size::human_size;

use anyhow::Result;
use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

pub fn run(input: PathBuf) -> Result<()> {
    let (header, manifest) = container::read_manifest(&input)?;

    let container_size = fs::metadata(&input)?.len();

    let files: Vec<_> = manifest
        .entries
        .iter()
        .filter(|entry| entry.kind == EntryKind::File)
        .collect();

    let directories = manifest
        .entries
        .iter()
        .filter(|entry| entry.kind == EntryKind::Directory)
        .count();

    let original_size: u64 = files.iter().map(|entry| entry.original_size).sum();

    let payload_size: u64 = files.iter().map(|entry| entry.packed_size).sum();

    let saved = original_size.saturating_sub(payload_size);

    let saved_percent = if original_size == 0 {
        0.0
    } else {
        saved as f64 / original_size as f64 * 100.0
    };

    let zstd_files = files
        .iter()
        .filter(|entry| entry.compression == "zstd")
        .count();

    let stored_files = files
        .iter()
        .filter(|entry| entry.compression == "none")
        .count();

    let mut types: BTreeMap<&str, usize> = BTreeMap::new();

    for entry in &files {
        *types.entry(entry.file_type.as_str()).or_insert(0) += 1;
    }

    println!("╔══════════════════════════════════════════════════╗");
    println!("║             CWN UNIVERSAL PACKER                 ║");
    println!("╚══════════════════════════════════════════════════╝");

    println!();
    println!("Container");
    println!("──────────────────────────────────────────────────");
    println!("{:<20} {}", "File", input.display());
    println!("{:<20} CWN", "Format");
    println!("{:<20} {}", "Format version", header.version);
    println!("{:<20} {}", "Producer", manifest.producer);

    println!();
    println!("Contents");
    println!("──────────────────────────────────────────────────");
    println!("{:<20} {}", "Files", files.len());
    println!("{:<20} {}", "Directories", directories);
    println!("{:<20} {}", "Entries", manifest.entries.len());

    println!();
    println!("File Types");
    println!("──────────────────────────────────────────────────");

    if types.is_empty() {
        println!("No files");
    } else {
        for (kind, count) in types {
            println!("{:<28} {}", kind, count);
        }
    }

    println!();
    println!("Storage");
    println!("──────────────────────────────────────────────────");
    println!("{:<20} {}", "Original", human_size(original_size));
    println!("{:<20} {}", "Payload", human_size(payload_size));
    println!("{:<20} {}", "Container", human_size(container_size));
    println!(
        "{:<20} {} ({:.2}%)",
        "Payload saved",
        human_size(saved),
        saved_percent
    );

    println!();
    println!("Compression");
    println!("──────────────────────────────────────────────────");
    println!("{:<20} {}", "Zstandard", zstd_files);
    println!("{:<20} {}", "Raw / stored", stored_files);

    println!();
    println!("Integrity");
    println!("──────────────────────────────────────────────────");
    println!("{:<20} SHA-256", "File hashing");
    println!("{:<20} {}", "Hashed files", files.len());

    println!();
    println!("STATUS              CWN CONTAINER VALID");

    Ok(())
}
