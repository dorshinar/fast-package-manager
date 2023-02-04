use std::collections::HashMap;

use derive_more::{Deref, Display, Into};
use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub struct NpmPackage {
    pub json: serde_json::Value,
    pub parsed: NpmResolvedPackage,
}

#[derive(Deserialize, Debug)]
pub struct NpmResolvedPackage {
    pub name: String,

    #[serde(rename(deserialize = "dist-tags"))]
    pub dist_tags: HashMap<String, Version>,

    pub versions: HashMap<Version, NpmPackageVersion>,
    pub modified: String,
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Clone)]
pub struct NpmPackageVersion {
    pub name: String,
    pub version: Version,
    pub dependencies: Option<HashMap<String, VersionRangeSpecifier>>,
    pub dist: NpmVersionDist,
    pub engines: Option<HashMap<String, VersionRangeSpecifier>>,
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Clone)]
pub struct NpmVersionDist {
    pub shasum: String,
    pub tarball: UrlString,
    pub integrity: String,
    pub signatures: Vec<NpmVersionDistSignature>,
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Clone)]
pub struct NpmVersionDistSignature {
    pub keyid: String,
    pub sig: String,
}

/// A semver-compatible version identifier.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Hash, Eq, Into, Display)]
#[serde(try_from = "String", into = "String")]
pub struct Version(String);

impl Version {
    pub fn new(value: String) -> Version {
        Self(value)
    }
}

impl TryFrom<String> for Version {
    type Error = VersionParseError;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Ok(Self(value))
    }
}

#[derive(Debug, PartialEq, Display)]
pub enum VersionParseError {
    InvalidFormat,
}
impl std::error::Error for VersionParseError {}

/// A semver-compatible version range.
/// Can be either a range - ">3.0.0", "1.2.4" or tag - "latest".
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Hash, Eq, Deref, Into)]
#[serde(try_from = "String", into = "String")]
pub struct VersionRangeSpecifier(String);

impl VersionRangeSpecifier {
    pub fn new(value: String) -> Self {
        Self(value)
    }
}

impl TryFrom<String> for VersionRangeSpecifier {
    type Error = VersionRangeSpecifierParseError;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Ok(Self(value))
    }
}

#[derive(Debug, PartialEq, Display)]
pub enum VersionRangeSpecifierParseError {
    InvalidFormat,
}
impl std::error::Error for VersionRangeSpecifierParseError {}

/// A string containing fully-formed URL.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Hash, Eq, Deref, Into)]
#[serde(try_from = "String", into = "String")]
pub struct UrlString(String);

impl UrlString {
    pub fn new(value: String) -> Self {
        Self(value)
    }
}

impl TryFrom<String> for UrlString {
    type Error = UrlStringParseError;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Ok(Self(value))
    }
}

#[derive(Debug, PartialEq, Display)]
pub enum UrlStringParseError {
    InvalidFormat,
}
impl std::error::Error for UrlStringParseError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_eq() {
        assert_eq!(
            Version::new(String::from("1.0.0"),),
            Version::new(String::from("1.0.0"))
        )
    }

    #[test]
    fn version_range_eq() {
        assert_eq!(
            VersionRangeSpecifier::new(String::from(">1.0.0")),
            VersionRangeSpecifier::new(String::from(">1.0.0"))
        )
    }

    #[test]
    fn version_range_ref_eq() {
        let range = VersionRangeSpecifier::new(String::from(">1.0.0"));
        let range_ref = &range;

        assert_eq!(
            *range_ref,
            VersionRangeSpecifier::new(String::from(">1.0.0"))
        )
    }
}
