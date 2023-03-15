#![deny(clippy::pedantic, clippy::cargo)]

use fast_package_manager::{
    config::Config, install_manifest::install_manifest, install_package::install_package,
    npm::VersionRangeSpecifier, DEPS_FOLDER, STORE_FOLDER,
};
use std::{collections::HashMap, env, fs, io::ErrorKind};
use tokio::net::lookup_host;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut args = env::args();
    args.next();

    let mut packages = HashMap::new();
    while let Some(package_name) = args.next() {
        packages.insert(
            package_name,
            VersionRangeSpecifier::new(String::from("latest")),
        );
    }

    match fs::remove_dir_all(STORE_FOLDER) {
        Err(error) if error.kind() != ErrorKind::AlreadyExists => return Err(error.into()),
        _ => {}
    }
    match fs::remove_dir_all(DEPS_FOLDER) {
        Err(error) if error.kind() != ErrorKind::AlreadyExists => return Err(error.into()),
        _ => {}
    }
    match fs::create_dir_all(STORE_FOLDER) {
        Err(error) if error.kind() != ErrorKind::AlreadyExists => return Err(error.into()),
        _ => {}
    }
    match fs::create_dir_all(DEPS_FOLDER) {
        Err(error) if error.kind() != ErrorKind::AlreadyExists => return Err(error.into()),
        _ => {}
    }

    // let ip = lookup_host("registry.npmjs.org:443")
    //     .await?
    //     .next()
    //     .expect("failed to resolve registry.npmjs.org:443");
    // println!("registry ip: {:?}", ip);
    let client = reqwest::Client::builder()
        // .resolve("registry.npmjs.org", ip)
        // .danger_accept_invalid_certs(true)
        .build()
        .expect("failed to build reqwest client");

    let config = Config {
        client,
        // npm_registry_ip: ip,
    };

    // let package = &String::from("uuid");
    // let pac = get_npm_package(package, &config).await?;

    // println!("{}", serde_json::to_string_pretty(&pac).unwrap());
    if packages.len() == 0 {
        install_manifest(&config).await?;
    } else {
        install_package(packages, &config).await?;
    }

    Ok(())
}
