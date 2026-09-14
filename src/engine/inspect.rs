use crate::container;
use crate::container::manifest::EntryKind;
use crate::engine::types::{ContainerEntryInfo, ContainerInfo};

use anyhow::Result;
use std::fs;
use std::path::Path;

pub fn inspect_container(path: &Path) -> Result<ContainerInfo> {
    let (header, manifest) = container::read_manifest(path)?;

    let container_size = fs::metadata(path)?.len();

    let mut files = 0usize;
    let mut directories = 0usize;

    let mut original_size = 0u64;
    let mut payload_size = 0u64;

    let mut zstd_files = 0usize;
    let mut stored_files = 0usize;

    let mut entries = Vec::with_capacity(manifest.entries.len());

    for entry in manifest.entries {
        let is_directory = entry.kind == EntryKind::Directory;

        if is_directory {
            directories += 1;
        } else {
            files += 1;

            original_size += entry.original_size;
            payload_size += entry.packed_size;

            match entry.compression.as_str() {
                "zstd" => zstd_files += 1,
                "none" => stored_files += 1,
                _ => {}
            }
        }

        entries.push(ContainerEntryInfo {
            path: entry.path,
            file_type: entry.file_type,
            original_size: entry.original_size,
            packed_size: entry.packed_size,
            compression: entry.compression,
            is_directory,
        });
    }

    Ok(ContainerInfo {
        path: path.to_path_buf(),

        format_version: header.version,

        package_name: manifest.package_name,
        package_version: manifest.package_version,
        publisher: manifest.publisher,
        producer: manifest.producer,

        files,
        directories,

        original_size,
        payload_size,
        container_size,

        zstd_files,
        stored_files,

        entries,
    })
}
