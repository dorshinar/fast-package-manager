use std::{
    collections::HashSet,
    error,
    io::ErrorKind,
    path::{Path, PathBuf},
};

use async_compression::tokio::bufread::GzipDecoder;
use derive_more::Display;
use futures::{future::join_all, TryStreamExt};
use tar::Archive;
use tokio::{
    fs,
    io::{self, BufReader},
    task,
};
use tokio_util::{compat::FuturesAsyncReadCompatExt, io::SyncIoBridge};

use crate::{
    config::Config,
    http::get_package_tar,
    npm::{ResolvedDependencies, UrlString, Version},
    STORE_FOLDER,
};

#[derive(Debug, Display, PartialEq)]
pub enum Error {
    UnpackError,
}

impl error::Error for Error {}

/// download packages to store.
/// returns the top level package, if specified.
pub async fn download_packages(
    packages: &Vec<ResolvedDependencies>,
    config: &Config,
) -> anyhow::Result<Vec<ResolvedDependencies>> {
    let mut top_level = vec![];

    let mut futures = Vec::new();
    let mut downloaded = HashSet::new();
    for dep in packages.iter() {
        if downloaded.contains(&dep.version.dist.tarball) {
            continue;
        }

        if dep.is_root {
            top_level.push(dep.to_owned());
        }

        futures.push(download_package_to_store(
            dep.version.name.clone(),
            dep.version.version.clone(),
            dep.version.dist.tarball.clone(),
            config,
        ));

        downloaded.insert(&dep.version.dist.tarball);
    }
    let _results = join_all(futures).await;
    println!("download to store: ");
    println!(
        "{:?}",
        _results
            .iter()
            .filter(|res| res.is_err())
            .collect::<Vec<_>>()
    );

    Ok(top_level)
}

/// download a single package to store.
pub async fn download_package_to_store(
    package_name: String,
    version: Version,
    tar: UrlString,
    config: &Config,
) -> anyhow::Result<()> {
    let tar_content = get_package_tar(&tar, config).await.unwrap();

    let deps_dest = get_store_package_path(&package_name, &version);

    let tgz = GzipDecoder::new(
        tar_content
            .bytes_stream()
            .map_err(|e| io::Error::new(ErrorKind::Other, e))
            .into_async_read()
            .compat(),
    );

    fs::create_dir_all(&deps_dest).await.unwrap();

    let mut extracted = HashSet::new();

    task::spawn_blocking(move || {
        let mut archive = Archive::new(SyncIoBridge::new(BufReader::new(tgz)));
        let mut entries = archive.entries().unwrap();
        while let Some(file) = &mut entries.next() {
            match file {
                Ok(file) => {
                    let file_path = file.path().unwrap();
                    let file_path = match file_path.strip_prefix("package") {
                        Ok(path) => path.to_path_buf(),
                        Err(_) => file_path.to_path_buf(),
                    };

                    if extracted.contains(&file_path) {
                        continue;
                    }

                    extracted.insert(file_path.to_owned());

                    if let Some(parent) = file_path.parent() {
                        match std::fs::create_dir_all(deps_dest.join(parent)) {
                            Err(error) if error.kind() == ErrorKind::AlreadyExists => {}
                            Err(error) => return Err(error),
                            _ => {}
                        }
                    }

                    let dst = deps_dest.join(file_path);
                    match file.unpack(dst) {
                        Err(error) => {
                            let formatted = format!("{:?}", error);
                            println!("{formatted}");
                            panic!("{:?}", error);
                        }
                        Ok(_) => {}
                    }
                }
                Err(error) => {
                    println!(
                        "{}, {}, {}, {}",
                        &package_name,
                        &version,
                        &tar,
                        &deps_dest.to_str().unwrap_or("failed deps_dest"),
                    );
                    panic!("{:?}", error);
                }
            }
        }

        Ok(())
    })
    .await??;

    Ok(())
}

pub fn get_store_package_path(package_name: &String, version: &Version) -> PathBuf {
    Path::new(STORE_FOLDER).join(format!("{}@{}", &package_name, &version))
}
