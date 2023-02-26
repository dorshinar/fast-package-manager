#![deny(clippy::pedantic, clippy::cargo)]

use std::{env, error::Error, fs};

use fast_package_manager::{
    install_package::install_package, npm::VersionRangeSpecifier, DEPS_FOLDER, STORE_FOLDER,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut args = env::args();
    args.next();

    let package_name = args.next().unwrap_or(String::from("create-react-app"));

    // loop {
    fs::remove_dir_all(STORE_FOLDER);
    fs::remove_dir_all(DEPS_FOLDER);
    fs::create_dir_all(STORE_FOLDER);
    fs::create_dir_all(DEPS_FOLDER);

    install_package(
        package_name.clone(),
        VersionRangeSpecifier::new(String::from("latest")),
    )
    .await?;
    // }

    Ok(())
}
