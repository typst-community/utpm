use inquire::Confirm;
use regex::Regex;
use std::fs;
use tracing::instrument;

use crate::{
    path,
    utils::{
        dryrun::get_dry_run,
        paths::{self, check_path_dir},
        regex_package,
        state::Result,
    },
    utpm_bail, utpm_log,
};

use super::UnlinkArgs;

/// Returns the package directory path for a given namespace.
///
/// # Arguments
/// * `namespace` - The package namespace ("preview" or other)
///
/// # Returns
/// The cache directory for "preview" namespace, or the data directory for others.
fn package_path(namespace: &str) -> Result<String> {
    if namespace == "preview" {
        Ok(paths::package_cache_path()?.to_string_lossy().to_string())
    } else {
        Ok(paths::package_path()?.to_string_lossy().to_string())
    }
}

/// Unlinks (removes) a package from local storage.
///
/// Supports removing:
/// - A specific version: `@namespace/package:1.0.0`
/// - All versions of a package: `@namespace/package`
/// - An entire namespace: `@namespace`
///
/// Prompts for confirmation unless `--yes` flag is used.
#[instrument(skip(cmd))]
pub async fn run(cmd: &UnlinkArgs) -> Result<bool> {
    utpm_log!(trace, "executing unlink command");
    let packages = &cmd.package;

    // Use regex to parse the package string, which can be a full package spec,
    // a package name, or just a namespace.
    let re_all = regex_package();
    let re_name = Regex::new(r"^@([a-zA-Z]+)\/([a-zA-Z]+(?:\-[a-zA-Z]+)?)$").unwrap();
    let re_namespace = Regex::new(r"^@([a-zA-Z]+)$").unwrap();

    let path = if let Some(cap) = re_all.captures(packages.as_str()) {
        let (_, [namespace, package, major, minor, patch]) = cap.extract();
        path!(
            package_path(namespace)?,
            namespace,
            package,
            format!("{major}.{minor}.{patch}")
        )
    } else if let Some(cap) = re_name.captures(packages.as_str()) {
        let (_, [namespace, package]) = cap.extract();
        path!(package_path(namespace)?, namespace, package)
    } else if let Some(cap) = re_namespace.captures(packages.as_str()) {
        let (_, [namespace]) = cap.extract();
        path!(package_path(namespace)?, namespace)
    } else {
        utpm_bail!(PackageNotValid);
    };

    // Check if the package directory exists.
    if !check_path_dir(&path) {
        utpm_bail!(PackageNotExist)
    }

    // Confirm with the user before deleting, unless `--yes` is provided.
    if !cmd.yes {
        match Confirm::new("This is irreversible. Are you sure to delete this?")
            .with_help_message(format!("You want to delete {packages}").as_str())
            .prompt()
        {
            Ok(_) => {
                utpm_log!(info, "Deleting {}", path.display());
                if !get_dry_run() {
                    fs::remove_dir_all(path)?;
                }
                Ok(true)
            },
            Err(_) => Ok(false),
        }
    } else {
        utpm_log!(info, "Deleting {}", path.display());
        if !get_dry_run() {
            fs::remove_dir_all(path)?;
        }
        Ok(true)
    }
}
