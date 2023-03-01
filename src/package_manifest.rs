use std::{
    collections::HashMap,
    env,
    fs::{self, File},
    io::BufReader,
    path::Path,
};

use serde_json::Value;
use thiserror::Error;
use tokio::task;

use crate::npm::VersionRangeSpecifier;

#[derive(Error, Debug)]
pub enum ManifestError {
    #[error("Failed to locate manifest file")]
    ManifestNotFound,
}

pub async fn update_package_manifest(
    packages_to_add: HashMap<String, VersionRangeSpecifier>,
) -> anyhow::Result<()> {
    task::spawn_blocking(|| update_manifest(packages_to_add)).await??;

    Ok(())
}

fn update_manifest(packages_to_add: HashMap<String, VersionRangeSpecifier>) -> anyhow::Result<()> {
    // Read the JSON contents of the file as an instance of `User`.
    let mut package_json = get_manifest_file()?;

    match &mut package_json {
        Value::Object(package_json) => match package_json.get_mut("dependencies") {
            Some(deps) => match deps {
                Value::Object(deps_obj) => {
                    for (package, range) in packages_to_add {
                        deps_obj.insert(
                            package.to_owned(),
                            Value::String(range.to_owned().to_string()),
                        );
                    }
                }
                _ => (),
            },
            None => match serde_json::to_string(&packages_to_add) {
                Ok(deps) => {
                    package_json.insert(String::from("dependencies"), Value::String(deps));
                }
                Err(error) => panic!("{}", error),
            },
        },
        _ => panic!("failed to read package.json"),
    };

    fs::write(
        "./package.json",
        serde_json::to_string_pretty(&package_json).unwrap(),
    )?;

    Ok(())
}

fn get_manifest_file() -> anyhow::Result<Value> {
    let mut manifest_path = env::current_dir()?.join("package.json");

    while !manifest_path.exists() {
        if let Some(parent) = manifest_path.parent() {
            if let Some(parent) = parent.parent() {
                manifest_path = parent.join("package.json");
            } else {
                return Err(ManifestError::ManifestNotFound.into());
            }
        } else {
            return Err(ManifestError::ManifestNotFound.into());
        }
    }

    let file = File::open(manifest_path)?;
    let reader = BufReader::new(file);

    // Read the JSON contents of the file as an instance of `User`.
    let package_json: Value = serde_json::from_reader(reader)?;

    Ok(package_json)
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use serde_json::json;
    use tempfile::{tempdir, tempdir_in};

    use super::*;

    #[test]
    fn get_manifest_from_pwd() {
        let pwd = std::env::current_dir().unwrap();

        let file_content = json!({ "a": 1 });

        let temp_dir = tempdir().unwrap();
        std::env::set_current_dir(temp_dir.path()).unwrap();

        let mut manifest = File::create(temp_dir.path().join("package.json")).unwrap();

        manifest
            .write_all(file_content.to_string().as_bytes())
            .unwrap();

        let file = get_manifest_file().unwrap();

        assert_eq!(file, file_content);

        std::env::set_current_dir(pwd).unwrap();
    }

    #[test]
    fn get_manifest_from_nested() {
        let pwd = std::env::current_dir().unwrap();

        let file_content = json!({ "a": 1 });

        let temp_dir = tempdir().unwrap();
        let mut manifest = File::create(temp_dir.path().join("package.json")).unwrap();

        let nested = tempdir_in(&temp_dir).unwrap();
        std::env::set_current_dir(nested.path()).unwrap();

        manifest
            .write_all(file_content.to_string().as_bytes())
            .unwrap();

        let file = get_manifest_file().unwrap();

        assert_eq!(file, file_content);

        std::env::set_current_dir(pwd).unwrap();
    }

    #[test]
    fn get_manifest_from_deep_nested() {
        let pwd = std::env::current_dir().unwrap();

        let file_content = json!({ "a": 1 });

        let temp_dir = tempdir().unwrap();
        let mut manifest = File::create(temp_dir.path().join("package.json")).unwrap();

        let nested = tempdir_in(&temp_dir).unwrap();
        let nested = tempdir_in(&nested).unwrap();
        std::env::set_current_dir(nested.path()).unwrap();

        manifest
            .write_all(file_content.to_string().as_bytes())
            .unwrap();

        let file = get_manifest_file().unwrap();

        assert_eq!(file, file_content);

        std::env::set_current_dir(pwd).unwrap();
    }
}
