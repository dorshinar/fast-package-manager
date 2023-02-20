use crate::npm::{NpmPackageVersion, UrlString, Version, VersionRangeSpecifier};
use async_recursion::async_recursion;
use derive_more::Display;
use futures::future::join_all;
use reqwest::Url;

use std::collections::HashMap;
use std::error::{self, Error};
use std::path::Path;

// pub struct PackageFetcher<'a, Network: NetworkAdapter, Writer: WriteResponse, Extractor: Extract> {
//     network_adapter: &'a Network,
//     response_writer: &'a Writer,
//     tar_extractor: &'a Extractor,
// }

// impl<'a, Network: NetworkAdapter, Writer: WriteResponse, Extractor: Extract>
//     PackageFetcher<'a, Network, Writer, Extractor>
// {
//     pub fn new(
//         network_adapter: &'a Network,
//         response_writer: &'a Writer,
//         tar_extractor: &'a Extractor,
//     ) -> Self {
//         PackageFetcher {
//             network_adapter,
//             response_writer,
//             tar_extractor,
//         }
//     }

//     pub async fn install_package(&self, package_name: &String) -> Result<Version, Box<dyn Error>> {
//         let mut dependencies = HashMap::new();
//         dependencies.insert(
//             package_name.to_owned(),
//             VersionRangeSpecifier::new(String::from("latest")),
//         );

//         let mut package_map = HashMap::new();
//         let deps = self
//             .get_deps_with_versions(&Some(dependencies), &mut package_map)
//             .await;

//         let tar_futures = deps
//             .iter()
//             .map(|dep| self.get_tar(&dep.0, &dep.1.dist.tarball))
//             .collect::<Vec<_>>();

//         join_all(tar_futures).await;

//         match deps.get(package_name) {
//             Some(version) => Ok(version.version.clone()),
//             None => Err(Box::new(FetchError::new())),
//         }
//     }

//     async fn get_tar(&self, package_name: &str, tarball: &UrlString) -> Result<(), Box<dyn Error>> {
//         let file_name = Url::parse(&tarball)?
//             .path_segments()
//             .and_then(|segments| segments.last())
//             .expect("failed to parse filename from tarball Url")
//             .to_string();

//         let tar_content = self.network_adapter.get_package_tar(tarball).await?;
//         let tarball_path = Path::new(&TEMP_FOLDER).join(&file_name);

//         self.response_writer
//             .write_response(tar_content, &tarball_path)
//             .await?;

//         let deps_dest = Path::new(DEPS_FOLDER).join(&package_name);

//         self.tar_extractor.extract_tar_gz(&tarball_path, &deps_dest)
//     }

//     /// Fetch the packages from NPM and return a map of the version
//     /// to download for each dependency, recursively.
//     #[async_recursion(?Send)]
//     async fn get_deps_with_versions<'b>(
//         &self,
//         deps: &Option<HashMap<String, VersionRangeSpecifier>>,
//         packages_to_download: &'b mut HashMap<String, NpmPackageVersion>,
//     ) -> &'b mut Vec<RangedDependencyTree> {
//         println!("getting deps: {:?}", deps);

//         if deps.is_none() {
//             return packages_to_download;
//         }

//         let owned_deps = deps.as_ref().unwrap();

//         let mut futures = vec![];
//         let mut downloaded_new_packages = false;

//         for (dep_name, _) in owned_deps.iter() {
//             if packages_to_download.contains_key(dep_name) {
//                 continue;
//             }

//             futures.push(self.get_package(dep_name));
//             downloaded_new_packages = true;
//         }

//         let awaited_packages = join_all(futures).await;

//         let mut sub_deps: HashMap<String, VersionRangeSpecifier> = HashMap::new();
//         for package_result in awaited_packages {
//             match package_result {
//                 Ok(package) => {
//                     let resolved = self.resolve_version_from_version_identifier(
//                         &package,
//                         owned_deps.get(&package.parsed.name).unwrap(),
//                     );

