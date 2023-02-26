use std::{error, io::ErrorKind, path::PathBuf};

use tokio::{fs, task};

use crate::{
    downloader::{get_local_store_package_path, get_store_package_path},
    npm::Version,
};

/// Hardlink all files form `source` recursively into `dest`.
pub async fn hardlink_package(
    package_name: &String,
    version: &Version,
) -> Result<(), Box<dyn error::Error>> {
    let original = get_store_package_path(package_name, version);

    let link = get_local_store_package_path(package_name, version);

    if let Some(parent) = link.parent() {
        fs::create_dir_all(parent).await?;
    }

    task::spawn_blocking(|| hardlink(original, link)).await??;

    Ok(())
}

fn hardlink(source: PathBuf, dest: PathBuf) -> Result<(), std::io::Error> {
    let files = std::fs::read_dir(source)?;

    for file in files {
        if let Ok(file) = file {
            if let Ok(file_type) = file.file_type() {
                if file_type.is_dir() && file.file_name() != "node_modules" {
                    let sub_dir = dest.join(&file.file_name());

                    match std::fs::create_dir_all(&sub_dir) {
                        Err(error) if error.kind() != ErrorKind::AlreadyExists => {
                            return Err(error)
                        }
                        _ => {}
                    }

                    match hardlink(file.path().clone(), sub_dir) {
                        Err(error) if error.kind() != ErrorKind::AlreadyExists => {
                            return Err(error)
                        }
                        _ => {}
                    }
                } else if file_type.is_file() {
                    match std::fs::create_dir_all(&dest) {
                        Err(error) if error.kind() != ErrorKind::AlreadyExists => {
                            return Err(error)
                        }
                        _ => {}
                    }

                    match std::fs::hard_link(file.path(), dest.join(file.file_name())) {
                        Err(error) if error.kind() != ErrorKind::AlreadyExists => {
                            return Err(error)
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    Ok(())
}
