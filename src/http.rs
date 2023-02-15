use derive_more::Display;

use crate::npm::{NpmResolvedPackage, UrlString};

const NPM_REGISTRY_URL: &str = "https://registry.npmjs.org/";
const INSTALL_FETCH_HEADER: &str =
    "application/vnd.npm.install-v1+json; q=1.0, application/json; q=0.8, */*";

#[derive(Debug, Display, derive_more::Error)]
pub enum Error {
    HttpError,
}

pub async fn get_npm_package(name: String) -> Result<NpmResolvedPackage, Error> {
    let package_url = format!("{NPM_REGISTRY_URL}{name}");
    println!("fetching {package_url}...");

    let response = match reqwest::Client::new()
        .get(package_url)
        .header(reqwest::header::ACCEPT, INSTALL_FETCH_HEADER)
        .send()
        .await
    {
        Ok(response) => response,
        Err(_) => return Err(Error::HttpError),
    };

    let response = match response.text().await {
        Ok(response) => response,
        Err(_) => return Err(Error::HttpError),
    };

    let resolved = match serde_json::from_str(response.as_str()) {
        Ok(response) => response,
        Err(_) => return Err(Error::HttpError),
    };
    Ok(resolved)
}

pub async fn get_package_tar(tarball: &UrlString) -> Result<reqwest::Response, reqwest::Error> {
    Ok(reqwest::Client::new().get(tarball.as_str()).send().await?)
}
// }
