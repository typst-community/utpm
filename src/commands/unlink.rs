use inquire::Confirm;
use std::fs;
use tracing::instrument;

use crate::{
    format_package,
    utils::{
        paths::check_path_dir,
        regex_namespace, regex_package, regex_packagename,
        state::Result,
    },
    utpm_bail,
};

use super::UnlinkArgs;

/// Unlinks/deletes a package from the local storage.
#[instrument(skip(cmd))]
pub fn run(cmd: &UnlinkArgs) -> Result<bool> {
    let packages = &cmd.package;

    // Use regex to parse the package string, which can be a full package spec,
    // a package name, or just a namespace.
    let re_all = regex_package();
    let re_name = regex_packagename();
    let re_namespace = regex_namespace();
    let path: String;

    if let Some(cap) = re_all.captures(packages.as_str()) {
        let (_, [namespace, package, major, minor, patch]) = cap.extract();
        path = format_package!(namespace, package, major, minor, patch);
    } else if let Some(cap) = re_name.captures(packages.as_str()) {
        let (_, [namespace, package]) = cap.extract();
        path = format_package!(namespace, package);
    } else if let Some(cap) = re_namespace.captures(packages.as_str()) {
        let (_, [namespace]) = cap.extract();
        path = format_package!(namespace);
    } else {
        utpm_bail!(PackageNotValid);
    }

    // Check if the package directory exists.
    if !check_path_dir(&path) {
        utpm_bail!(PackageNotExist)
    }

    // Confirm with the user before deleting, unless `--yes` is provided.
    if !cmd.yes {
        match Confirm::new("Are you sure to delete this? This is irreversible.")
            .with_help_message(format!("You want to delete {packages}").as_str())
            .prompt()
        {
            Ok(_) => {
                fs::remove_dir_all(path)?;
                Ok(true)
            }
            Err(_) => Ok(false),
        }
    } else {
        fs::remove_dir_all(path)?;
        Ok(true)
    }
}
