use std::{
    env::{self, current_dir},
    fs::read_dir,
    path::{Path, PathBuf},
};

use crate::path;

use super::state::Result;

/// The URL for the official typst packages repository.
pub const TYPST_PACKAGE_URL: &str = "https://github.com/typst/packages";
/// The default share directory on Linux for user-specific data files.
pub const DATA_HOME_SHARE: &str = ".local/share";
/// The default cache directory.
pub const CACHE_HOME: &str = ".cache";
/// The subdirectory within data and cache directories for typst packages.
pub const TYPST_PACKAGE_PATH: &str = typst_kit::package::DEFAULT_PACKAGES_SUBDIR;
/// The subdirectory for UTPM's own data files.
pub const UTPM_PATH: &str = "utpm";
/// The name of the manifest file.
pub const MANIFEST_FILE: &str = "typst.toml";
/// The subdirectory for locally cloned git packages.
pub const LOCAL_PACKAGES: &str = "git-packages";

/// Gets the path to the user's data directory.
///
/// This path can be overridden by setting the `UTPM_DATA_DIR` environment variable.
/// It is used for storing local packages.
pub fn get_data_dir() -> Result<PathBuf> {
    if let Ok(str) = env::var("UTPM_DATA_DIR") {
        Ok(PathBuf::from(str).canonicalize()?)
    } else if let Some(dir) = dirs::data_local_dir() {
        Ok(dir)
    } else {
        // Default on Linux: ~/.local/share
        let home = dirs::home_dir().ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Could not find home directory",
            )
        })?;
        Ok(path!(home, DATA_HOME_SHARE))
    }
}

/// Gets the path to the user's cache directory.
///
/// This path can be overridden by setting the `UTPM_CACHE_DIR` environment variable.
/// It is used for storing packages downloaded from the typst registry.
pub fn get_cache_dir() -> Result<PathBuf> {
    if let Ok(str) = env::var("UTPM_CACHE_DIR") {
        Ok(PathBuf::from(str).canonicalize()?)
    } else if let Some(dir) = dirs::cache_dir() {
        Ok(dir)
    } else {
        // Default fallback: ~/.cache
        let home = dirs::home_dir().ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Could not find home directory",
            )
        })?;
        Ok(path!(home, CACHE_HOME))
    }
}

/// Gets the path to the directory for downloaded packages from the typst registry.
pub fn c_packages() -> Result<PathBuf> {
    Ok(path!(get_cache_dir()?, TYPST_PACKAGE_PATH))
}

/// Gets the path to the directory for local packages.
pub fn d_packages() -> Result<PathBuf> {
    Ok(path!(get_data_dir()?, TYPST_PACKAGE_PATH))
}

/// Gets the path to UTPM's data directory.
///
/// Used for storing temporary files.
pub fn datalocalutpm() -> Result<PathBuf> {
    Ok(path!(get_data_dir()?, UTPM_PATH))
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