//                     if let Some(resolved) = resolved {
//                         packages_to_download.insert(package.parsed.name.clone(), resolved.clone());
//                         if let Some(package_deps) = &resolved.dependencies {
//                             for (dependency_name, version_range) in package_deps.iter() {
//                                 sub_deps
//                                     .insert(dependency_name.to_owned(), version_range.to_owned());
//                             }
//                         }
//                     }
//                 }
//                 Err(e) => panic!("{:?}", e),
//             }
//         }

//         if downloaded_new_packages {
//             self.get_deps_with_versions(&Some(sub_deps), packages_to_download)
//                 .await
//         } else {
//             packages_to_download
//         }
//     }

//     async fn get_package(&self, name: &String) -> Result<NpmPackage, Box<dyn Error>> {
//         let resolved = self.network_adapter.get_package(name).await?;

//         Ok(NpmPackage {
//             json: serde_json::from_str(&resolved)?,
//             parsed: serde_json::from_str(&resolved)?,
//         })
//     }
// }

// #[derive(Debug, Display)]
// pub struct FetchError;

// impl FetchError {
//     pub fn new() -> Self {
//         Self
//     }
// }

// impl error::Error for FetchError {}

// #[cfg(test)]
// mod tests {
//     use mockall::predicate::eq;

//     use crate::{
//         http::NpmNetworkAdapter,
//         network_adapter::MockNetworkAdapter,
//         npm::{NpmResolvedPackage, RangedDependencyTree},
//         npm_tar_extractor::NpmTarExtractor,
//         response_writer::ResponseWriter,
//     };

//     use super::*;

//     #[test]
//     fn resolves_semver_returns_last_matching() {
//         let network_adapter = NpmNetworkAdapter::new();
//         let resp_writer = ResponseWriter::new();
//         let tar_ext = NpmTarExtractor::new();
//         let fetcher = PackageFetcher::new(&network_adapter, &resp_writer, &tar_ext);

//         let package_json = r#"{
//             "name": "is-even",
//             "dist-tags": {
//               "latest": "1.0.0"
//             },
//             "versions": {
//               "0.1.2": {
//                 "name": "is-even",
//                 "version": "0.1.2",
//                 "dependencies": {
//                   "is-odd": "^0.1.2"
//                 },
//                 "devDependencies": {
//                   "mocha": "^3.4.2"
//                 },
//                 "dist": {
//                   "shasum": "e0432a7379f2d20b6ebbc2cb11e69beaaf31cd63",
//                   "tarball": "https://registry.npmjs.org/is-even/-/is-even-0.1.2.tgz",
//                   "integrity": "sha512-Ft/TASRTCMIR20eeGtXIx7W+TfAbw/LkG7D3Pw5rncxaF1LCei/WVgO2qxsJiOROZb7JABl6Z8N2pEHjNONt9A==",
//                   "signatures": [
//                     {
//                       "keyid": "SHA256:jl3bwswu80PjjokCgh0o2w5c2U4LhQAE57gj9cz1kzA",
//                       "sig": "MEUCIGBqTtBRc6/6dqmI2lc+kmJRw4bB5kGHp5dM0fQH3V5pAiEA18DczU8X1bgDIkckzUOYpzWgZZJeQnpbgdPp9WtLnwY="
//                     }
//                   ]
//                 },
//                 "engines": {
//                   "node": ">=0.10.0"
//                 }
//               },
//               "1.0.0": {
//                 "name": "is-even",
//                 "version": "1.0.0",
//                 "dependencies": {
//                   "is-odd": "^0.1.2"
//                 },
//                 "devDependencies": {
//                   "gulp-format-md": "^0.1.12",
//                   "mocha": "^3.4.2"
//                 },
//                 "dist": {
//                   "shasum": "76b5055fbad8d294a86b6a949015e1c97b717c06",
//                   "tarball": "https://registry.npmjs.org/is-even/-/is-even-1.0.0.tgz",
//                   "integrity": "sha512-LEhnkAdJqic4Dbqn58A0y52IXoHWlsueqQkKfMfdEnIYG8A1sm/GHidKkS6yvXlMoRrkM34csHnXQtOqcb+Jzg==",
//                   "signatures": [
//                     {
//                       "keyid": "SHA256:jl3bwswu80PjjokCgh0o2w5c2U4LhQAE57gj9cz1kzA",
//                       "sig": "MEQCIGdFCa72n+vIbeujikn3ExFVcAX2rnuKWBBFWQlQIH1gAiBn5HXb7zzOTEFwAnHX8zrI8+2gPyDaxgy5gAMFq7fzhA=="
//                     }
//                   ]
//                 },
//                 "engines": {
//                   "node": ">=0.10.0"
//                 }
//               },
//               "1.0.1": {
//                 "name": "is-even",
//                 "version": "1.0.1",
//                 "dependencies": {
//                   "is-odd": "^0.1.2"
//                 },
//                 "devDependencies": {
//                   "gulp-format-md": "^0.1.12",
//                   "mocha": "^3.4.2"
//                 },
//                 "dist": {
//                   "shasum": "76b5055fbad8d294a86b6a949015e1c97b717c06",
//                   "tarball": "https://registry.npmjs.org/is-even/-/is-even-1.0.1.tgz",
//                   "integrity": "sha512-LEhnkAdJqic4Dbqn58A0y52IXoHWlsueqQkKfMfdEnIYG8A1sm/GHidKkS6yvXlMoRrkM34csHnXQtOqcb+Jzg==",
//                   "signatures": [
//                     {
//                       "keyid": "SHA256:jl3bwswu80PjjokCgh0o2w5c2U4LhQAE57gj9cz1kzA",
//                       "sig": "MEQCIGdFCa72n+vIbeujikn3ExFVcAX2rnuKWBBFWQlQIH1gAiBn5HXb7zzOTEFwAnHX8zrI8+2gPyDaxgy5gAMFq7fzhA=="
//                     }
//                   ]
//                 },
//                 "engines": {
//                   "node": ">=0.10.0"
//                 }
//               }
//             },
//             "modified": "2022-06-19T02:40:54.045Z"
//           }"#;

