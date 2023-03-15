use std::collections::HashMap;

use tokio::task;

use crate::{
    config::Config, install_package::install_package, npm::VersionRangeSpecifier,
    package_manifest::get_manifest_file,
};

pub async fn install_manifest(config: &Config) -> anyhow::Result<()> {
    let manifest_file = task::spawn_blocking(|| get_manifest_file()).await??;

    let deps: Option<HashMap<String, VersionRangeSpecifier>> =
        match manifest_file.get("dependencies") {
            Some(deps) => serde_json::from_value(deps.to_owned())
                .map(Some)
                .unwrap_or(None),
            None => None,
        };

    if let Some(deps) = deps {
        install_package(deps, config).await?;
    }

    Ok(())
}
