use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub struct NpmResolvedPackage {
    pub _id: String,
    pub _rev: String,
    pub name: String,
    pub description: String,

    #[serde(rename(deserialize = "dist-tags"))]
    pub dist_tags: HashMap<String, Version>,
    pub versions: HashMap<Version, NpmPackageVersion>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct NpmPackageVersion {
    pub name: String,
    pub version: Version,
    pub dependencies: Option<HashMap<String, VersionRangeSpecifier>>,

    #[serde(rename(deserialize = "devDependencies"))]
    pub dev_dependencies: HashMap<String, VersionRangeSpecifier>,

    pub _id: String,
    pub _shasum: Option<String>,
    pub dist: NpmVersionDist,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct NpmVersionDist {
    pub shasum: ChecksumString,
    pub tarball: UrlString,
    pub integrity: String,
    pub signatures: Vec<NpmVersionDistSignatures>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct NpmVersionDistSignatures {
    pub keyid: String,
    pub sig: String,
}

type Version = String;
type VersionRangeSpecifier = String;
type ChecksumString = String;
type UrlString = String;
