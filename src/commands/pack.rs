use crate::container::header::CwnHeader;
use crate::container::manifest::{CwnEntry, CwnManifest, EntryKind};
use crate::filesystem::filetype::detect_file_type;
use crate::security::hash::sha256_reader;

use anyhow::{Context, Result, bail};
use std::fs::{self, File};
use std::io::{self, BufWriter, Seek, Write};
use std::path::{Path, PathBuf};
use tempfile::tempfile;
use walkdir::WalkDir;

pub fn run(inputs: Vec<PathBuf>, output: PathBuf, level: i32) -> Result<()> {
    if inputs.is_empty() {
        bail!("no input files or directories were provided");
    }

    if output.exists() {
        bail!("output already exists: {}", output.display());
    }

    let output_file =
        File::create(&output).with_context(|| format!("failed to create {}", output.display()))?;

    let mut writer = BufWriter::new(output_file);

    let mut header = CwnHeader::placeholder();
    header.write(&mut writer)?;

    let mut entries = Vec::new();

    for input in inputs {
        if !input.exists() {
            bail!("input does not exist: {}", input.display());
        }

        if input.is_file() {
            let archive_path = input
                .file_name()
                .context("invalid input filename")?
                .to_string_lossy()
                .to_string();

            pack_file(&input, archive_path, &mut writer, &mut entries, level)?;
        } else if input.is_dir() {
            pack_directory(&input, &mut writer, &mut entries, level)?;
        }
    }

    let package_name = output
        .file_stem()
        .and_then(|name| name.to_str())
        .unwrap_or("CWN Package")
        .to_string();

    let manifest = CwnManifest::new(entries, package_name);

    let manifest_bytes =
        serde_json::to_vec_pretty(&manifest).context("failed to encode manifest")?;

    let manifest_offset = writer.stream_position()?;

    writer.write_all(&manifest_bytes)?;
    writer.flush()?;

    header.manifest_offset = manifest_offset;
    header.manifest_size = manifest_bytes.len() as u64;
    header.entry_count = manifest.entries.len() as u64;

    header.rewrite(&mut writer)?;
    writer.flush()?;

    let packed_size = fs::metadata(&output)?.len();

    let original_size: u64 = manifest
        .entries
        .iter()
        .filter(|entry| entry.kind == EntryKind::File)
        .map(|entry| entry.original_size)
        .sum();

    let payload_size: u64 = manifest
        .entries
        .iter()
        .filter(|entry| entry.kind == EntryKind::File)
        .map(|entry| entry.packed_size)
        .sum();

    let saved = original_size.saturating_sub(payload_size);

    let ratio = if original_size == 0 {
        0.0
    } else {
        (saved as f64 / original_size as f64) * 100.0
    };

    let compressed_files = manifest
        .entries
        .iter()
        .filter(|entry| entry.kind == EntryKind::File && entry.compression == "zstd")
        .count();

    let stored_files = manifest
        .entries
        .iter()
        .filter(|entry| entry.kind == EntryKind::File && entry.compression == "none")
        .count();

    println!();
    println!("CWN Universal Packer");
    println!("────────────────────────────────────────");
    println!("Output:          {}", output.display());
    println!("Entries:         {}", manifest.entries.len());
    println!("Original size:   {} bytes", original_size);
    println!("Payload size:    {} bytes", payload_size);
    println!("Container size:  {} bytes", packed_size);
    println!("Saved:           {} bytes ({:.2}%)", saved, ratio);
    println!("Zstd files:      {}", compressed_files);
    println!("Stored files:    {}", stored_files);
    println!("Format:          CWN v1");
    println!("Integrity:       SHA-256");
    println!();
    println!("CWN container created successfully.");

    Ok(())
}

fn pack_directory(
    root: &Path,
    writer: &mut BufWriter<File>,
    entries: &mut Vec<CwnEntry>,
    level: i32,
) -> Result<()> {
    let parent = root.parent().unwrap_or_else(|| Path::new(""));

    for item in WalkDir::new(root).follow_links(false) {
        let item = item?;
        let path = item.path();

        let relative = path
            .strip_prefix(parent)
            .context("failed to determine archive path")?;

        let archive_path = relative.to_string_lossy().replace('\\', "/");

        if item.file_type().is_dir() {
            entries.push(CwnEntry {
                path: archive_path,
                kind: EntryKind::Directory,
                file_type: "Directory".to_string(),
                original_size: 0,
                packed_size: 0,
                data_offset: 0,
                compression: "none".to_string(),
                sha256: None,
            });
        } else if item.file_type().is_file() {
            pack_file(path, archive_path, writer, entries, level)?;
        }
    }

    Ok(())
}

fn pack_file(
    source: &Path,
    archive_path: String,
    writer: &mut BufWriter<File>,
    entries: &mut Vec<CwnEntry>,
    level: i32,
) -> Result<()> {
    let original_size = fs::metadata(source)?.len();

    let mut hash_file = File::open(source)?;
    let sha256 = sha256_reader(&mut hash_file)?;

    let mut compressed = tempfile()?;

    {
        let mut input = File::open(source)?;

        zstd::stream::copy_encode(&mut input, &mut compressed, level)
            .with_context(|| format!("failed to compress {}", source.display()))?;
    }

    let compressed_size = compressed.metadata()?.len();

    let data_offset = writer.stream_position()?;

    let (compression, packed_size);

    if compressed_size < original_size {
        compressed.rewind()?;
        io::copy(&mut compressed, writer)?;

        compression = "zstd".to_string();
        packed_size = compressed_size;

        let saved = original_size - compressed_size;

        let percent = if original_size == 0 {
            0.0
        } else {
            (saved as f64 / original_size as f64) * 100.0
        };

        println!(
            "[ZSTD] {}  {} -> {} bytes  ({:.2}% saved)",
            archive_path, original_size, compressed_size, percent
        );
    } else {
        let mut input = File::open(source)?;
        io::copy(&mut input, writer)?;

        compression = "none".to_string();
        packed_size = original_size;

        println!("[STORE] {}  {} bytes", archive_path, original_size);
    }

    writer.flush()?;

    entries.push(CwnEntry {
        path: archive_path,
        kind: EntryKind::File,
        file_type: detect_file_type(source),
        original_size,
        packed_size,
        data_offset,
        compression,
        sha256: Some(sha256),
    });

    Ok(())
}
