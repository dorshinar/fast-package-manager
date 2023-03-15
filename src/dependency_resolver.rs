use std::collections::{HashMap, HashSet};

use futures::{stream::FuturesUnordered, StreamExt};

use crate::{
    config::Config,
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
    deps: HashMap<String, VersionRangeSpecifier>,
    client: &Config,
) -> anyhow::Result<Vec<ResolvedDependencies>> {
    // let mut package_to_get_from_npm = HashSet::new();
    let mut futures = FuturesUnordered::new();
    let mut fetched_packages = HashSet::new();

    for (dep_name, dep_version_range) in deps {
        // package_to_get_from_npm.insert((dep_name, dep_version_range, true));
        if fetched_packages.contains(&dep_name) {
            continue;
        }
        let future = get_npm_package_version(dep_name.clone(), dep_version_range, true, client);
        fetched_packages.insert(dep_name);
        futures.push(future);
    }

    let mut resolved_versions: HashMap<
        String,
        HashMap<VersionRangeSpecifier, (NpmPackageVersion, bool)>,
    > = HashMap::new();

    // while !package_to_get_from_npm.is_empty() {
    // let mut futures = FuturesUnordered::new();
    // for package in package_to_get_from_npm.iter() {
    //     if fetched_packages.contains(&package.0) {
    //         continue;
    //     }
    //     let future =
    //         get_npm_package_version(package.0.clone(), package.1.clone(), package.2, client);
    //     fetched_packages.insert(package.0.clone());
    //     futures.push(future);
    // }

    // let mut versions: Vec<(
    //     VersionRangeSpecifier,
    //     Result<NpmPackageVersion, anyhow::Error>,
    //     bool,
    // )> = vec![];

    // for fut in futures
    //     .take(100)
    //     .collect::<Vec<(
    //         VersionRangeSpecifier,
    //         Result<NpmPackageVersion, anyhow::Error>,
    //         bool,
    //     )>>()
    //     .await
    // {
    //     versions.push(fut);
    // }
    // versions.push(join_all(futures).await.iter().tak)
    //  = join_all(futures).await;

    // package_to_get_from_npm.clear();

    loop {
        match futures.next().await {
            Some(Ok((range, version, is_root))) => {
                let version_clone = version.clone();
                for (dep_name, dep_version_range) in version.dependencies {
                    match resolved_versions.get(&dep_name) {
                        Some(ranges) if !ranges.contains_key(&dep_version_range) => {
                            let future =
                                get_npm_package_version(dep_name, dep_version_range, false, client);
                            // .then(|version| {
                            // return (
                            //     String::from(""),
                            //     VersionRangeSpecifier::new(String::from("")),
                            //     false,
                            // );
                            // });
                            futures.push(future);
                            // package_to_get_from_npm.insert((
                            //     dep.0.to_owned(),
                            //     dep.1.to_owned(),
                            //     false,
                            // ));
                        }
                        Some(_) => {}
                        None => {
                            // package_to_get_from_npm.insert((
                            //     dep.0.to_owned(),
                            //     dep.1.to_owned(),
                            //     false,
                            // ));
                        }
                    }
                }

                match resolved_versions.get_mut(&version.name) {
                    Some(range_to_versions) => {
                        range_to_versions.insert(range.clone(), (version_clone, is_root.clone()));
                    }
                    None => {
                        let version_name = version.name.clone();

                        let mut range_to_version = HashMap::new();
                        range_to_version.insert(range.clone(), (version_clone, is_root.clone()));
                        resolved_versions.insert(version_name, range_to_version);
                    }
                }
            }
            Some(Err(_)) => {}
            None => {
                break;
            }
        }
    }
    // while let Some(Ok((range, version, is_root))) = &futures.next().await {
    //     // match version {
    //     //     Ok(version) => {
    //     for (dep_name, dep_version_range) in &version.dependencies {
    //         match resolved_versions.get(dep_name) {
    //             Some(ranges) if !ranges.contains_key(dep_version_range) => {
    //                 let name = String::from("");
    //                 let future =
    //                     get_npm_package_version(dep_name, dep_version_range, false, client);
    //                 // .then(|version| {
    //                 // return (
    //                 //     String::from(""),
    //                 //     VersionRangeSpecifier::new(String::from("")),
    //                 //     false,
    //                 // );
    //                 // });
    //                 futures.push(future);
    //                 // package_to_get_from_npm.insert((
    //                 //     dep.0.to_owned(),
    //                 //     dep.1.to_owned(),
    //                 //     false,
    //                 // ));
    //             }
    //             Some(_) => {}
    //             None => {
    //                 // package_to_get_from_npm.insert((
    //                 //     dep.0.to_owned(),
    //                 //     dep.1.to_owned(),
    //                 //     false,
    //                 // ));
    //             }
    //         }
    //     }

    //     match resolved_versions.get_mut(&version.name) {
    //         Some(range_to_versions) => {
    //             range_to_versions.insert(range.clone(), (version.clone(), is_root.clone()));
    //         }
    //         None => {
    //             let version_name = version.name.clone();

    //             let mut range_to_version = HashMap::new();
    //             range_to_version.insert(range.clone(), (version.clone(), is_root.clone()));
    //             resolved_versions.insert(version_name, range_to_version);
    //         }
    //     }
    //     //     }
    //     //     Err(error) => {
    //     //         println!("{:?}", error);
    //     //         return Err(Error::DependencyResolveError.into());
    //     //     }
    //     // }
    // }
    // }

    construct_dependency_vec(resolved_versions)
}

pub fn construct_dependency_vec(
    resolved: HashMap<String, HashMap<VersionRangeSpecifier, (NpmPackageVersion, bool)>>,
) -> anyhow::Result<Vec<ResolvedDependencies>> {
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
) -> anyhow::Result<ResolvedDependencyTree> {
    let root_resolved_version = match resolved_versions.get(root_name) {
        Some(versions) => versions.get(&root_range),
        None => None,
    };

    let root_resolved_version = match root_resolved_version {
        Some(version) => version.to_owned(),
        None => {
            return Err(Error::VersionDoesNotExist.into());
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
    package_name: String,
    version_range: VersionRangeSpecifier,
    is_root: bool,
    client: &Config,
) -> anyhow::Result<(VersionRangeSpecifier, NpmPackageVersion, bool)> {
    let package = get_npm_package(&package_name, client).await?;

    let version =
        resolve_version_from_version_range(&package, &version_range).map_err(|error| error)?;

    Ok((version_range.to_owned(), version, is_root))
}
