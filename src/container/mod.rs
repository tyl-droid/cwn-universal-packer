pub mod header;
pub mod manifest;

use anyhow::{Context, Result, bail};
use header::CwnHeader;
use manifest::{CwnManifest, EntryKind};

use std::collections::HashSet;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

pub fn read_manifest(path: &Path) -> Result<(CwnHeader, CwnManifest)> {
    let mut file =
        File::open(path).with_context(|| format!("failed to open {}", path.display()))?;

    let size = file.metadata()?.len();

    let header = CwnHeader::read(&mut file)?;

    let manifest_end = header
        .manifest_offset
        .checked_add(header.manifest_size)
        .context("manifest size overflow")?;

    if manifest_end > size {
        bail!("CWN manifest points outside the container");
    }

    file.seek(SeekFrom::Start(header.manifest_offset))?;

    let mut data = vec![0u8; header.manifest_size as usize];
    file.read_exact(&mut data)?;

    let manifest: CwnManifest = serde_json::from_slice(&data).context("invalid CWN manifest")?;

    if manifest.entries.len() as u64 != header.entry_count {
        bail!(
            "manifest entry count mismatch: header={}, manifest={}",
            header.entry_count,
            manifest.entries.len()
        );
    }

    validate_manifest(&header, &manifest, size)?;

    Ok((header, manifest))
}

fn validate_manifest(
    header: &CwnHeader,
    manifest: &CwnManifest,
    container_size: u64,
) -> Result<()> {
    let mut paths = HashSet::new();
    let mut ranges = Vec::new();

    for entry in &manifest.entries {
        if !paths.insert(entry.path.clone()) {
            bail!("duplicate archive path: {}", entry.path);
        }

        match entry.kind {
            EntryKind::Directory => {
                if entry.original_size != 0 || entry.packed_size != 0 || entry.data_offset != 0 {
                    bail!(
                        "directory entry contains invalid payload metadata: {}",
                        entry.path
                    );
                }
            }

            EntryKind::File => {
                if entry.sha256.is_none() {
                    bail!("file entry missing SHA-256: {}", entry.path);
                }

                let sha256 = entry.sha256.as_ref().unwrap();

                if sha256.len() != 64 || !sha256.bytes().all(|b| b.is_ascii_hexdigit()) {
                    bail!("invalid SHA-256 metadata for {}", entry.path);
                }

                match entry.compression.as_str() {
                    "none" | "zstd" => {}
                    other => {
                        bail!("unsupported compression '{}' for {}", other, entry.path);
                    }
                }

                let end = entry
                    .data_offset
                    .checked_add(entry.packed_size)
                    .context("payload range overflow")?;

                if end > container_size {
                    bail!("payload extends outside container: {}", entry.path);
                }

                if end > header.manifest_offset {
                    bail!("payload overlaps CWN manifest: {}", entry.path);
                }

                ranges.push((entry.data_offset, end, entry.path.as_str()));
            }
        }
    }

    ranges.sort_by_key(|range| range.0);

    for pair in ranges.windows(2) {
        let (_, previous_end, previous_path) = pair[0];
        let (next_start, _, next_path) = pair[1];

        if previous_end > next_start {
            bail!(
                "payload overlap detected between '{}' and '{}'",
                previous_path,
                next_path
            );
        }
    }

    Ok(())
}
