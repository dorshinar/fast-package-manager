#![deny(clippy::pedantic, clippy::cargo)]

use std::{env, error::Error, fs};

use fast_package_manager::{
    get_deps_with_versions, get_package, get_tar, resolve_version_from_tag, DEPS_FOLDER,
    TEMP_FOLDER,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    fs::remove_dir_all(TEMP_FOLDER);
    fs::remove_dir_all(DEPS_FOLDER);
    fs::create_dir_all(TEMP_FOLDER);
    fs::create_dir_all(DEPS_FOLDER);

    let mut args = env::args();
    args.next();

    let package_name = args.next().unwrap_or(String::from("is-even"));

    let package = get_package(&package_name).await?;

    let version = resolve_version_from_tag(&package, &String::from("latest")).unwrap();
    let tarball = &version.dist.tarball;

    let _deps = get_deps_with_versions(&version.dependencies).await;

    get_tar(&package_name, &tarball).await?;

    Ok(())
}
