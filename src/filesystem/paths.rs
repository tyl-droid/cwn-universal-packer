use anyhow::{Result, bail};
use std::path::{Component, Path, PathBuf};

pub fn safe_relative_path(path: &str) -> Result<PathBuf> {
    let path = Path::new(path);

    if path.is_absolute() {
        bail!("absolute archive path rejected: {}", path.display());
    }

    let mut output = PathBuf::new();

    for component in path.components() {
        match component {
            Component::Normal(part) => {
                output.push(part);
            }

            Component::CurDir => {}

            Component::ParentDir => {
                bail!("parent traversal rejected: {}", path.display());
            }

            Component::RootDir | Component::Prefix(_) => {
                bail!("unsafe archive path rejected: {}", path.display());
            }
        }
    }

    if output.as_os_str().is_empty() {
        bail!("empty archive path");
    }

    Ok(output)
}
