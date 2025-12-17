use greentic_config_types::PathsConfig;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use anyhow::{Context, Result, anyhow, bail};
use url::Url;

/// Environment-driven configuration for pack management.
#[derive(Debug, Clone)]
pub struct PackConfig {
    pub source: PackSource,
    pub index_location: IndexLocation,
    pub cache_dir: PathBuf,
    pub public_key: Option<String>,
    pub network: Option<greentic_config_types::NetworkConfig>,
}

impl PackConfig {
    /// Build a [`PackConfig`] using greentic-config paths and sensible defaults.
    pub fn default_for_paths(paths: &PathsConfig) -> Result<Self> {
        let cache_dir = paths.cache_dir.join("packs");
        let default_index = paths.greentic_root.join("index.json");
        let index_location = if default_index.exists() {
            IndexLocation::File(default_index)
        } else if Path::new("examples/index.json").exists() {
            IndexLocation::File(PathBuf::from("examples/index.json"))
        } else {
            IndexLocation::File(default_index)
        };
        Ok(Self {
            source: PackSource::Fs,
            index_location,
            cache_dir,
            public_key: None,
            network: None,
        })
    }

    /// Build from the structured packs section of greentic-config.
    pub fn from_packs(cfg: &greentic_config_types::PacksConfig) -> Result<Self> {
        let index_location = match &cfg.source {
            greentic_config_types::PackSourceConfig::LocalIndex { path } => {
                IndexLocation::File(path.clone())
            }
            greentic_config_types::PackSourceConfig::HttpIndex { url } => {
                IndexLocation::from_value(url)?
            }
            greentic_config_types::PackSourceConfig::OciRegistry { reference } => {
                IndexLocation::from_value(reference)?
            }
        };
        let public_key = cfg
            .trust
            .as_ref()
            .and_then(|trust| trust.public_keys.first().cloned());
        Ok(Self {
            source: PackSource::Fs,
            index_location,
            cache_dir: cfg.cache_dir.clone(),
            public_key,
            network: None,
        })
    }
}

/// Location of the pack index document (supports file paths and HTTP/S URLs).
#[derive(Debug, Clone)]
pub enum IndexLocation {
    File(PathBuf),
    Remote(Url),
}

impl IndexLocation {
    pub fn from_value(value: &str) -> Result<Self> {
        if value.starts_with("http://") || value.starts_with("https://") {
            let url = Url::parse(value).context("PACK_INDEX_URL is not a valid URL")?;
            return Ok(Self::Remote(url));
        }
        if value.starts_with("file://") {
            let url = Url::parse(value).context("PACK_INDEX_URL is not a valid file:// URL")?;
            let path = url
                .to_file_path()
                .map_err(|_| anyhow!("PACK_INDEX_URL points to an invalid file URI"))?;
            return Ok(Self::File(path));
        }
        Ok(Self::File(PathBuf::from(value)))
    }

    pub fn display(&self) -> String {
        match self {
            Self::File(path) => path.display().to_string(),
            Self::Remote(url) => url.to_string(),
        }
    }
}

/// Supported default sources for packs when the index omits the URI scheme.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PackSource {
    Fs,
    Http,
    Oci,
    S3,
    Gcs,
    AzBlob,
}

impl PackSource {
    pub fn scheme(self) -> &'static str {
        match self {
            Self::Fs => "fs",
            Self::Http => "http",
            Self::Oci => "oci",
            Self::S3 => "s3",
            Self::Gcs => "gcs",
            Self::AzBlob => "azblob",
        }
    }
}

impl FromStr for PackSource {
    type Err = anyhow::Error;

    fn from_str(value: &str) -> Result<Self> {
        match value.to_ascii_lowercase().as_str() {
            "fs" => Ok(Self::Fs),
            "http" | "https" => Ok(Self::Http),
            "oci" => Ok(Self::Oci),
            "s3" => Ok(Self::S3),
            "gcs" => Ok(Self::Gcs),
            "azblob" | "azure" | "azureblob" => Ok(Self::AzBlob),
            other => bail!("unsupported PACK_SOURCE `{other}`"),
        }
    }
}
