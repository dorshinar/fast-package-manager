use std::{
    io::{ErrorKind, Result},
    os::unix::fs,
    path::{Path, PathBuf},
};

use tokio::task;

use crate::{downloader::get_local_store_package_path, npm::Version, DEPS_FOLDER, STORE_FOLDER};

pub async fn symlink_dep(
    dep_name: &String,
    dep_version: &Version,
    dest_name: &String,
    dest_version: &Version,
) -> Result<()> {
    let original = get_dep_symlink_path(dep_name, dep_version);

    let link = get_local_store_package_path(dest_name, dest_version)
        .parent()
        .expect("failed to get package folder")
        .join(dep_name);

    task::spawn_blocking(|| match fs::symlink(original, link) {
        Err(error) if error.kind() == ErrorKind::AlreadyExists => Ok(()),
        Err(error) => return Err(error),
        Ok(_) => Ok(()),
    })
    .await?
}

pub async fn symlink_direct(name: &String, version: &Version) -> Result<()> {
    let path_base = if name.starts_with("@") {
        Path::new("../")
    } else {
        Path::new(".")
    };

    let original = path_base
        .join(STORE_FOLDER)
        .join(format!("{}@{}", name, version))
        .join(DEPS_FOLDER)
        .join(name);

    let link = Path::new(DEPS_FOLDER).join(name);

    if let Some(parent) = link.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }

    task::spawn_blocking(|| fs::symlink(original, link)).await?
}

fn get_dep_symlink_path(name: &String, version: &Version) -> PathBuf {
    Path::new("..")
        .join("..")
        .join(format!("{}@{}", name, version))
        .join(DEPS_FOLDER)
        .join(name)
}
