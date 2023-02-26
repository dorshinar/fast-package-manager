use std::error;

use derive_more::Display;

use crate::npm::{NpmResolvedPackage, UrlString};

const NPM_REGISTRY_URL: &str = "https://registry.npmjs.org/";
const INSTALL_FETCH_HEADER: &str =
    "application/vnd.npm.install-v1+json; q=1.0, application/json; q=0.8, */*";

#[derive(Debug, Display, derive_more::Error)]
pub enum Error {
    HttpError,
}

pub async fn get_npm_package(name: &String) -> Result<NpmResolvedPackage, Box<dyn error::Error>> {
    let package_url = format!("{NPM_REGISTRY_URL}{name}");

    let response = match reqwest::Client::new()
        .get(package_url)
        .header(reqwest::header::ACCEPT, INSTALL_FETCH_HEADER)
        .send()
        .await
    {
        Ok(response) => response,
        Err(_) => return Err(Box::new(Error::HttpError)),
    };

    match response.json().await {
        Ok(response) => {
            return Ok(response);
        }
        Err(error) => {
            println!("{:?}", error);
            return Err(Box::new(error));
        }
    }
}

pub async fn get_package_tar(tarball: &UrlString) -> Result<reqwest::Response, reqwest::Error> {
    Ok(reqwest::Client::new().get(tarball.as_str()).send().await?)
}
