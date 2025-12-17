use anyhow::Result;
use greentic_config_types::NetworkConfig;

use super::{FetchResponse, HttpResolver, PackResolver};

pub struct GcsResolver {
    inner: HttpResolver,
}

impl GcsResolver {
    pub fn new(network: Option<&NetworkConfig>) -> Result<Self> {
        Ok(Self {
            inner: HttpResolver::new("gcs", network)?,
        })
    }
}

impl PackResolver for GcsResolver {
    fn scheme(&self) -> &'static str {
        "gcs"
    }

    fn fetch(&self, locator: &str) -> Result<FetchResponse> {
        self.inner.fetch(locator)
    }
}
