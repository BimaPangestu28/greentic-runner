use std::path::{Path, PathBuf};

use anyhow::{Context, Result, anyhow};
use url::Url;

use crate::path_safety::normalize_under_root;

use super::{FetchResponse, PackResolver};

#[derive(Debug)]
pub struct FsResolver {
    root: PathBuf,
}

impl FsResolver {
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    fn parse_path(&self, locator: &str) -> Result<PathBuf> {
        if let Some(stripped) = locator.strip_prefix("fs://") {
            if stripped.starts_with('/')
                || stripped.starts_with("./")
                || stripped.starts_with("../")
            {
                return Ok(PathBuf::from(stripped));
            }
            if cfg!(windows) && stripped.chars().nth(1) == Some(':') {
                return Ok(PathBuf::from(stripped));
            }
            let file_url = format!("file://{stripped}");
            let url = Url::parse(&file_url).context("failed to parse fs:// locator as file URL")?;
            return url
                .to_file_path()
                .map_err(|_| anyhow!("fs locator {locator} cannot be represented as a path"));
        }
        Ok(PathBuf::from(locator))
    }

    fn normalize(&self, path: PathBuf) -> Result<PathBuf> {
        if path.is_absolute() {
            let parent = path
                .parent()
                .ok_or_else(|| anyhow!("fs locator missing parent: {}", path.display()))?;
            let root = parent
                .canonicalize()
                .with_context(|| format!("failed to canonicalize {}", parent.display()))?;
            let file = path
                .file_name()
                .ok_or_else(|| anyhow!("fs locator missing file name: {}", path.display()))?;
            return normalize_under_root(&root, Path::new(file));
        }
        normalize_under_root(&self.root, &path)
    }
}

impl PackResolver for FsResolver {
    fn scheme(&self) -> &'static str {
        "fs"
    }

    fn fetch(&self, locator: &str) -> Result<FetchResponse> {
        let path = self.parse_path(locator)?;
        let path = self.normalize(path)?;
        if !path.exists() {
            anyhow::bail!("fs resolver: {} does not exist", path.display());
        }
        Ok(FetchResponse::from_path(path))
    }
}