//         let package = &NpmPackage {
//             json: serde_json::from_str(&package_json).unwrap(),
//             parsed: serde_json::from_str(&package_json).unwrap(),
//         };

//         let resolved = fetcher.resolve_version_from_version_identifier(
//             package,
//             &VersionRangeSpecifier::new(String::from("^1.0.0")),
//         );

//         assert_eq!(
//             resolved,
//             package
//                 .parsed
//                 .versions
//                 .get(&Version::new("1.0.1".to_string()))
//         );
//     }

//     #[test]
//     fn resolve_version_returns_none_with_semver() {
//         let network_adapter = NpmNetworkAdapter::new();
//         let resp_writer = ResponseWriter::new();
//         let tar_ext = NpmTarExtractor::new();
//         let fetcher = PackageFetcher::new(&network_adapter, &resp_writer, &tar_ext);

//         let package_json = r#"{
//             "name": "is-even",
//             "dist-tags": {
//               "latest": "1.0.0"
//             },
//             "versions": {
//               "0.1.2": {
//                 "name": "is-even",
//                 "version": "0.1.2",
//                 "dependencies": {
//                   "is-odd": "^0.1.2"
//                 },
//                 "devDependencies": {
//                   "mocha": "^3.4.2"
//                 },
//                 "dist": {
//                   "shasum": "e0432a7379f2d20b6ebbc2cb11e69beaaf31cd63",
//                   "tarball": "https://registry.npmjs.org/is-even/-/is-even-0.1.2.tgz",
//                   "integrity": "sha512-Ft/TASRTCMIR20eeGtXIx7W+TfAbw/LkG7D3Pw5rncxaF1LCei/WVgO2qxsJiOROZb7JABl6Z8N2pEHjNONt9A==",
//                   "signatures": [
//                     {
//                       "keyid": "SHA256:jl3bwswu80PjjokCgh0o2w5c2U4LhQAE57gj9cz1kzA",
//                       "sig": "MEUCIGBqTtBRc6/6dqmI2lc+kmJRw4bB5kGHp5dM0fQH3V5pAiEA18DczU8X1bgDIkckzUOYpzWgZZJeQnpbgdPp9WtLnwY="
//                     }
//                   ]
//                 },
//                 "engines": {
//                   "node": ">=0.10.0"
//                 }
//               },
//               "1.0.0": {
//                 "name": "is-even",
//                 "version": "1.0.0",
//                 "dependencies": {
//                   "is-odd": "^0.1.2"
//                 },
//                 "devDependencies": {
//                   "gulp-format-md": "^0.1.12",
//                   "mocha": "^3.4.2"
//                 },
//                 "dist": {
//                   "shasum": "76b5055fbad8d294a86b6a949015e1c97b717c06",
//                   "tarball": "https://registry.npmjs.org/is-even/-/is-even-1.0.0.tgz",
//                   "integrity": "sha512-LEhnkAdJqic4Dbqn58A0y52IXoHWlsueqQkKfMfdEnIYG8A1sm/GHidKkS6yvXlMoRrkM34csHnXQtOqcb+Jzg==",
//                   "signatures": [
//                     {
//                       "keyid": "SHA256:jl3bwswu80PjjokCgh0o2w5c2U4LhQAE57gj9cz1kzA",
//                       "sig": "MEQCIGdFCa72n+vIbeujikn3ExFVcAX2rnuKWBBFWQlQIH1gAiBn5HXb7zzOTEFwAnHX8zrI8+2gPyDaxgy5gAMFq7fzhA=="
//                     }
//                   ]
//                 },
//                 "engines": {
//                   "node": ">=0.10.0"
//                 }
//               }
//             },
//             "modified": "2022-06-19T02:40:54.045Z"
//           }"#;

