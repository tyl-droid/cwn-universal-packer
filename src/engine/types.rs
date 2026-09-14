use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct ContainerEntryInfo {
    pub path: String,
    pub file_type: String,
    pub original_size: u64,
    pub packed_size: u64,
    pub compression: String,
    pub is_directory: bool,
}

#[derive(Debug, Clone)]
pub struct ContainerInfo {
    pub path: PathBuf,

    pub format_version: u16,

    pub package_name: String,
    pub package_version: String,
    pub publisher: String,
    pub producer: String,

    pub files: usize,
    pub directories: usize,

    pub original_size: u64,
    pub payload_size: u64,
    pub container_size: u64,

    pub zstd_files: usize,
    pub stored_files: usize,

    pub entries: Vec<ContainerEntryInfo>,
}
