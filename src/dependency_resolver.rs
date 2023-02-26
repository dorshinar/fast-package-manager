use std::{
    collections::{HashMap, HashSet},
    error,
};

use futures::{future::join_all, FutureExt};

use crate::{
    http::get_npm_package,
    npm::{NpmPackageVersion, ResolvedDependencies, ResolvedDependencyTree, VersionRangeSpecifier},
    resolve_version_range::resolve_version_from_version_range,
};

#[derive(Debug, derive_more::Display, derive_more::Error)]
pub enum Error {
    DependencyResolveError,
    VersionDoesNotExist,
}

pub async fn resolve_deps(
    dep_name: String,
    dep_version_range: VersionRangeSpecifier,
) -> Result<Vec<ResolvedDependencies>, Box<dyn error::Error>> {
    let mut package_to_get_from_npm = HashSet::new();
    package_to_get_from_npm.insert((dep_name, dep_version_range, true));

    let mut resolved_versions: HashMap<
        String,
        HashMap<VersionRangeSpecifier, (NpmPackageVersion, bool)>,
    > = HashMap::new();

    let mut fetched_packages = HashSet::new();

    while !package_to_get_from_npm.is_empty() {
        let mut futures = Vec::new();
        for package in package_to_get_from_npm.iter() {
            if fetched_packages.contains(&package.0) {
                continue;
            }
            let future = get_npm_package_version(&package.0, &package.1)
                .then(|version| async { (package.1.to_owned(), version, package.2) });
            fetched_packages.insert(package.0.clone());
            futures.push(future);
        }

        let versions = join_all(futures).await;

        package_to_get_from_npm.clear();

        for (range, version, is_root) in versions {
            match version {
                Ok(version) => {
                    for dep in &version.dependencies {
                        match resolved_versions.get(dep.0) {
                            Some(ranges) if !ranges.contains_key(dep.1) => {
                                package_to_get_from_npm.insert((
                                    dep.0.to_owned(),
                                    dep.1.to_owned(),
                                    false,
                                ));
                            }
                            Some(_) => {}
                            None => {
                                package_to_get_from_npm.insert((
                                    dep.0.to_owned(),
                                    dep.1.to_owned(),
                                    false,
                                ));
                            }
                        }
                    }

                    match resolved_versions.get_mut(&version.name) {
                        Some(range_to_versions) => {
                            range_to_versions.insert(range, (version, is_root));
                        }
                        None => {
                            let version_name = version.name.clone();

                            let mut range_to_version = HashMap::new();
                            range_to_version.insert(range, (version, is_root));
                            resolved_versions.insert(version_name, range_to_version);
                        }
                    }
                }
                Err(error) => {
                    return Err(error);
                }
            }
        }
    }

    construct_dependency_vec(resolved_versions)
}

pub fn construct_dependency_vec(
    resolved: HashMap<String, HashMap<VersionRangeSpecifier, (NpmPackageVersion, bool)>>,
) -> Result<Vec<ResolvedDependencies>, Box<dyn error::Error>> {
    let mut resolved_deps = vec![];

    for (_package, ranges) in resolved.iter() {
        for (_range, (version, is_root)) in ranges {
            let mut dependencies = vec![];

            for dep in &version.dependencies {
                if let Some(ranges) = resolved.get(dep.0) {
                    if let Some((version, _)) = ranges.get(dep.1) {
                        dependencies.push(version.to_owned())
                    }
                }
            }

            resolved_deps.push(ResolvedDependencies::new(
                version.to_owned(),
                dependencies,
                is_root.to_owned(),
            ));
        }
    }

    Ok(resolved_deps)
}

pub fn construct_dependency_tree(
    root_name: &String,
    root_range: &VersionRangeSpecifier,
    resolved_versions: &HashMap<String, HashMap<VersionRangeSpecifier, NpmPackageVersion>>,
) -> Result<ResolvedDependencyTree, Box<dyn error::Error>> {
    let root_resolved_version = match resolved_versions.get(root_name) {
        Some(versions) => versions.get(&root_range),
        None => None,
    };

    let root_resolved_version = match root_resolved_version {
        Some(version) => version.to_owned(),
        None => {
            return Err(Box::new(Error::VersionDoesNotExist));
        }
    };

    let mut trees = Vec::new();

    for (dep_name, dep_range) in &root_resolved_version.dependencies {
        match construct_dependency_tree(dep_name, dep_range, resolved_versions) {
            Ok(tree) => {
                trees.push(tree);
            }
            Err(error) => return Err(error),
        }
    }

    let dep_tree = ResolvedDependencyTree::new(root_name.to_owned(), root_resolved_version, trees);
    Ok(dep_tree)
}

async fn get_npm_package_version(
    package_name: &String,
    version: &VersionRangeSpecifier,
) -> Result<NpmPackageVersion, Box<dyn error::Error>> {
    let package = get_npm_package(package_name).await?;

    resolve_version_from_version_range(&package, version).map_err(|error| error.into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn resolves_deps() {
        let resolved = resolve_deps(
            String::from("create-react-app"),
            VersionRangeSpecifier::new(String::from("latest")),
        )
        .await
        .expect("failed to get deps");

        println!("{:#?}", serde_json::to_string(&resolved).unwrap())
    }
}
