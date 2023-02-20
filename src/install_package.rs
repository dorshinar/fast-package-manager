use std::error;

use futures::future::join_all;

use crate::{
    dependency_resolver::resolve_deps, npm::VersionRangeSpecifier,
    npm_fs::download_package_to_store,
};

pub async fn install_package(
    dep_name: String,
    dep_version_range: VersionRangeSpecifier,
) -> Result<(), Box<dyn error::Error>> {
    let (deps, tarballs) = resolve_deps(dep_name, dep_version_range).await?;

    let mut futures = Vec::new();
    for (package, tar) in tarballs {
        futures.push(download_package_to_store(package, tar));
    }

    join_all(futures).await;

    Ok(())
}
