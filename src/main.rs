#![deny(clippy::pedantic, clippy::cargo)]

use std::{collections::HashMap, env, error::Error, fs};

use fast_package_manager::{
    install_package::install_package, npm::VersionRangeSpecifier, DEPS_FOLDER, STORE_FOLDER,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut args = env::args();
    args.next();

    let mut packages = HashMap::new();
    while let Some(package_name) = args.next() {
        packages.insert(
            package_name,
            VersionRangeSpecifier::new(String::from("latest")),
        );
    }

    // loop {
    fs::remove_dir_all(STORE_FOLDER);
    fs::remove_dir_all(DEPS_FOLDER);
    fs::create_dir_all(STORE_FOLDER);
    fs::create_dir_all(DEPS_FOLDER);

    install_package(packages).await?;
    // }

    Ok(())
}
