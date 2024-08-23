use std::{fs, path::Path};

use std::io;

#[cfg(any(feature = "clone", feature = "publish", feature = "unlink"))]
use regex::Regex;
use typst_kit::download::{DownloadState, Progress};

pub mod macros;
pub mod paths;
pub mod specs;
pub mod state;

/// Copy all subdirectories from a point to an other
/// From https://stackoverflow.com/questions/26958489/how-to-copy-a-folder-recursively-in-rust
/// Edited to prepare a ci version
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

/// Implementing a symlink function for all platform (unix version)
#[cfg(unix)]
pub fn symlink_all(
    origin: impl AsRef<Path>,
    new_path: impl AsRef<Path>,
) -> Result<(), std::io::Error> {
    use std::os::unix::fs::symlink;
    symlink(origin, new_path)
}

/// Implementing a symlink function for all platform (windows version)
#[cfg(windows)]
pub fn symlink_all(
    origin: impl AsRef<Path>,
    new_path: impl AsRef<Path>,
) -> Result<(), std::io::Error> {
    use std::os::windows::fs::symlink_dir;
    symlink_dir(origin, new_path)
}

#[cfg(any(feature = "clone", feature = "publish", feature = "unlink"))]
pub fn regex_package() -> Regex {
    Regex::new(r"^@([a-z]+)\/([a-z]+(?:\-[a-z]+)?)\:(\d+)\.(\d+)\.(\d+)$").unwrap()
}
#[cfg(any(feature = "unlink"))]
pub fn regex_namespace() -> Regex {
    Regex::new(r"^@([a-z]+)$").unwrap()
}

#[cfg(any(feature = "unlink"))]
pub fn regex_packagename() -> Regex {
    Regex::new(r"^@([a-z]+)\/([a-z]+(?:\-[a-z]+)?)$").unwrap()
}

//todo: impl
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
