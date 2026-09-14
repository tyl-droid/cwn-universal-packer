use crate::container;
use crate::container::manifest::{CwnEntry, EntryKind};
use crate::engine::progress::ProgressEvent;
use crate::filesystem::paths::safe_relative_path;

use anyhow::{Context, Result, bail};
use sha2::{Digest, Sha256};

use std::fs::{self, File};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

pub fn run(input: PathBuf, output: PathBuf) -> Result<()> {
    run_with_progress(input, output, |_| {})
}

pub fn run_with_progress<F>(input: PathBuf, output: PathBuf, mut progress: F) -> Result<()>
where
    F: FnMut(ProgressEvent),
{
    let (_, manifest) = container::read_manifest(&input)?;

    let total = manifest
        .entries
        .iter()
        .filter(|entry| entry.kind == EntryKind::File)
        .count();

    progress(ProgressEvent::Started {
        operation: "Extraction".to_string(),
        total_items: Some(total),
    });

    fs::create_dir_all(&output)?;

    let mut archive = File::open(&input)?;
    let mut current = 0usize;

    for entry in &manifest.entries {
        let relative = safe_relative_path(&entry.path)?;
        let destination = output.join(relative);

        match entry.kind {
            EntryKind::Directory => {
                fs::create_dir_all(&destination)?;
                println!("[DIR ] {}", entry.path);
            }

            EntryKind::File => {
                current += 1;

                progress(ProgressEvent::Item {
                    current,
                    total,
                    path: entry.path.clone(),
                });

                extract_file_entry(&mut archive, entry, &destination)?;

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

    progress(ProgressEvent::Finished);

    Ok(())
}

pub fn run_selected(input: PathBuf, output: PathBuf, selected_path: String) -> Result<()> {
    run_selected_with_progress(input, output, selected_path, |_| {})
}

pub fn run_selected_with_progress<F>(
    input: PathBuf,
    output: PathBuf,
    selected_path: String,
    mut progress: F,
) -> Result<()>
where
    F: FnMut(ProgressEvent),
{
    let (_, manifest) = container::read_manifest(&input)?;

    /*
     * Validate the requested path using the same path policy used for
     * archive entries before attempting to locate or extract it.
     */
    let requested_relative = safe_relative_path(&selected_path)?;

    let entry = manifest
        .entries
        .iter()
        .find(|entry| {
            safe_relative_path(&entry.path)
                .map(|path| path == requested_relative)
                .unwrap_or(false)
        })
        .with_context(|| format!("archive entry not found: {}", selected_path))?;

    progress(ProgressEvent::Started {
        operation: "Selected extraction".to_string(),
        total_items: Some(1),
    });

    fs::create_dir_all(&output)?;

    let relative = safe_relative_path(&entry.path)?;
    let destination = output.join(relative);

    match entry.kind {
        EntryKind::Directory => {
            fs::create_dir_all(&destination)?;

            progress(ProgressEvent::Item {
                current: 1,
                total: 1,
                path: entry.path.clone(),
            });

            println!("[DIR ] {}", entry.path);
        }

        EntryKind::File => {
            progress(ProgressEvent::Item {
                current: 1,
                total: 1,
                path: entry.path.clone(),
            });

            let mut archive = File::open(&input)?;

            extract_file_entry(&mut archive, entry, &destination)?;

            println!("[OK]   {} [{}]", entry.path, entry.compression);
        }
    }

    progress(ProgressEvent::Finished);

    println!(
        "Successfully extracted {} to {}",
        entry.path,
        destination.display()
    );

    Ok(())
}

fn extract_file_entry(archive: &mut File, entry: &CwnEntry, destination: &Path) -> Result<()> {
    if entry.kind != EntryKind::File {
        bail!("cannot extract non-file entry as a file: {}", entry.path);
    }

    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent)?;
    }

    if destination.exists() {
        bail!(
            "refusing to overwrite existing file: {}",
            destination.display()
        );
    }

    archive.seek(SeekFrom::Start(entry.data_offset))?;

    let mut output_file = File::create(destination)
        .with_context(|| format!("failed to create {}", destination.display()))?;

    let result = (|| -> Result<()> {
        extract_file_contents(archive, entry, &mut output_file)?;

        output_file
            .flush()
            .with_context(|| format!("failed to flush {}", destination.display()))?;

        Ok(())
    })();

    if let Err(error) = result {
        drop(output_file);
        let _ = fs::remove_file(destination);
        return Err(error);
    }

    Ok(())
}

fn extract_file_contents(
    archive: &mut File,
    entry: &CwnEntry,
    output_file: &mut File,
) -> Result<()> {
    let mut hasher = Sha256::new();
    let mut written = 0u64;
    let mut buffer = [0u8; 64 * 1024];

    match entry.compression.as_str() {
        "zstd" => {
            let limited = (&mut *archive).take(entry.packed_size);

            let mut decoder = zstd::stream::read::Decoder::new(limited)
                .with_context(|| format!("failed to decode {}", entry.path))?;

            loop {
                let count = decoder.read(&mut buffer)?;

                if count == 0 {
                    break;
                }

                if written.saturating_add(count as u64) > entry.original_size {
                    bail!("decompressed size exceeds declared size for {}", entry.path);
                }

                output_file.write_all(&buffer[..count])?;
                hasher.update(&buffer[..count]);

                written += count as u64;
            }
        }

        "none" => {
            let mut limited = (&mut *archive).take(entry.packed_size);

            loop {
                let count = limited.read(&mut buffer)?;

                if count == 0 {
                    break;
                }

                if written.saturating_add(count as u64) > entry.original_size {
                    bail!("decompressed size exceeds declared size for {}", entry.path);
                }

                output_file.write_all(&buffer[..count])?;
                hasher.update(&buffer[..count]);

                written += count as u64;
            }
        }

        other => {
            bail!("unsupported compression '{}' for {}", other, entry.path);
        }
    }

    if written != entry.original_size {
        bail!("size verification failed for {}", entry.path);
    }

    let actual_hash = hex::encode(hasher.finalize());

    let expected_hash = entry
        .sha256
        .as_ref()
        .context("file entry missing SHA-256")?;

    if actual_hash != *expected_hash {
        bail!("SHA-256 verification failed for {}", entry.path);
    }

    Ok(())
}
