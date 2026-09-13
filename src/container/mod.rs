pub mod header;
pub mod manifest;

use anyhow::{Context, Result, bail};
use header::CwnHeader;
use manifest::CwnManifest;

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

    Ok((header, manifest))
}
