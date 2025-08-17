use std::fs::read_to_string;
use std::path::PathBuf;
use std::{fs, path::Path};

use regex::Regex;
#[cfg(any(feature = "clone", feature = "publish", feature = "unlink"))]
use std::{io, result::Result as R};
use typst_kit::download::{DownloadState, Progress};
use typst_syntax::package::PackageManifest;

pub mod dryrun;
pub mod git;
pub mod macros;
pub mod output;
pub mod paths;
pub mod specs;
pub mod state;

use crate::utpm_bail;

use self::state::Result;

/// Recursively copies a directory from a source to a destination.
///
/// This function is based on the solution from:
/// <https://stackoverflow.com/questions/26958489/how-to-copy-a-folder-recursively-in-rust>
/// It has been edited to fit the needs of the CI environment.
pub fn copy_dir_all(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> io::Result<()> {
    fs::create_dir_all(&dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() && entry.file_name() != ".utpm" && entry.file_name() != "install" {
            copy_dir_all(entry.path(), dst.as_ref().join(entry.file_name()))?;
        } else {
            fs::copy(entry.path(), dst.as_ref().join(entry.file_name()))?;
        }
    }
    Ok(())
}

pub fn try_find_path(s: impl AsRef<Path>) -> Result<PathBuf> {
    let manifest_path = PathBuf::from_iter([s.as_ref(), "typst.toml".as_ref()]);

    if !manifest_path.try_exists()? {
        utpm_bail!(Manifest);
    }
    Ok(manifest_path)
}

pub fn try_find(s: impl AsRef<Path>) -> Result<PackageManifest> {
    let e = read_to_string(try_find_path(s)?)?;

    let f: PackageManifest = toml::from_str(&e)?;
    Ok(f)
}

/// Creates a symlink. This function is platform-specific.
///
/// On Unix systems, it creates a standard symbolic link.
#[cfg(unix)]
pub fn symlink_all(origin: impl AsRef<Path>, new_path: impl AsRef<Path>) -> R<(), std::io::Error> {
    use std::os::unix::fs::symlink;
    symlink(origin, new_path)
}

/// Creates a symlink. This function is platform-specific.
///
/// On Windows, it creates a directory symlink.
#[cfg(windows)]
pub fn symlink_all(origin: impl AsRef<Path>, new_path: impl AsRef<Path>) -> R<(), std::io::Error> {
    use std::os::windows::fs::symlink_dir;
    symlink_dir(origin, new_path)
}

/// Returns a regex for matching typst package specifications (`@namespace/name:version`).
#[cfg(any(feature = "clone", feature = "publish", feature = "unlink"))]
pub fn regex_package() -> Regex {
    Regex::new(r"^@([a-z]+)\/([a-z]+(?:\-[a-z]+)?)\:(\d+)\.(\d+)\.(\d+)$").unwrap()
}

/// Returns a regex for matching a typst package namespace (`@namespace`).
#[cfg(feature = "unlink")]
pub fn regex_namespace() -> Regex {
    Regex::new(r"^@([a-z]+)$").unwrap()
}

#[cfg(feature = "clone")]
pub fn regex_pkg_simple() -> Regex {
    Regex::new(r"^@(\w+)\/(\w+):(\d\.\d\.\d)$").unwrap()
}

#[cfg(feature = "clone")]
pub fn regex_pkg_simple_pkg() -> Regex {
    Regex::new(r"^(\w+):(\d\.\d\.\d)$").unwrap()
}

#[cfg(feature = "clone")]
pub fn regex_pkg_simple_ver() -> Regex {
    Regex::new(r"^(\d+)\.(\d+)\.(\d+)$").unwrap()
}

#[cfg(feature = "clone")]
pub fn regex_pkg_simple_name() -> Regex {
    Regex::new(r"^(\w+)$").unwrap()
}

/// Returns a regex for matching a typst package name (`@namespace/name`).
#[cfg(feature = "unlink")]
pub fn regex_packagename() -> Regex {
    Regex::new(r"^@([a-z]+)\/([a-z]+(?:\-[a-z]+)?)$").unwrap()
}

/// Returns a regex for matching a import of a package (`#import "@namespace/name:1.0.0"`).
pub fn regex_import() -> Regex {
    Regex::new("\\#import \"@([a-z]+)\\/([a-z]+(?:\\-[a-z]+)?)\\:(\\d+)\\.(\\d+)\\.(\\d+)\"")
        .unwrap()
}

//todo: impl
/// A progress indicator for package downloads.
///
/// (Warning) This is not fully implemented yet.
/// It is intended to provide feedback to the user during package downloads.
pub struct ProgressPrint {}

impl Progress for ProgressPrint {
    fn print_start(&mut self) {}

    fn print_progress(&mut self, _state: &DownloadState) {}

    fn print_finish(&mut self, _state: &DownloadState) {}
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn regex() {
        let re = regex_package();
        assert!(re.is_match("@preview/package:2.0.1"));
        assert!(!re.is_match("@preview/package-:2.0.1"));
        assert!(!re.is_match("@local/package-A:2.0.1"));
        assert!(re.is_match("@local/package-a:2.0.1"));
        assert!(!re.is_match("@local/p:1..1"));
        assert!(re.is_match("@a/p:1.0.1"));
        assert!(!re.is_match("@/p:1.0.1"));
        assert!(!re.is_match("p:1.0.1"));
        assert!(!re.is_match("@a/p"));
    }
}
