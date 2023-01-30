use std::collections::HashMap;

use serde::{Deserialize, Serialize};

pub struct NpmPackage {
    pub json: serde_json::Value,
    pub parsed: NpmResolvedPackage,
}

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

pub type Version = String;
pub type VersionRangeSpecifier = String;
pub type ChecksumString = String;
pub type UrlString = String;