//         let package = &NpmPackage {
//             json: serde_json::from_str(&package_json).unwrap(),
//             parsed: serde_json::from_str(&package_json).unwrap(),
//         };

//         let resolved = fetcher.resolve_version_from_version_identifier(
//             package,
//             &VersionRangeSpecifier::new(String::from("^2.0.0")),
//         );

//         assert_eq!(resolved, None);
//     }

//     #[test]
//     fn resolve_version_returns_none_with_latest() {
//         let network_adapter = NpmNetworkAdapter::new();
//         let resp_writer = ResponseWriter::new();
//         let tar_ext = NpmTarExtractor::new();
//         let fetcher = PackageFetcher::new(&network_adapter, &resp_writer, &tar_ext);

//         let package_json = r#"{
//             "name": "is-even",
//             "dist-tags": {
//               "newest": "1.0.0"
//             },
//             "versions": {
//               "0.1.2": {
//                 "name": "is-even",
//                 "version": "0.1.2",
//                 "dependencies": {
//                   "is-odd": "^0.1.2"
//                 },
//                 "devDependencies": {
//                   "mocha": "^3.4.2"
//                 },
//                 "dist": {
//                   "shasum": "e0432a7379f2d20b6ebbc2cb11e69beaaf31cd63",
//                   "tarball": "https://registry.npmjs.org/is-even/-/is-even-0.1.2.tgz",
//                   "integrity": "sha512-Ft/TASRTCMIR20eeGtXIx7W+TfAbw/LkG7D3Pw5rncxaF1LCei/WVgO2qxsJiOROZb7JABl6Z8N2pEHjNONt9A==",
//                   "signatures": [
//                     {
//                       "keyid": "SHA256:jl3bwswu80PjjokCgh0o2w5c2U4LhQAE57gj9cz1kzA",
//                       "sig": "MEUCIGBqTtBRc6/6dqmI2lc+kmJRw4bB5kGHp5dM0fQH3V5pAiEA18DczU8X1bgDIkckzUOYpzWgZZJeQnpbgdPp9WtLnwY="
//                     }
//                   ]
//                 },
//                 "engines": {
//                   "node": ">=0.10.0"
//                 }
//               },
//               "1.0.0": {
//                 "name": "is-even",
//                 "version": "1.0.0",
//                 "dependencies": {
//                   "is-odd": "^0.1.2"
//                 },
//                 "devDependencies": {
//                   "gulp-format-md": "^0.1.12",
//                   "mocha": "^3.4.2"
//                 },
//                 "dist": {
//                   "shasum": "76b5055fbad8d294a86b6a949015e1c97b717c06",
//                   "tarball": "https://registry.npmjs.org/is-even/-/is-even-1.0.0.tgz",
//                   "integrity": "sha512-LEhnkAdJqic4Dbqn58A0y52IXoHWlsueqQkKfMfdEnIYG8A1sm/GHidKkS6yvXlMoRrkM34csHnXQtOqcb+Jzg==",
//                   "signatures": [
//                     {
//                       "keyid": "SHA256:jl3bwswu80PjjokCgh0o2w5c2U4LhQAE57gj9cz1kzA",
//                       "sig": "MEQCIGdFCa72n+vIbeujikn3ExFVcAX2rnuKWBBFWQlQIH1gAiBn5HXb7zzOTEFwAnHX8zrI8+2gPyDaxgy5gAMFq7fzhA=="
//                     }
//                   ]
//                 },
//                 "engines": {
//                   "node": ">=0.10.0"
//                 }
//               }
//             },
//             "modified": "2022-06-19T02:40:54.045Z"
//           }"#;

