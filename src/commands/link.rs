use std::fs;
use std::path::PathBuf;
use tracing::instrument;

use crate::{
    utils::{
        copy_dir_all,
        dryrun::get_dry_run,
        paths::{c_packages, check_path_dir, d_packages, get_current_dir},
        state::Result,
        symlink_all, try_find,
    },
    utpm_bail, utpm_log,
};

use super::LinkArgs;

/// Links the current project to the local package directory, either by copying or symlinking.
#[instrument(skip(cmd))]
pub async fn run(cmd: &LinkArgs, path: Option<String>, pt: bool) -> Result<bool> {
    utpm_log!(trace, "executing link command");
    // Determine the source directory for the link operation.
    let curr = path.unwrap_or(get_current_dir()?);

    // Load the manifest and determine the namespace.
    let config = try_find(&curr)?;
    let namespace = cmd.namespace.clone().unwrap_or("local".into());

    // Construct the destination path for the package.
    let name = config.package.name;
    let version = config.package.version;
    let path = if namespace != "preview" {
        format!("{}/{}/{}/{}", d_packages()?, namespace, name, version)
    } else {
        format!("{}/{}/{}/{}", c_packages()?, namespace, name, version)
    };
    let path = PathBuf::from(path);

    // Check if the package already exists at the destination.
    if check_path_dir(&path) && !cmd.force {
        utpm_bail!(AlreadyExist, name.to_string(), version, "Info:".to_string());
    }

    if !get_dry_run() {
        fs::create_dir_all(path.parent().unwrap())?
    };

    // If force is used, remove the existing directory.
    if cmd.force && !get_dry_run() {
        fs::remove_dir_all(&path)?
    }

    // Create a symlink or copy the directory.
    if cmd.no_copy {
        if !get_dry_run() {
            symlink_all(&curr, &path)?
        };
        if pt {
            utpm_log!(
                info,
                "Project linked to: {}\nTry importing with: \n#import \"@{}/{}:{}\": *",
                path.to_string_lossy(),
                namespace,
                name,
                version
            );
        }
    } else {
        if !get_dry_run() {
            fs::create_dir_all(&path)?;
            copy_dir_all(&curr, &path)?
        };
        if pt {
            utpm_log!(
                info,
                "Project linked to: {}\nTry importing with: \n#import \"@{}/{}:{}\": *",
                path.to_string_lossy(),
                namespace,
                name,
                version
            );
        }
    }
    Ok(true)
}
