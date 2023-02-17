#![deny(clippy::pedantic, clippy::cargo)]

use std::{env, error::Error, fs};

use fast_package_manager::{
    dependency_resolver::resolve_deps,
    npm::VersionRangeSpecifier,
    package_fetcher::{DEPS_FOLDER, TEMP_FOLDER},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // fs::remove_dir_all(TEMP_FOLDER);
    // fs::remove_dir_all(DEPS_FOLDER);
    // fs::create_dir_all(TEMP_FOLDER);
    // fs::create_dir_all(DEPS_FOLDER);

    // let mut args = env::args();
    // args.next();

    // let package_name = args.next().unwrap_or(String::from("is-even"));

    // install_package(&package_name).await?;

    let resolved = resolve_deps(
        String::from("create-react-app"),
        VersionRangeSpecifier::new(String::from("latest")),
    )
    .await?;

    println!("{:#?}", serde_json::to_string(&resolved).unwrap());

    Ok(())
}
