pub mod config;
pub mod dependency_resolver;
pub mod downloader;
pub mod http;
pub mod install_manifest;
pub mod install_package;
mod linker;
pub mod npm;
mod package_manifest;
mod resolve_version_range;

pub const STORE_FOLDER: &str = ".fpm";
pub const DEPS_FOLDER: &str = "node_modules";
