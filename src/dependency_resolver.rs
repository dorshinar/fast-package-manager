use std::{
    collections::{HashMap, HashSet},
    error,
};

use futures::{future::join_all, FutureExt};

use crate::{
    http::get_npm_package,
    npm::{NpmPackageVersion, ResolvedDependencyTree, UrlString, VersionRangeSpecifier},
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
) -> Result<(ResolvedDependencyTree, HashMap<String, UrlString>), Box<dyn error::Error>> {
    let dep_name_copy = dep_name.to_owned();
    let dep_version_range_copy = dep_version_range.to_owned();

    let mut package_to_get_from_npm = HashSet::new();
    package_to_get_from_npm.insert((dep_name, dep_version_range));

    let mut resolved_versions: HashMap<String, HashMap<VersionRangeSpecifier, NpmPackageVersion>> =
        HashMap::new();

    let mut tarballs = HashMap::new();

    while !package_to_get_from_npm.is_empty() {
        let mut futures = Vec::new();
        for package in package_to_get_from_npm.iter() {
            let future = get_npm_package_version(&package.0, &package.1)
                .then(|version| async { (package.1.to_owned(), version) });
            futures.push(future);
        }

        let versions = join_all(futures).await;

        package_to_get_from_npm.clear();

        for (range, version) in versions {
            match version {
                Ok(version) => {
                    if let Some(deps) = &version.dependencies {
                        for dep in deps {
                            package_to_get_from_npm.insert((dep.0.to_owned(), dep.1.to_owned()));
                        }
                    }

                    tarballs.insert(version.name.clone(), version.dist.tarball.clone());

                    match resolved_versions.get_mut(&version.name) {
                        Some(range_to_versions) => {
                            range_to_versions.insert(range, version);
                        }
                        None => {
                            let version_name = version.name.clone();

                            let mut range_to_version = HashMap::new();
                            range_to_version.insert(range, version);
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

    construct_dependency_tree(&dep_name_copy, &dep_version_range_copy, &resolved_versions)
        .map(|tree| (tree, tarballs))
}

fn construct_dependency_tree(
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

    let deps = match &root_resolved_version.dependencies {
        Some(deps) => {
            let mut trees = Vec::new();

            for (dep_name, dep_range) in deps {
                match construct_dependency_tree(dep_name, dep_range, resolved_versions) {
                    Ok(tree) => {
                        trees.push(tree);
                    }
                    Err(error) => return Err(error),
                }
            }

            trees
        }
        None => Vec::new(),
    };

    let dep_tree = ResolvedDependencyTree::new(root_name.to_owned(), root_resolved_version, deps);
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

        println!("{:#?}", serde_json::to_string(&resolved.0).unwrap())
    }
}