//         let package = &NpmPackage {
//             json: serde_json::from_str(&package_json).unwrap(),
//             parsed: serde_json::from_str(&package_json).unwrap(),
//         };

//         let resolved = fetcher.resolve_version_from_version_identifier(
//             package,
//             &VersionRangeSpecifier::new(String::from("latest")),
//         );

//         assert_eq!(resolved, None);
//     }

//     #[tokio::test]
//     async fn get_deps_with_versions_return_packages_with_none_deps() {
//         let mut mock_network_adapter = MockNetworkAdapter::new();
//         mock_network_adapter
//             .expect_get_package()
//             .returning(|_| Ok(String::from("")));
//         mock_network_adapter.expect_get_package().times(0);

//         let resp_writer = ResponseWriter::new();
//         let tar_ext = NpmTarExtractor::new();
//         let fetcher = PackageFetcher::new(&mock_network_adapter, &resp_writer, &tar_ext);

//         let mut packages_to_download: Vec<RangedDependencyTree> = Vec::new();
//         let mut packages_to_download_clone = packages_to_download.clone();

//         let res: Vec<RangedDependencyTree> = fetcher
//             .get_deps_with_versions(&None, &mut packages_to_download_clone)
//             .await;

//         assert_eq!(res, packages_to_download);
//     }

//     #[tokio::test]
//     async fn get_deps_with_versions_return_packages_with_deps() {
//         let raw_dep = r#"{
//           "name": "dep-1",
//           "dist-tags": {
//               "latest": "1.0.0"
//           },
//           "versions": {
//               "1.0.0": {
//                   "name": "dep-1",
//                   "version": "1.0.0",
//                   "dependencies": {
//                   },
//                   "devDependencies": {
//                   },
//                   "dist": {
//                       "shasum": "76b5055fbad8d294a86b6a949015e1c97b717c06",
//                       "tarball": "https://registry.npmjs.org/dep-1/-/dep-1-1.0.0.tgz",
//                       "integrity": "sha512-LEhnkAdJqic4Dbqn58A0y52IXoHWlsueqQkKfMfdEnIYG8A1sm/GHidKkS6yvXlMoRrkM34csHnXQtOqcb+Jzg==",
//                       "signatures": [
//                           {
//                               "keyid": "SHA256:jl3bwswu80PjjokCgh0o2w5c2U4LhQAE57gj9cz1kzA",
//                               "sig": "MEQCIGdFCa72n+vIbeujikn3ExFVcAX2rnuKWBBFWQlQIH1gAiBn5HXb7zzOTEFwAnHX8zrI8+2gPyDaxgy5gAMFq7fzhA=="
//                           }
//                       ]
//                   },
//                   "engines": {
//                   }
//               }
//           },
//           "modified": "2022-06-19T02:40:54.045Z"
//         }"#;

