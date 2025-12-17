use anyhow::Result;
use greentic_config_types::NetworkConfig;

use super::{FetchResponse, HttpResolver, PackResolver};

pub struct AzBlobResolver {
    inner: HttpResolver,
}

impl AzBlobResolver {
    pub fn new(network: Option<&NetworkConfig>) -> Result<Self> {
        Ok(Self {
            inner: HttpResolver::new("azblob", network)?,
        })
    }
}

impl PackResolver for AzBlobResolver {
    fn scheme(&self) -> &'static str {
        "azblob"
    }

    fn fetch(&self, locator: &str) -> Result<FetchResponse> {
        self.inner.fetch(locator)
    }
}
