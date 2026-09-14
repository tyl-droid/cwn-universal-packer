use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct CwnManifest {
    pub format: String,
    pub version: u16,
    pub producer: String,
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

impl CwnManifest {
    pub fn new(entries: Vec<CwnEntry>) -> Self {
        Self {
            format: "CWN".to_string(),
            version: 1,
            producer: "Community Watch Network".to_string(),
            entries,
        }
    }
}