//         let dep: NpmResolvedPackage = serde_json::from_str(&raw_dep).unwrap();

//         let mut mock_network_adapter = MockNetworkAdapter::new();
//         mock_network_adapter
//             .expect_get_package()
//             .returning(|_| Ok(raw_dep.to_string()));

//         let resp_writer = ResponseWriter::new();
//         let tar_ext = NpmTarExtractor::new();
//         let fetcher = PackageFetcher::new(&mock_network_adapter, &resp_writer, &tar_ext);

//         let mut packages_to_download: Vec<RangedDependencyTree> = Vec::new();

//         let mut deps = HashMap::new();
//         deps.insert(
//             dep.name.clone(),
//             VersionRangeSpecifier::new(String::from("^1.0.0")),
//         );

//         let res: Vec<RangedDependencyTree> = fetcher
//             .get_deps_with_versions(&Some(deps), &mut packages_to_download)
//             .await;

//         let mut expected_packages: Vec<RangedDependencyTree> = Vec::new();
//         expected_packages.push(RangedDependencyTree::new(
//             dep.name.as_str(),
//             dep.versions
//                 .get(&Version::new(String::from("1.0.0")))
//                 .unwrap(),
//             Box::new(Vec::new()),
//         ));

//         assert_eq!(res, expected_packages);
//     }

//     #[tokio::test]
//     async fn get_deps_with_versions_return_packages_with_recursive_deps() {
//         let raw_dep_one = r#"{
//       "name": "dep-1",
//       "dist-tags": {
//           "latest": "1.0.0"
//       },
//       "versions": {
//           "1.0.0": {
//               "name": "dep-1",
//               "version": "1.0.0",
//               "dependencies": {
//                 "dep-2": "^2.0.0"
//               },
//               "devDependencies": {
//               },
//               "dist": {
//                   "shasum": "76b5055fbad8d294a86b6a949015e1c97b717c06",
//                   "tarball": "https://registry.npmjs.org/dep-1/-/dep-1-1.0.0.tgz",
//                   "integrity": "sha512-LEhnkAdJqic4Dbqn58A0y52IXoHWlsueqQkKfMfdEnIYG8A1sm/GHidKkS6yvXlMoRrkM34csHnXQtOqcb+Jzg==",
//                   "signatures": [
//                       {
//                           "keyid": "SHA256:jl3bwswu80PjjokCgh0o2w5c2U4LhQAE57gj9cz1kzA",
//                           "sig": "MEQCIGdFCa72n+vIbeujikn3ExFVcAX2rnuKWBBFWQlQIH1gAiBn5HXb7zzOTEFwAnHX8zrI8+2gPyDaxgy5gAMFq7fzhA=="
//                       }
//                   ]
//               },
//               "engines": {
//               }
//           }
//       },
//       "modified": "2022-06-19T02:40:54.045Z"
//     }"#;
//         let raw_dep_two = r#"{
//       "name": "dep-2",
//       "dist-tags": {
//           "latest": "2.0.0"
//       },
//       "versions": {
//           "2.0.0": {
//               "name": "dep-2",
//               "version": "2.0.0",
//               "dependencies": {
//               },
//               "devDependencies": {
//               },
//               "dist": {
//                   "shasum": "76b5055fbad8d294a86b6a949015e1c97b717c06",
//                   "tarball": "https://registry.npmjs.org/dep-2/-/dep-2-1.0.0.tgz",
//                   "integrity": "sha512-LEhnkAdJqic4Dbqn58A0y52IXoHWlsueqQkKfMfdEnIYG8A1sm/GHidKkS6yvXlMoRrkM34csHnXQtOqcb+Jzg==",
//                   "signatures": [
//                       {
//                           "keyid": "SHA256:jl3bwswu80PjjokCgh0o2w5c2U4LhQAE57gj9cz1kzA",
//                           "sig": "MEQCIGdFCa72n+vIbeujikn3ExFVcAX2rnuKWBBFWQlQIH1gAiBn5HXb7zzOTEFwAnHX8zrI8+2gPyDaxgy5gAMFq7fzhA=="
//                       }
//                   ]
//               },
//               "engines": {
//               }
//           }
//       },
//       "modified": "2022-06-19T02:40:54.045Z"
//     }"#;

