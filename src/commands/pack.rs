use crate::container::header::CwnHeader;
use crate::container::manifest::{CwnEntry, CwnManifest, EntryKind};
use crate::security::hash::sha256_reader;

use anyhow::{Context, Result, bail};
use std::fs::{self, File};
use std::io::{BufWriter, Seek, Write};
use std::path::{Path, PathBuf};
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

    let manifest = CwnManifest::new(entries);

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

    println!();
    println!("CWN Universal Packer");
    println!("────────────────────────────────────────");
    println!("Output:          {}", output.display());
    println!("Entries:         {}", manifest.entries.len());
    println!("Original size:   {} bytes", original_size);
    println!("Container size:  {} bytes", packed_size);
    println!("Format:          CWN v1");
    println!("Compression:     Zstandard");
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

    let data_offset = writer.stream_position()?;

    let mut input = File::open(source)?;

    zstd::stream::copy_encode(&mut input, &mut *writer, level)
        .with_context(|| format!("failed to compress {}", source.display()))?;

    writer.flush()?;

    let end_offset = writer.stream_position()?;

    let packed_size = end_offset
        .checked_sub(data_offset)
        .context("invalid compressed size")?;

    println!(
        "[PACK] {}  {} -> {} bytes",
        archive_path, original_size, packed_size
    );

    entries.push(CwnEntry {
        path: archive_path,
        kind: EntryKind::File,
        original_size,
        packed_size,
        data_offset,
        compression: "zstd".to_string(),
        sha256: Some(sha256),
    });

    Ok(())
}
