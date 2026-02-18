use std::{
    env::{self, current_dir},
    fs::read_dir,
    path::{Path, PathBuf},
};

use crate::path;

use super::state::Result;

/// The URL for the official typst packages repository.
pub const TYPST_PACKAGE_URL: &str = "https://github.com/typst/packages";
/// The subdirectory for UTPM's own data files.
pub const UTPM_SUBDIR: &str = "utpm";
/// The name of the manifest file.
pub const MANIFEST_FILE: &str = "typst.toml";
/// The subdirectory for locally cloned git packages.
pub const LOCAL_PACKAGES: &str = "git-packages";

/// Gets the path to the directory for downloaded packages from the typst registry.
///
/// This path can be overridden by setting the `TYPST_PACKAGE_CACHE_PATH` environment variable.
/// It is used for storing packages downloaded from the typst registry.
pub fn c_packages() -> Result<PathBuf> {
    let package_cache_path = env::var("TYPST_PACKAGE_CACHE_PATH")
        .ok()
        .map(PathBuf::from)
        .or_else(typst_kit::package::default_package_cache_path);
    let package_cache_path = package_cache_path.ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Could not find package cache directory",
        )
    })?;
    Ok(package_cache_path)
}

/// Gets the path to the directory for local packages.
///
/// This path can be overridden by setting the `TYPST_PACKAGE_PATH` environment variable.
/// It is used for storing local packages.
pub fn d_packages() -> Result<PathBuf> {
    let package_path = env::var("TYPST_PACKAGE_PATH")
        .ok()
        .map(PathBuf::from)
        .or_else(typst_kit::package::default_package_path);
    let package_path = package_path.ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Could not find package directory",
        )
    })?;
    Ok(package_path)
}

/// Gets the path to UTPM's data directory.
///
/// Used for storing temporary files.
///
/// This path can be overridden by setting the `UTPM_DATA_PATH` environment variable.
/// It is used for storing local packages.
pub fn datalocalutpm() -> Result<PathBuf> {
    let utpm_data_path = env::var("UTPM_DATA_PATH")
        .ok()
        .map(PathBuf::from)
        .or_else(|| dirs::data_dir().map(|data_dir| data_dir.join(UTPM_SUBDIR)));
    let utpm_data_path = utpm_data_path.ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Could not find utpm data directory",
        )
    })?;
    Ok(utpm_data_path)
}

/// Gets the path to the default directory for cloned git packages.
pub fn default_typst_packages() -> Result<PathBuf> {
    Ok(path!(datalocalutpm()?, LOCAL_PACKAGES))
}

/// Gets the current working directory.
///
/// This path can be overridden by setting the `UTPM_CURRENT_DIR` environment variable.
/// It is used for reading and writing the `typst.toml` manifest.
pub fn get_current_dir() -> Result<PathBuf> {
    if let Ok(str) = env::var("UTPM_CURRENT_DIR") {
        Ok(PathBuf::from(str).canonicalize()?)
    } else {
        Ok(current_dir()?)
    }
}

/// Checks if a directory at the given path is not empty.
pub fn has_content(path: impl AsRef<Path>) -> Result<bool> {
    Ok(read_dir(path)?.next().is_some())
}

/// Checks if a directory exists at the given path.
pub fn check_path_dir(path: impl AsRef<Path>) -> bool {
    path.as_ref().is_dir()
}

/// Checks if a file exists at the given path.
pub fn check_path_file(path: impl AsRef<Path>) -> bool {
    path.as_ref().is_file()
}
