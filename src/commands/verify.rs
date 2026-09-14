use crate::container;
use crate::container::manifest::EntryKind;
use crate::security::hash::sha256_reader_counted;

use anyhow::{Context, Result, bail};
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::PathBuf;

pub fn run(input: PathBuf) -> Result<()> {
    let (_, manifest) = container::read_manifest(&input)?;

    let mut archive = File::open(&input)?;

    let mut valid = 0usize;
    let mut failed = 0usize;

    println!("CWN Integrity Verification");
    println!("────────────────────────────────────────");

    for entry in &manifest.entries {
        if entry.kind != EntryKind::File {
            continue;
        }

        archive.seek(SeekFrom::Start(entry.data_offset))?;

        let expected = entry
            .sha256
            .as_ref()
            .context("file entry missing SHA-256")?;

        let result = match entry.compression.as_str() {
            "zstd" => {
                let limited = (&mut archive).take(entry.packed_size);

                let decoder = zstd::stream::read::Decoder::new(limited)
                    .with_context(|| format!("failed to decode {}", entry.path))?;

                // Never permit decompression to exceed the declared size
                // by more than one byte.
                let mut bounded = decoder.take(entry.original_size.saturating_add(1));

                sha256_reader_counted(&mut bounded)?
            }

            "none" => {
                let mut limited = (&mut archive).take(entry.packed_size);

                sha256_reader_counted(&mut limited)?
            }

            other => {
                bail!("unsupported compression '{}' for {}", other, entry.path);
            }
        };

        let (actual_hash, actual_size) = result;

        if actual_size != entry.original_size {
            println!(
                "[FAIL] {} (size {} != {})",
                entry.path, actual_size, entry.original_size
            );
            failed += 1;
            continue;
        }

        if actual_hash == *expected {
            println!("[OK]   {}", entry.path);
            valid += 1;
        } else {
            println!("[FAIL] {}", entry.path);
            failed += 1;
        }
    }

    println!();
    println!("Valid:   {}", valid);
    println!("Failed:  {}", failed);

    if failed > 0 {
        bail!("CWN container integrity verification failed");
    }

    println!();
    println!("Container integrity: VERIFIED");

    Ok(())
}
