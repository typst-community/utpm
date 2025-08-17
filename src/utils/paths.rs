use std::{
    env::{self, current_dir},
    fs::{self, read, read_dir, symlink_metadata},
    path::{self, Path},
    result::Result as R,
};

use dirs::cache_dir;

use super::state::Result;
use crate::utpm_bail;

/// The URL for the official typst packages repository.
pub const TYPST_PACKAGE_URL: &str = "https://github.com/typst/packages";
/// The default share directory on Linux for user-specific data files.
pub const DATA_HOME_SHARE: &str = "/.local/share";
/// The default cache directory.
pub const CACHE_HOME: &str = "~/.cache";
/// The default directory for SSH keys.
pub const SSH_HOME: &str = "/.ssh";
/// The subdirectory within data and cache directories for typst packages.
pub const TYPST_PACKAGE_PATH: &str = "/typst/packages";
/// The subdirectory for UTPM's own data files.
pub const UTPM_PATH: &str = "/utpm";
/// The name of the manifest file.
pub const MANIFEST_PATH: &str = "/typst.toml";
/// The subdirectory for locally cloned git packages.
pub const LOCAL_PACKAGES: &str = "/git-packages";

/// Gets the path to the user's data directory.
///
/// This path can be overridden by setting the `UTPM_DATA_DIR` environment variable.
/// It is used for storing local packages.
pub fn get_data_dir() -> Result<String> {
    match env::var("UTPM_DATA_DIR") {
        Ok(str) => Ok(path::absolute(str)?.to_str().unwrap().to_string()),
        _ => match dirs::data_local_dir() {
            Some(dir) => match dir.to_str() {
                Some(string) => Ok(String::from(string)),
                None => Ok(String::from(DATA_HOME_SHARE)), //default on linux
            },
            None => Ok(String::from(DATA_HOME_SHARE)),
        },
    }
}

/// Gets the path to the user's home directory.
///
/// This path can be overridden by setting the `UTPM_HOME_DIR` environment variable.
/// It is used for locating SSH keys for the `publish` command.
pub fn get_home_dir() -> Result<String> {
    match env::var("UTPM_HOME_DIR") {
        Ok(str) => Ok(path::absolute(str)?.to_str().unwrap().to_string()),
        _ => match dirs::home_dir() {
            Some(val) => match val.to_str() {
                Some(v) => Ok(String::from(v)),
                None => utpm_bail!(HomeDir),
            },
            None => utpm_bail!(HomeDir),
        },
    }
}

/// Gets the path to the user's cache directory.
///
/// This path can be overridden by setting the `UTPM_CACHE_DIR` environment variable.
/// It is used for storing packages downloaded from the typst registry.
pub fn get_cache_dir() -> Result<String> {
    match env::var("UTPM_CACHE_DIR") {
        Ok(str) => Ok(path::absolute(str)?.to_str().unwrap().to_string()),
        _ => Ok(cache_dir()
            .unwrap_or(CACHE_HOME.into())
            .to_str()
            .unwrap_or(CACHE_HOME)
            .into()),
    }
}

/// Gets the path to the user's SSH directory.
///
/// This path can be overridden by setting the `UTPM_SSH_DIR` environment variable.
/// It is used for locating SSH keys for the `publish` command.
pub fn get_ssh_dir() -> Result<String> {
    match env::var("UTPM_SSH_DIR") {
        Ok(str) => Ok(path::absolute(str)?.to_str().unwrap().to_string()),
        _ => Ok(get_home_dir()? + SSH_HOME),
    }
}

/// Gets the path to the directory for downloaded packages from the typst registry.
pub fn c_packages() -> Result<String> {
    Ok(get_cache_dir()? + TYPST_PACKAGE_PATH)
}

/// Gets the path to the directory for local packages.
pub fn d_packages() -> Result<String> {
    Ok(get_data_dir()? + TYPST_PACKAGE_PATH)
}

/// Gets the path to UTPM's data directory.
///
/// Used for storing temporary files.
pub fn datalocalutpm() -> Result<String> {
    Ok(get_data_dir()? + UTPM_PATH)
}

/// Gets the path to the default directory for cloned git packages.
pub fn default_typst_packages() -> Result<String> {
    Ok(datalocalutpm()? + LOCAL_PACKAGES)
}

/// Gets the current working directory.
///
/// This path can be overridden by setting the `UTPM_CURRENT_DIR` environment variable.
/// It is used for reading and writing the `typst.toml` manifest.
pub fn get_current_dir() -> Result<String> {
    match env::var("UTPM_CURRENT_DIR") {
        Ok(str) => Ok(path::absolute(str)?.to_str().unwrap().to_string()),
        _ => match current_dir() {
            Ok(val) => match val.to_str() {
                Some(v) => Ok(String::from(v)),
                None => utpm_bail!(CurrentDir),
            },
            Err(_) => utpm_bail!(CurrentDir),
        },
    }
}

/// Checks if a directory at the given path is not empty.
pub fn has_content(path: impl AsRef<Path>) -> Result<bool> {
    Ok(!fs::read_dir(path)?.collect::<R<Vec<_>, _>>()?.is_empty())
}

/// Gets the full path to the manifest file in the current directory.
pub fn current_package() -> Result<String> {
    Ok(get_current_dir()? + MANIFEST_PATH)
}

/// Checks if a directory exists at the given path.
pub fn check_path_dir(path: impl AsRef<Path>) -> bool {
    read_dir(path).is_ok()
}

/// Checks if a file exists at the given path.
pub fn check_path_file(path: impl AsRef<Path>) -> bool {
    read(path).is_ok()
}

/// Checks if a symlink exists at the given path.
pub fn check_existing_symlink(path: impl AsRef<Path>) -> bool {
    let x = match symlink_metadata(path) {
        Ok(val) => val,
        _ => return false,
    };
    x.file_type().is_symlink()
}
