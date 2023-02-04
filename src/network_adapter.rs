use std::error::Error;

use async_trait::async_trait;
use mockall::automock;

use crate::npm::UrlString;

#[automock]
#[async_trait]
pub trait NetworkAdapter {
    async fn get_package(&self, name: &String) -> Result<String, Box<dyn Error>>;
    async fn get_package_tar(
        &self,
        tarball_url: &UrlString,
    ) -> Result<reqwest::Response, reqwest::Error>;
}
