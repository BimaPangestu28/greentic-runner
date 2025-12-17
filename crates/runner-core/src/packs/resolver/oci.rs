use anyhow::Result;
use greentic_config_types::NetworkConfig;

use super::{FetchResponse, HttpResolver, PackResolver};

pub struct OciResolver {
    inner: HttpResolver,
}

impl OciResolver {
    pub fn new(network: Option<&NetworkConfig>) -> Result<Self> {
        Ok(Self {
            inner: HttpResolver::new("oci", network)?,
        })
    }
}

impl PackResolver for OciResolver {
    fn scheme(&self) -> &'static str {
        "oci"
    }

    fn fetch(&self, locator: &str) -> Result<FetchResponse> {
        self.inner.fetch(locator)
    }
}
