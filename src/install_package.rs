use futures::future::join_all;
use std::{
    collections::HashMap,
    error::{self},
};

use crate::{
    dependency_resolver::resolve_deps,
    downloader::download_packages,
    hardlink::hardlink_package,
    npm::VersionRangeSpecifier,
    package_manifest::update_package_manifest,
    symlink::{symlink_dep, symlink_direct},
};

pub async fn install_package(
    dep_name: String,
    dep_version_range: VersionRangeSpecifier,
) -> Result<(), Box<dyn error::Error>> {
    let resolved_deps = resolve_deps(dep_name.clone(), dep_version_range.clone()).await?;

    let top_level = download_packages(&resolved_deps).await?;

    let mut futures = vec![];
    for dep in resolved_deps.iter() {
        futures.push(hardlink_package(&dep.version.name, &dep.version.version))
    }
    let _results = join_all(futures).await;
    println!("hardlink: ");
    println!(
        "{:?}",
        _results
            .iter()
            .filter(|res| res.is_err())
            .collect::<Vec<_>>()
    );

    let mut futures = vec![];
    for package in resolved_deps.iter() {
        for dep in package.dependencies.iter() {
            futures.push(symlink_dep(
                &dep.name,
                &dep.version,
                &package.version.name,
                &package.version.version,
            ));
        }
    }
    let _results = join_all(futures).await;
    println!("symlink deps: ");
    println!(
        "{:?}",
        _results
            .iter()
            .filter(|res| res.is_err())
            .collect::<Vec<_>>()
    );

    if let Some(top_level) = top_level {
        symlink_direct(&top_level.version.name, &top_level.version.version).await?;

        update_package_manifest(HashMap::from([(
            top_level.version.name,
            VersionRangeSpecifier::new(format!("^{}", top_level.version.version)),
        )]))
        .await?;
    }

    Ok(())
}