//         let dep_one: NpmResolvedPackage = serde_json::from_str(&raw_dep_one).unwrap();
//         let dep_two: NpmResolvedPackage = serde_json::from_str(&raw_dep_two).unwrap();

//         let mut mock_network_adapter = MockNetworkAdapter::new();
//         mock_network_adapter
//             .expect_get_package()
//             .with(eq(dep_one.name.clone()))
//             .returning(|_| Ok(raw_dep_one.to_string()));
//         mock_network_adapter
//             .expect_get_package()
//             .with(eq(dep_two.name.clone()))
//             .returning(|_| Ok(raw_dep_two.to_string()));

//         let resp_writer = ResponseWriter::new();
//         let tar_ext = NpmTarExtractor::new();
//         let fetcher = PackageFetcher::new(&mock_network_adapter, &resp_writer, &tar_ext);

//         let mut packages_to_download: Vec<RangedDependencyTree> = Vec::new();

//         let mut deps = HashMap::new();
//         deps.insert(
//             dep_one.name.clone(),
//             VersionRangeSpecifier::new(String::from("^1.0.0")),
//         );

//         let res: Vec<RangedDependencyTree> = fetcher
//             .get_deps_with_versions(&Some(deps), &mut packages_to_download)
//             .await;

//         let mut expected_packages: Vec<RangedDependencyTree> = Vec::new();
//         expected_packages.push(RangedDependencyTree::new(
//             dep_one.name.as_str(),
//             dep_one
//                 .versions
//                 .get(&Version::new(String::from("1.0.0")))
//                 .unwrap(),
//             Box::new(vec![RangedDependencyTree::new(
//                 dep_two.name.as_str(),
//                 dep_two
//                     .versions
//                     .get(&Version::new(String::from("2.0.0")))
//                     .unwrap(),
//                 Box::new(Vec::new()),
//             )]),
//         ));

//         assert_eq!(res, expected_packages);
//     }

