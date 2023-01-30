#![deny(clippy::pedantic, clippy::cargo)]

use futures_util::StreamExt;
use std::error::Error;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::{env, fs};

use flate2::read::GzDecoder;
use npm::NpmResolvedPackage;
use reqwest::Url;
use tar::Archive;

mod npm;

const TEMP_FOLDER: &str = ".fpm";
const DEPS_FOLDER: &str = "node_modules";

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    fs::remove_dir_all(TEMP_FOLDER);
    fs::remove_dir_all(DEPS_FOLDER);
    fs::create_dir_all(TEMP_FOLDER);
    fs::create_dir_all(DEPS_FOLDER);

    let mut args = env::args();
    args.next();

    let package_name = args.next().unwrap_or(String::from("is-number"));

    let package_url = format!("https://registry.npmjs.org/{package_name}");
    println!("fetching {package_url}...");

    let response = reqwest::get(package_url).await?;
    let package: NpmResolvedPackage = response.json().await?;

    let latest = package
        .dist_tags
        .get("latest")
        .expect("expected to find latest tag");
    let version = package
        .versions
        .get(latest)
        .expect("expected to find version matching latest tag");
    let tarball = &version.dist.tarball;

    let file_name = Url::parse(&tarball)?
        .path_segments()
        .and_then(|segments| segments.last())
        .expect("failed to parse filename from tarball Url")
        .to_string();

    let tar_content = reqwest::get(tarball).await?;
    let tarball_path = Path::new(&TEMP_FOLDER).join(&file_name);
    let mut tarball_file = File::create(&tarball_path)?;

    let mut stream = tar_content.bytes_stream();

    while let Some(chunk) = stream.next().await {
        tarball_file.write_all(&chunk?)?;
    }

    let tar_gz_file = File::open(&tarball_path)?;
    let tar_file = GzDecoder::new(tar_gz_file);
    let mut archive = Archive::new(tar_file);

    let deps_dest = Path::new(DEPS_FOLDER).join(&package.name);

    fs::create_dir_all(&deps_dest)?;

    for file in archive.entries().unwrap() {
        let mut file = file.unwrap();

        println!("file path: {}", file.path().unwrap().to_str().unwrap());
        let file_path = file.path().unwrap();
        let file_path = file_path.strip_prefix("package")?;
        println!("file path: {}", file_path.to_str().unwrap());
        file.unpack(deps_dest.join(file_path))?;
    }

    Ok(())
}
