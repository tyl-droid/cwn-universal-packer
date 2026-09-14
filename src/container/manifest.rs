use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct CwnManifest {
    pub format: String,
    pub version: u16,

    #[serde(default = "default_producer")]
    pub producer: String,

    #[serde(default = "default_package_name")]
    pub package_name: String,

    #[serde(default = "default_package_version")]
    pub package_version: String,

    #[serde(default = "default_publisher")]
    pub publisher: String,

    pub entries: Vec<CwnEntry>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CwnEntry {
    pub path: String,
    pub kind: EntryKind,

    #[serde(default = "default_file_type")]
    pub file_type: String,

    pub original_size: u64,
    pub packed_size: u64,

    pub data_offset: u64,

    pub compression: String,

    pub sha256: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum EntryKind {
    File,
    Directory,
}

fn default_file_type() -> String {
    "Binary / Other".to_string()
}

fn default_producer() -> String {
    "CWN Universal Packer".to_string()
}

fn default_package_name() -> String {
    "Legacy CWN Package".to_string()
}

fn default_package_version() -> String {
    "unknown".to_string()
}

fn default_publisher() -> String {
    "Community Watch Network".to_string()
}

impl CwnManifest {
    pub fn new(entries: Vec<CwnEntry>, package_name: String) -> Self {
        Self {
            format: "CWN".to_string(),
            version: 1,
            producer: format!("CWN Universal Packer {}", env!("CARGO_PKG_VERSION")),
            package_name,
            package_version: env!("CARGO_PKG_VERSION").to_string(),
            publisher: "Community Watch Network".to_string(),
            entries,
        }
    }
}
