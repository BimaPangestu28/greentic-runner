use anyhow::Result;
use greentic_config_types::NetworkConfig;

use super::{FetchResponse, HttpResolver, PackResolver};

pub struct S3Resolver {
    inner: HttpResolver,
}

impl S3Resolver {
    pub fn new(network: Option<&NetworkConfig>) -> Result<Self> {
        Ok(Self {
            inner: HttpResolver::new("s3", network)?,
        })
    }
}

impl PackResolver for S3Resolver {
    fn scheme(&self) -> &'static str {
        "s3"
    }

    fn fetch(&self, locator: &str) -> Result<FetchResponse> {
        self.inner.fetch(locator)
    }
}
