use flate2::read::GzDecoder;
use futures::future::join_all;
use futures_util::StreamExt;
use npm::{NpmPackage, NpmPackageVersion, NpmResolvedPackage, VersionRangeSpecifier};
use reqwest::Url;

use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use tar::Archive;

pub mod npm;

pub const TEMP_FOLDER: &str = ".fpm";
pub const DEPS_FOLDER: &str = "node_modules";

pub async fn get_tar(package_name: &str, tarball: &str) -> Result<(), Box<dyn Error>> {
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

    let deps_dest = Path::new(DEPS_FOLDER).join(&package_name);

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

pub async fn get_deps_with_versions(
    deps: &Option<HashMap<String, VersionRangeSpecifier>>,
) -> Vec<(String, String)> {
    println!("getting deps: {:?}", deps);

    if deps.is_none() {
        return vec![];
    }

    let deps = deps.as_ref().unwrap();

    let mut package_fetched = HashSet::new();
    let mut futures = vec![];

    for (dep, _) in deps.iter() {
        if package_fetched.contains(dep) {
            continue;
        }

        futures.push(get_package(dep));
        package_fetched.insert(dep);
    }

    let packages: Vec<(&String, &NpmPackageVersion)> = join_all(futures)
        .await
        .iter()
        .map(|package| package.as_ref().unwrap())
        .map(|package| {
            (
                &package.parsed.name,
                resolve_version_from_tag(&package, deps.get(&package.parsed.name).unwrap()),
            )
        })
        .filter(|tup| tup.1.is_some())
        .map(|tup| (tup.0, tup.1.unwrap()))
        .collect();

    // println!("{:?}", packages);

    vec![]
}

pub async fn get_package(name: &str) -> Result<NpmPackage, Box<dyn Error>> {
    let package_url = format!("https://registry.npmjs.org/{name}");
    println!("fetching {package_url}...");

    let response = reqwest::get(package_url).await?;
    let resolved = response.text().await?;

    Ok(NpmPackage {
        json: serde_json::from_str(&resolved)?,
        parsed: serde_json::from_str(&resolved)?,
    })
}

pub fn resolve_version_from_tag<'a>(
    package: &'a NpmPackage,
    version: &VersionRangeSpecifier,
) -> Option<&'a NpmPackageVersion> {
    if version == "latest" {
        let latest = package
            .parsed
            .dist_tags
            .get("latest")
            .expect("expected to find latest tag");
        return package.parsed.versions.get(latest);
    }

    // println!("{:?}", serde_json::to_string(&package.parsed.versions));

    let version_req = semver::VersionReq::parse(&version).unwrap();
    let raw_versions = package.json.get("versions");

    match raw_versions {
        Some(serde_json::Value::Object(versions)) => {
            for vrs in versions.into_iter().rev() {
                println!("{:?}", serde_json::to_string(&vrs.0));
            }
        }
        _ => (),
    }

    // for version_value in package.json.get("version").iter().rev() {
    //     if version_req.matches(&semver::Version::parse(&version_tag).unwrap()) {
    //         return Some(&version_content);
    //     }
    // }

    None
}
