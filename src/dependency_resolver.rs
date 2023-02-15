use std::{
    borrow::Borrow,
    cell::RefCell,
    collections::{HashMap, HashSet},
    pin::Pin,
    rc::Rc,
    sync::Arc,
};

use async_recursion::async_recursion;
use futures::{future::join_all, FutureExt};
use tokio::sync::Mutex;

use crate::{
    http::get_npm_package,
    npm::{NpmPackageVersion, ResolvedDependencyTree, VersionRangeSpecifier},
    version_range_resolver::resolve_version_from_version_range,
};

#[async_recursion]
pub async fn resolve_deps(
    deps: HashMap<String, VersionRangeSpecifier>,
    // resolved_tree: Arc<Mutex<HashMap<String, ResolvedDependencyTree>>>,
) {
    if deps.len() == 0 {
        return;
    }

    let mut fetched_descriptor_names = HashSet::new();
    let fetched_ranges: Arc<
        Mutex<HashMap<String, HashMap<VersionRangeSpecifier, &NpmPackageVersion>>>,
    > = Arc::new(Mutex::new(HashMap::new()));

    let mut futures = vec![];

    // Iterating over all of the dependencies, and fetching their metadata
    for (dep_name, range) in deps.iter() {
        // This is not good enough - we need to resolve the dependencies with the `range`, but we can skip the `get_npm_package`
        // if fetched_descriptor_names.contains(dep_name.clone()) {
        // Already fetched this package, no need to do it again
        // continue;
        // }

        let fetched_ranges = Arc::clone(&fetched_ranges);

        let future = get_npm_package(dep_name.to_string().clone()).then(move |resolved| {
            async move {
                // We have the metadata of a dependency
                // Now we need to figure out which version is the relevant one, and fetch its dependencies
                match resolved {
                    Ok(package) => {
                        let (package, version) = resolve_version_from_version_range(package, range);

                        let version = version.expect("failed to find matching version");

                        // We update the fetched ranges so we can skip it in the future
                        if !fetched_ranges.lock().await.contains_key(dep_name) {
                            fetched_ranges
                                .lock()
                                .await
                                .insert(dep_name.to_owned(), HashMap::new());
                        }

                        if let Some(dep_fetched_ranges) =
                            fetched_ranges.lock().await.get_mut(dep_name)
                        {
                            if !dep_fetched_ranges.contains_key(range) {
                                dep_fetched_ranges.insert(range.clone(), &version);
                            }
                        };

                        // We get the list of packages `dep_name` depends on and the ranges, and fetch them recursively
                        // let result = if let Some(package_deps) = &version.dependencies {
                        //     resolve_deps(package_deps.to_owned()).await
                        // } else {
                        //     ()
                        // };

                        // result
                    }
                    Err(e) => panic!("{:?}", e),
                }
            }
        });

        futures.push(future);
        fetched_descriptor_names.insert(dep_name.clone());
    }

    let result = join_all(futures).await;
}

// async fn get_package(
//     dep_name: &String,
// ) -> Option<Pin<Box<HashMap<String, ResolvedDependencyTree>>>> {
//     get_npm_package(dep_name.to_string().clone())
//         .then(move |resolved| {
//             async move {
//                 // We have the metadata of a dependency
//                 // Now we need to figure out which version is the relevant one, and fetch its dependencies
//                 match resolved {
//                     Ok(package) => {
//                         let (package, version) = resolve_version_from_version_range(package, range);

//                         let version = version.expect("failed to find matching version");

//                         // We update the fetched ranges so we can skip it in the future
//                         if !fetched_ranges.lock().await.contains_key(dep_name) {
//                             fetched_ranges
//                                 .lock()
//                                 .await
//                                 .insert(dep_name.to_owned(), HashMap::new());
//                         }

//                         if let Some(dep_fetched_ranges) =
//                             fetched_ranges.lock().await.get_mut(dep_name)
//                         {
//                             if !dep_fetched_ranges.contains_key(range) {
//                                 dep_fetched_ranges.insert(range.clone(), version);
//                             }
//                         }

//                         // We get the list of packages `dep_name` depends on and the ranges, and fetch them recursively
//                         let result = if let Some(package_deps) = &version.dependencies {
//                             Some(resolve_deps(package_deps.to_owned()))
//                         } else {
//                             None
//                         };

//                         result
//                     }
//                     Err(e) => panic!("{:?}", e),
//                 }
//             }
//         })
//         .await
// }
