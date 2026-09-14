use crate::container;
use crate::container::manifest::EntryKind;
use crate::filesystem::paths::safe_relative_path;

use anyhow::{Context, Result, bail};
use sha2::{Digest, Sha256};

use std::fs::{self, File};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::PathBuf;

pub fn run(input: PathBuf, output: PathBuf) -> Result<()> {
    let (_, manifest) = container::read_manifest(&input)?;

    fs::create_dir_all(&output)?;

    let mut archive = File::open(&input)?;

    for entry in &manifest.entries {
        let relative = safe_relative_path(&entry.path)?;
        let destination = output.join(relative);

        match entry.kind {
            EntryKind::Directory => {
                fs::create_dir_all(&destination)?;
                println!("[DIR ] {}", entry.path);
            }

            EntryKind::File => {
                if let Some(parent) = destination.parent() {
                    fs::create_dir_all(parent)?;
                }

                archive.seek(SeekFrom::Start(entry.data_offset))?;

                if destination.exists() {
                    bail!(
                        "refusing to overwrite existing file: {}",
                        destination.display()
                    );
                }

                let mut output_file = File::create(&destination)?;
                let mut hasher = Sha256::new();
                let mut written = 0u64;
                let mut buffer = [0u8; 64 * 1024];

                match entry.compression.as_str() {
                    "zstd" => {
                        let limited = (&mut archive).take(entry.packed_size);

                        let mut decoder = zstd::stream::read::Decoder::new(limited)
                            .with_context(|| format!("failed to decode {}", entry.path))?;

                        loop {
                            let count = decoder.read(&mut buffer)?;

                            if count == 0 {
                                break;
                            }

                            output_file.write_all(&buffer[..count])?;
                            hasher.update(&buffer[..count]);
                            written += count as u64;
                        }
                    }

                    "none" => {
                        let mut limited = (&mut archive).take(entry.packed_size);

                        loop {
                            let count = limited.read(&mut buffer)?;

                            if count == 0 {
                                break;
                            }

                            output_file.write_all(&buffer[..count])?;
                            hasher.update(&buffer[..count]);
                            written += count as u64;
                        }
                    }

                    other => {
                        let _ = fs::remove_file(&destination);

                        bail!("unsupported compression '{}' for {}", other, entry.path);
                    }
                }

                output_file.flush()?;

                if written != entry.original_size {
                    let _ = fs::remove_file(&destination);

                    bail!("size verification failed for {}", entry.path);
                }

                let actual_hash = hex::encode(hasher.finalize());

                let expected_hash = entry
                    .sha256
                    .as_ref()
                    .context("file entry missing SHA-256")?;

                if actual_hash != *expected_hash {
                    let _ = fs::remove_file(&destination);

                    bail!("SHA-256 verification failed for {}", entry.path);
                }

                println!("[OK]   {} [{}]", entry.path, entry.compression);
            }
        }
    }

    println!();
    println!(
        "Successfully extracted {} entries to {}",
        manifest.entries.len(),
        output.display()
    );

    Ok(())
}
