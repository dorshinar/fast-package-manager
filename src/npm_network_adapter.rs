use std::error::Error;

use async_trait::async_trait;

use crate::{network_adapter::NetworkAdapter, npm::UrlString};

const NPM_REGISTRY_URL: &str = "https://registry.npmjs.org/";
const INSTALL_FETCH_HEADER: &str =
    "application/vnd.npm.install-v1+json; q=1.0, application/json; q=0.8, */*";

pub struct NpmNetworkAdapter {
    client: reqwest::Client,
}

impl NpmNetworkAdapter {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl NetworkAdapter for NpmNetworkAdapter {
    async fn get_package(&self, name: &String) -> Result<String, Box<dyn Error>> {
        let package_url = format!("{NPM_REGISTRY_URL}{name}");
        println!("fetching {package_url}...");

        let response = self
            .client
            .get(package_url)
            .header(reqwest::header::ACCEPT, INSTALL_FETCH_HEADER)
            .send()
            .await?;
        Ok(response.text().await?)
    }

    async fn get_package_tar(
        &self,
        tarball: &UrlString,
    ) -> Result<reqwest::Response, reqwest::Error> {
        Ok(self.client.get(tarball.as_str()).send().await?)
    }
}
