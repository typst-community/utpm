use inquire::Confirm;
use regex::Regex;
use std::{fs, path::Path};
use tracing::instrument;

use crate::{
    path,
    utils::{
        dryrun::get_dry_run,
        paths::{check_path_dir, has_content, package_cache_path, package_path},
        regex_package,
        state::Result,
    },
    utpm_bail, utpm_log,
};

use super::UnlinkArgs;

/// Unlinks (removes) a package from local storage.
///
/// Supports removing:
/// - A specific version: `@namespace/package:1.0.0`
/// - All versions of a package: `@namespace/package`
/// - An entire namespace: `@namespace`
///
/// By default the local package directory is targeted; pass `--cache` to
/// remove packages from the package cache shared with the Typst compiler.
///
/// When the removal leaves a parent directory empty (no versions left for a
/// package, or no packages left for a namespace), that parent is removed too,
/// up to the package directory root. Prompts for confirmation unless the
/// `--yes` flag is used.
#[instrument(skip(cmd))]
pub async fn run(cmd: &UnlinkArgs) -> Result<bool> {
    utpm_log!(trace, "executing unlink command");
    let packages = &cmd.package;

    // Deprecation notice for --local-only.
    if cmd.local_only {
        utpm_log!(
            warn,
            "--local-only is deprecated and has no effect: the local package directory is now the default. Pass --cache to target the package cache."
        );
    }

    // Use regex to parse the package string, which can be a full package spec,
    // a package name, or just a namespace.
    let re_all = regex_package();
    let re_name: Regex = Regex::new(r"^@([a-zA-Z]+)\/([a-zA-Z]+(?:\-[a-zA-Z]+)*)$").unwrap();
    let re_namespace = Regex::new(r"^@([a-zA-Z]+)$").unwrap();

    // Parse the spec and build the target path. The root is the local package
    // directory by default, or the package cache with --cache.
    let root = if cmd.cache {
        package_cache_path()?
    } else {
        package_path()?
    };

    let path = if let Some(cap) = re_all.captures(packages.as_str()) {
        let (_, [namespace, package, major, minor, patch]) = cap.extract();
        path!(
            root.clone(),
            namespace,
            package,
            format!("{major}.{minor}.{patch}")
        )
    } else if let Some(cap) = re_name.captures(packages.as_str()) {
        let (_, [namespace, package]) = cap.extract();
        path!(root.clone(), namespace, package)
    } else if let Some(cap) = re_namespace.captures(packages.as_str()) {
        let (_, [namespace]) = cap.extract();
        path!(root.clone(), namespace)
    } else {
        utpm_bail!(PackageNotValid);
    };

    // Check if the package directory exists.
    if !check_path_dir(&path) {
        utpm_bail!(PackageNotExist)
    }

    // Confirm with the user before deleting, unless `--yes` is provided.
    if !cmd.yes
        && Confirm::new("This is irreversible. Are you sure to delete this?")
            .with_help_message(format!("You want to delete {packages}").as_str())
            .prompt()
            .is_err()
    {
        return Ok(false);
    }

    utpm_log!(info, "Deleting {}", path.display());
    if !get_dry_run() {
        remove_package_dir(&path, &root)?;
    }
    Ok(true)
}

/// Deletes a package directory and then removes every now-empty parent
/// directory (package name, then namespace), stopping at `root` (exclusive).
///
/// After removing `path`, if its parent directory holds no more entries, that
/// parent is deleted too, cascading upwards until `root` is reached. `root`
/// itself is never removed.
///
/// # Arguments
/// * `path` - The package directory (version, package or namespace) to delete
/// * `root` - The package directory root at which the cleanup must stop
///
/// # Errors
/// Returns an error if a directory cannot be read or removed.
pub fn remove_package_dir(path: &Path, root: &Path) -> Result<()> {
    fs::remove_dir_all(path)?;

    let mut current = path.parent();
    while let Some(dir) = current {
        if dir == root || !check_path_dir(dir) || has_content(dir)? {
            break;
        }
        utpm_log!(info, "Removing empty directory {}", dir.display());
        fs::remove_dir(dir)?;
        current = dir.parent();
    }
    Ok(())
}
