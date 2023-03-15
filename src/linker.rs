use std::{
    io::{ErrorKind, Result},
    path::{Path, PathBuf},
};
use tokio::task;

use crate::{downloader::get_store_package_path, npm::Version, DEPS_FOLDER, STORE_FOLDER};

pub async fn symlink_dep(
    dep_name: &String,
    dep_version: &Version,
    dest_name: &String,
    dest_version: &Version,
) -> anyhow::Result<()> {
    let original = get_dep_symlink_path(&dep_name, dep_version);

    let link = get_local_store_package_path(dest_name, dest_version);
    let mut parent = link
        .parent()
        .expect("failed to get package folder")
        .to_path_buf();
    if dest_name.starts_with("@") {
        parent = parent
            .parent()
            .expect("failed to get package folder")
            .to_path_buf();
    }

    parent = parent.join(&dep_name);

    let name = dep_name.clone();

    task::spawn_blocking(
        move || match std::os::unix::fs::symlink(&original, &parent) {
            Err(error) if error.kind() == ErrorKind::AlreadyExists => Ok(()),
            Err(error) => {
                // TODO: check what pnpm does with @floating-ui/react-dom
                // I bet they convert the / to something else like _

                println!("{:?}, {:?}, {:?}, {:?}", error, original, parent, name);

                return Err(error.into());
            }
            Ok(_) => Ok(()),
        },
    )
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

    task::spawn_blocking(|| std::os::unix::fs::symlink(original, link)).await?
}

fn get_dep_symlink_path(name: &String, version: &Version) -> PathBuf {
    let folder_name = if name.starts_with("@") {
        name.replace("/", "+")
    } else {
        name.to_string()
    };

    Path::new("..")
        .join("..")
        .join(format!("{}@{}", folder_name, version))
        .join(DEPS_FOLDER)
        .join(name)
}

pub fn get_local_store_package_path(package_name: &String, version: &Version) -> PathBuf {
    let folder_name = if package_name.starts_with("@") {
        package_name.replace("/", "+")
    } else {
        package_name.to_string()
    };

    Path::new(DEPS_FOLDER)
        .join(STORE_FOLDER)
        .join(format!("{}@{}", folder_name, &version))
        .join("node_modules")
        .join(&package_name)
}

/// Hardlink all files form `source` recursively into `dest`.
pub async fn hardlink_package(package_name: &String, version: &Version) -> anyhow::Result<()> {
    let original = get_store_package_path(package_name, version);

    let link = get_local_store_package_path(package_name, version);

    if let Some(parent) = link.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }

    task::spawn_blocking(|| hardlink(original, link)).await??;

    Ok(())
}

fn hardlink(source: PathBuf, dest: PathBuf) -> anyhow::Result<()> {
    let files = std::fs::read_dir(source)?;

    for file in files {
        if let Ok(file) = file {
            if let Ok(file_type) = file.file_type() {
                if file_type.is_dir() && file.file_name() != "node_modules" {
                    let sub_dir = dest.join(&file.file_name());

                    match std::fs::create_dir_all(&sub_dir) {
                        Err(error) if error.kind() != ErrorKind::AlreadyExists => {
                            return Err(error.into())
                        }
                        _ => {}
                    }

                    match hardlink(file.path().clone(), sub_dir) {
                        Err(error) if error.downcast_ref() == Some(&ErrorKind::AlreadyExists) => {
                            return Err(error)
                        }
                        _ => {}
                    }
                } else if file_type.is_file() {
                    match std::fs::create_dir_all(&dest) {
                        Err(error) if error.kind() != ErrorKind::AlreadyExists => {
                            return Err(error.into())
                        }
                        _ => {}
                    }

                    match std::fs::hard_link(file.path(), dest.join(file.file_name())) {
                        Err(error) if error.kind() != ErrorKind::AlreadyExists => {
                            return Err(error.into())
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn symlink_path_no_scope() {
        let path =
            get_dep_symlink_path(&String::from("react"), &Version::new(String::from("1.0.0")));

        assert_eq!(
            path.to_str().unwrap().to_string(),
            format!("../../react@1.0.0/node_modules/react")
        )
    }

    #[test]
    fn symlink_path_with_scope() {
        let path = get_dep_symlink_path(
            &String::from("@react/dom"),
            &Version::new(String::from("1.0.0")),
        );

        assert_eq!(
            path.to_str().unwrap().to_string(),
            format!("../../@react+dom@1.0.0/node_modules/@react/dom")
        )
    }

    #[test]
    fn local_store_path_no_scope() {
        let path = get_local_store_package_path(
            &String::from("react"),
            &Version::new(String::from("1.0.0")),
        );

        assert_eq!(
            path.to_str().unwrap().to_string(),
            format!("node_modules/.fpm/react@1.0.0/node_modules/react")
        )
    }

    #[test]
    fn local_store_path_with_scope() {
        let path = get_local_store_package_path(
            &String::from("@react/dom"),
            &Version::new(String::from("1.0.0")),
        );

        assert_eq!(
            path.to_str().unwrap().to_string(),
            format!("node_modules/.fpm/@react+dom@1.0.0/node_modules/@react/dom")
        )
    }
}
