use std::error;

use derive_more::Display;

use crate::npm::{NpmPackageVersion, NpmResolvedPackage, Version, VersionRangeSpecifier};

#[derive(Debug, Display, PartialEq)]
pub enum Error {
    VersionRangeResolveError,
}

impl error::Error for Error {}

/// Get a package and a version range,
/// and return the matching version. It will return None if the version is not found.
pub fn resolve_version_from_version_range<'a>(
    package: &'a NpmResolvedPackage,
    version_range: &VersionRangeSpecifier,
) -> Result<NpmPackageVersion, Error> {
    if *version_range == VersionRangeSpecifier::new(String::from("latest")) {
        let latest = package.dist_tags.get("latest");

        return match latest {
            Some(latest) => package
                .versions
                .get(latest)
                .map(|version| version.to_owned())
                .ok_or(Error::VersionRangeResolveError),
            None => Err(Error::VersionRangeResolveError),
        };
    }

    let version_req: node_semver::Range = match version_range.parse() {
        Ok(req) => req,
        Err(_) => return Err(Error::VersionRangeResolveError),
    };

    let mut matched_version: Result<NpmPackageVersion, Error> =
        Err(Error::VersionRangeResolveError);

    for vrs in package.versions.iter().rev() {
        matched_version = match vrs.0.parse::<node_semver::Version>() {
            Ok(parsed_version) if parsed_version.satisfies(&version_req) => package
                .versions
                .get(&Version::new(vrs.0.to_string()))
                .map(|version| version.to_owned())
                .ok_or(Error::VersionRangeResolveError),
            _ => Err(Error::VersionRangeResolveError),
        };

        if matched_version.is_ok() {
            break;
        }
    }

    matched_version
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_latest() {
        let package_json = r#"{
            "name": "is-even",
            "dist-tags": {
              "latest": "1.0.0"
            },
            "versions": {
              "0.1.2": {
                "name": "is-even",
                "version": "0.1.2",
                "dependencies": {
                  "is-odd": "^0.1.2"
                },
                "devDependencies": {
                  "mocha": "^3.4.2"
                },
                "dist": {
                  "shasum": "e0432a7379f2d20b6ebbc2cb11e69beaaf31cd63",
                  "tarball": "https://registry.npmjs.org/is-even/-/is-even-0.1.2.tgz",
                  "integrity": "sha512-Ft/TASRTCMIR20eeGtXIx7W+TfAbw/LkG7D3Pw5rncxaF1LCei/WVgO2qxsJiOROZb7JABl6Z8N2pEHjNONt9A==",
                  "signatures": [
                    {
                      "keyid": "SHA256:jl3bwswu80PjjokCgh0o2w5c2U4LhQAE57gj9cz1kzA",
                      "sig": "MEUCIGBqTtBRc6/6dqmI2lc+kmJRw4bB5kGHp5dM0fQH3V5pAiEA18DczU8X1bgDIkckzUOYpzWgZZJeQnpbgdPp9WtLnwY="
                    }
                  ]
                },
                "engines": {
                  "node": ">=0.10.0"
                }
              },
              "1.0.0": {
                "name": "is-even",
                "version": "1.0.0",
                "dependencies": {
                  "is-odd": "^0.1.2"
                },
                "devDependencies": {
                  "gulp-format-md": "^0.1.12",
                  "mocha": "^3.4.2"
                },
                "dist": {
                  "shasum": "76b5055fbad8d294a86b6a949015e1c97b717c06",
                  "tarball": "https://registry.npmjs.org/is-even/-/is-even-1.0.0.tgz",
                  "integrity": "sha512-LEhnkAdJqic4Dbqn58A0y52IXoHWlsueqQkKfMfdEnIYG8A1sm/GHidKkS6yvXlMoRrkM34csHnXQtOqcb+Jzg==",
                  "signatures": [
                    {
                      "keyid": "SHA256:jl3bwswu80PjjokCgh0o2w5c2U4LhQAE57gj9cz1kzA",
                      "sig": "MEQCIGdFCa72n+vIbeujikn3ExFVcAX2rnuKWBBFWQlQIH1gAiBn5HXb7zzOTEFwAnHX8zrI8+2gPyDaxgy5gAMFq7fzhA=="
                    }
                  ]
                },
                "engines": {
                  "node": ">=0.10.0"
                }
              }
            },
            "modified": "2022-06-19T02:40:54.045Z"
          }"#;

        let package: NpmResolvedPackage = serde_json::from_str(package_json).unwrap();

        let resolved = resolve_version_from_version_range(
            &package,
            &VersionRangeSpecifier::new(String::from("latest")),
        );

        assert_eq!(
            resolved,
            package
                .versions
                .get(&Version::new("1.0.0".to_string()))
                .map(|version| version.to_owned())
                .ok_or(Error::VersionRangeResolveError)
        );
    }

    #[test]
    fn resolves_semver() {
        let package_json = r#"{
            "name": "is-even",
            "dist-tags": {
              "latest": "1.0.0"
            },
            "versions": {
              "0.1.2": {
                "name": "is-even",
                "version": "0.1.2",
                "dependencies": {
                  "is-odd": "^0.1.2"
                },
                "devDependencies": {
                  "mocha": "^3.4.2"
                },
                "dist": {
                  "shasum": "e0432a7379f2d20b6ebbc2cb11e69beaaf31cd63",
                  "tarball": "https://registry.npmjs.org/is-even/-/is-even-0.1.2.tgz",
                  "integrity": "sha512-Ft/TASRTCMIR20eeGtXIx7W+TfAbw/LkG7D3Pw5rncxaF1LCei/WVgO2qxsJiOROZb7JABl6Z8N2pEHjNONt9A==",
                  "signatures": [
                    {
                      "keyid": "SHA256:jl3bwswu80PjjokCgh0o2w5c2U4LhQAE57gj9cz1kzA",
                      "sig": "MEUCIGBqTtBRc6/6dqmI2lc+kmJRw4bB5kGHp5dM0fQH3V5pAiEA18DczU8X1bgDIkckzUOYpzWgZZJeQnpbgdPp9WtLnwY="
                    }
                  ]
                },
                "engines": {
                  "node": ">=0.10.0"
                }
              },
              "1.0.0": {
                "name": "is-even",
                "version": "1.0.0",
                "dependencies": {
                  "is-odd": "^0.1.2"
                },
                "devDependencies": {
                  "gulp-format-md": "^0.1.12",
                  "mocha": "^3.4.2"
                },
                "dist": {
                  "shasum": "76b5055fbad8d294a86b6a949015e1c97b717c06",
                  "tarball": "https://registry.npmjs.org/is-even/-/is-even-1.0.0.tgz",
                  "integrity": "sha512-LEhnkAdJqic4Dbqn58A0y52IXoHWlsueqQkKfMfdEnIYG8A1sm/GHidKkS6yvXlMoRrkM34csHnXQtOqcb+Jzg==",
                  "signatures": [
                    {
                      "keyid": "SHA256:jl3bwswu80PjjokCgh0o2w5c2U4LhQAE57gj9cz1kzA",
                      "sig": "MEQCIGdFCa72n+vIbeujikn3ExFVcAX2rnuKWBBFWQlQIH1gAiBn5HXb7zzOTEFwAnHX8zrI8+2gPyDaxgy5gAMFq7fzhA=="
                    }
                  ]
                },
                "engines": {
                  "node": ">=0.10.0"
                }
              }
            },
            "modified": "2022-06-19T02:40:54.045Z"
          }"#;

        let package: NpmResolvedPackage = serde_json::from_str(&package_json).unwrap();

        let resolved = resolve_version_from_version_range(
            &package,
            &VersionRangeSpecifier::new(String::from("^0.1.2")),
        );

        assert_eq!(
            resolved,
            package
                .versions
                .get(&Version::new("0.1.2".to_string()))
                .map(|version| version.to_owned())
                .ok_or(Error::VersionRangeResolveError)
        );
    }
}
