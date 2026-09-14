use crate::container;
use crate::container::manifest::EntryKind;
use crate::security::hash::sha256_reader;

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

        let actual = match entry.compression.as_str() {
            "zstd" => {
                let limited = (&mut archive).take(entry.packed_size);

                let mut decoder = zstd::stream::read::Decoder::new(limited)
                    .with_context(|| format!("failed to decode {}", entry.path))?;

                sha256_reader(&mut decoder)?
            }

            "none" => {
                let mut limited = (&mut archive).take(entry.packed_size);
                sha256_reader(&mut limited)?
            }

            other => {
                bail!("unsupported compression '{}' for {}", other, entry.path);
            }
        };

        if actual == *expected {
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
