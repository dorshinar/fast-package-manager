use std::str::FromStr;

use derive_more::Display;

use crate::{
    config::Config,
    npm::{NpmResolvedPackage, UrlString},
};

const NPM_REGISTRY_URL: &str = "registry.npmjs.org/";
const INSTALL_FETCH_HEADER: &str =
    "application/vnd.npm.install-v1+json; q=1.0, application/json; q=0.8, */*";

#[derive(Debug, Display, derive_more::Error)]
pub enum Error {
    HttpError,
}

pub async fn get_npm_package(name: &String, config: &Config) -> anyhow::Result<NpmResolvedPackage> {
    let package_url =
        reqwest::Url::from_str(format!("https://{NPM_REGISTRY_URL}").as_str())?.join(name)?;

    let response = match config
        .client
        .get(package_url.clone())
        .header(reqwest::header::ACCEPT, INSTALL_FETCH_HEADER)
        .send()
        .await
    {
        Ok(response) => response,
        Err(error) => {
            println!("Fetch: {name}, {package_url}, {:?}", error);
            return Err(Error::HttpError.into());
        }
    };

    let text = match response.text().await {
        Ok(response) => match serde_json::from_str(response.as_str()) {
            Ok(json) => Ok(json),
            Err(error) => {
                println!("{:?}", response);
                println!("JSON: {name}, {package_url}, {:?}", error,);
                return Err(error.into());
            }
        },
        Err(error) => {
            println!("TEXT: {name}, {package_url}, {:?}", error,);
            return Err(error.into());
        }
    };

    text
}

pub async fn get_package_tar(
    tarball: &UrlString,
    config: &Config,
) -> Result<reqwest::Response, reqwest::Error> {
    Ok(config.client.get(tarball.as_str()).send().await?)
}