//     // #[tokio::test]
//     // async fn get_deps_with_versions_skips_downloaded_packages() {
//     //     let raw_dep_one = r#"{
//     //       "name": "dep-1",
//     //       "dist-tags": {
//     //           "latest": "1.0.0"
//     //       },
//     //       "versions": {
//     //           "1.0.0": {
//     //               "name": "dep-1",
//     //               "version": "1.0.0",
//     //               "dependencies": {
//     //                 "dep-2": "^2.0.0"
//     //               },
//     //               "devDependencies": {
//     //               },
//     //               "dist": {
//     //                   "shasum": "76b5055fbad8d294a86b6a949015e1c97b717c06",
//     //                   "tarball": "https://registry.npmjs.org/dep-1/-/dep-1-1.0.0.tgz",
//     //                   "integrity": "sha512-LEhnkAdJqic4Dbqn58A0y52IXoHWlsueqQkKfMfdEnIYG8A1sm/GHidKkS6yvXlMoRrkM34csHnXQtOqcb+Jzg==",
//     //                   "signatures": [
//     //                       {
//     //                           "keyid": "SHA256:jl3bwswu80PjjokCgh0o2w5c2U4LhQAE57gj9cz1kzA",
//     //                           "sig": "MEQCIGdFCa72n+vIbeujikn3ExFVcAX2rnuKWBBFWQlQIH1gAiBn5HXb7zzOTEFwAnHX8zrI8+2gPyDaxgy5gAMFq7fzhA=="
//     //                       }
//     //                   ]
//     //               },
//     //               "engines": {
//     //               }
//     //           }
//     //       },
//     //       "modified": "2022-06-19T02:40:54.045Z"
//     //     }"#;
//     //     let raw_dep_two = r#"{
//     //       "name": "dep-2",
//     //       "dist-tags": {
//     //           "latest": "2.0.0"
//     //       },
//     //       "versions": {
//     //           "2.0.0": {
//     //               "name": "dep-2",
//     //               "version": "2.0.0",
//     //               "dependencies": {
//     //               },
//     //               "devDependencies": {
//     //               },
//     //               "dist": {
//     //                   "shasum": "76b5055fbad8d294a86b6a949015e1c97b717c06",
//     //                   "tarball": "https://registry.npmjs.org/dep-2/-/dep-2-1.0.0.tgz",
//     //                   "integrity": "sha512-LEhnkAdJqic4Dbqn58A0y52IXoHWlsueqQkKfMfdEnIYG8A1sm/GHidKkS6yvXlMoRrkM34csHnXQtOqcb+Jzg==",
//     //                   "signatures": [
//     //                       {
//     //                           "keyid": "SHA256:jl3bwswu80PjjokCgh0o2w5c2U4LhQAE57gj9cz1kzA",
//     //                           "sig": "MEQCIGdFCa72n+vIbeujikn3ExFVcAX2rnuKWBBFWQlQIH1gAiBn5HXb7zzOTEFwAnHX8zrI8+2gPyDaxgy5gAMFq7fzhA=="
//     //                       }
//     //                   ]
//     //               },
//     //               "engines": {
//     //               }
//     //           }
//     //       },
//     //       "modified": "2022-06-19T02:40:54.045Z"
//     //     }"#;

//     //     let dep_one: NpmResolvedPackage = serde_json::from_str(&raw_dep_one).unwrap();
//     //     let dep_two: NpmResolvedPackage = serde_json::from_str(&raw_dep_two).unwrap();

//     //     let mut mock_network_adapter = MockNetworkAdapter::new();
//     //     mock_network_adapter
//     //         .expect_get_package()
//     //         .once()
//     //         .times(1)
//     //         .with(eq(dep_one.name.clone()))
//     //         .returning(|_| Ok(raw_dep_one.to_string()));
//     //     mock_network_adapter
//     //         .expect_get_package()
//     //         .once()
//     //         .times(0)
//     //         .with(eq(dep_two.name.clone()))
//     //         .returning(|_| Ok(raw_dep_two.to_string()));

//     //     let resp_writer = ResponseWriter::new();
//     //     let tar_ext = TarExtractor::new();
//     //     let fetcher = PackageFetcher::new(&mock_network_adapter, &resp_writer, &tar_ext);

//     //     let mut packages_to_download: Vec<DependencyTree> = Vec::new();
//     //     packages_to_download.insert(
//     //         dep_two.name.clone(),
//     //         dep_two
//     //             .versions
//     //             .get(&Version::new(String::from("2.0.0")))
//     //             .unwrap()
//     //             .to_owned(),
//     //     );

//     //     let mut deps = HashMap::new();
//     //     deps.insert(
//     //         dep_one.name.clone(),
//     //         VersionRangeSpecifier::new(String::from("^1.0.0")),
//     //     );

//     //     let res = fetcher
//     //         .get_deps_with_versions(&Some(deps), &mut packages_to_download)
//     //         .await;

//     //     let mut expected_packages = HashMap::new();
//     //     expected_packages.insert(
//     //         dep_one.name.clone(),
//     //         dep_one
//     //             .versions
//     //             .get(&Version::new(String::from("1.0.0")))
//     //             .unwrap()
//     //             .to_owned(),
//     //     );
//     //     expected_packages.insert(
//     //         dep_two.name.clone(),
//     //         dep_two
//     //             .versions
//     //             .get(&Version::new(String::from("2.0.0")))
//     //             .unwrap()
//     //             .to_owned(),
//     //     );

//     //     assert_eq!(res, &mut expected_packages);
//     // }
// }
