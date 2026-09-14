use crate::engine::inspect::inspect_container;
use crate::filesystem::size::human_size;

use anyhow::Result;
use std::collections::BTreeMap;
use std::path::PathBuf;

pub fn run(input: PathBuf) -> Result<()> {
    let info = inspect_container(&input)?;

    let saved = info.original_size.saturating_sub(info.payload_size);

    let saved_percent = if info.original_size == 0 {
        0.0
    } else {
        saved as f64 / info.original_size as f64 * 100.0
    };

    let mut types: BTreeMap<&str, usize> = BTreeMap::new();

    for entry in &info.entries {
        if !entry.is_directory {
            *types.entry(entry.file_type.as_str()).or_insert(0) += 1;
        }
    }

    println!("╔══════════════════════════════════════════════════╗");
    println!("║             CWN UNIVERSAL PACKER                 ║");
    println!("╚══════════════════════════════════════════════════╝");

    println!();
    println!("Container");
    println!("──────────────────────────────────────────────────");

    println!("{:<20} {}", "File", info.path.display());
    println!("{:<20} CWN", "Format");
    println!("{:<20} {}", "Format version", info.format_version);

    println!("{:<20} {}", "Package", info.package_name);
    println!("{:<20} {}", "Package version", info.package_version);
    println!("{:<20} {}", "Publisher", info.publisher);
    println!("{:<20} {}", "Producer", info.producer);

    println!();
    println!("Contents");
    println!("──────────────────────────────────────────────────");
    println!("{:<20} {}", "Files", info.files);
    println!("{:<20} {}", "Directories", info.directories);
    println!("{:<20} {}", "Entries", info.entries.len());

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

    println!("{:<20} {}", "Original", human_size(info.original_size));

    println!("{:<20} {}", "Payload", human_size(info.payload_size));

    println!("{:<20} {}", "Container", human_size(info.container_size));

    println!(
        "{:<20} {} ({:.2}%)",
        "Payload saved",
        human_size(saved),
        saved_percent
    );

    println!();
    println!("Compression");
    println!("──────────────────────────────────────────────────");
    println!("{:<20} {}", "Zstandard", info.zstd_files);
    println!("{:<20} {}", "Raw / stored", info.stored_files);

    println!();
    println!("Integrity");
    println!("──────────────────────────────────────────────────");
    println!("{:<20} SHA-256", "File hashing");
    println!("{:<20} {}", "Hashed files", info.files);

    println!();
    println!("STATUS              CWN CONTAINER VALID");

    Ok(())
}
