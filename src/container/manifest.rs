use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackageMetadata {
    pub name: String,
    pub version: String,
    pub publisher: String,
}

impl PackageMetadata {
    pub fn new(
        name: impl Into<String>,
        version: impl Into<String>,
        publisher: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            version: version.into(),
            publisher: publisher.into(),
        }
    }

    pub fn validate(self) -> anyhow::Result<Self> {
        let name = self.name.trim().to_string();
        let version = self.version.trim().to_string();
        let publisher = self.publisher.trim().to_string();

        validate_metadata_field("package name", &name, 128)?;
        validate_metadata_field("package version", &version, 64)?;
        validate_metadata_field("publisher", &publisher, 128)?;

        Ok(Self {
            name,
            version,
            publisher,
        })
    }
}

fn validate_metadata_field(field: &str, value: &str, max_chars: usize) -> anyhow::Result<()> {
    if value.is_empty() {
        anyhow::bail!("{field} cannot be empty");
    }

    if value.chars().count() > max_chars {
        anyhow::bail!("{field} cannot exceed {max_chars} characters");
    }

    if value.chars().any(char::is_control) {
        anyhow::bail!("{field} cannot contain control characters");
    }

    Ok(())
}

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
        Self::with_metadata(
            entries,
            PackageMetadata::new(
                package_name,
                env!("CARGO_PKG_VERSION"),
                "Community Watch Network",
            ),
        )
    }

    pub fn with_metadata(entries: Vec<CwnEntry>, metadata: PackageMetadata) -> Self {
        Self {
            format: "CWN".to_string(),
            version: 1,
            producer: format!("CWN Universal Packer {}", env!("CARGO_PKG_VERSION")),
            package_name: metadata.name,
            package_version: metadata.version,
            publisher: metadata.publisher,
            entries,
        }
    }
}
