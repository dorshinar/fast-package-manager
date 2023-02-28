use std::{
    collections::HashMap,
    fs::{self, File},
    io::{BufReader, Result},
};

use serde_json::Value;
use tokio::task;

use crate::npm::VersionRangeSpecifier;

pub async fn update_package_manifest(
    packages_to_add: HashMap<String, VersionRangeSpecifier>,
) -> Result<()> {
    task::spawn_blocking(|| update_manifest(packages_to_add)).await??;

    Ok(())
}

fn update_manifest(packages_to_add: HashMap<String, VersionRangeSpecifier>) -> Result<()> {
    let file = File::open(String::from("./package.json"))?;
    let reader = BufReader::new(file);

    // Read the JSON contents of the file as an instance of `User`.
    let mut package_json: Value = serde_json::from_reader(reader)?;

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
