use ecow::EcoVec;
use ignore::{WalkBuilder, overrides::OverrideBuilder};
use std::fs::{copy, create_dir_all};
use std::path::PathBuf;
use std::{fs, path::Path};
use tracing::instrument;

use crate::{
    path,
    utils::{
        dryrun::get_dry_run,
        paths::{
            check_path_dir, check_path_file, get_current_dir, package_cache_path, package_path,
        },
        specs::Extra,
        state::Result,
        symlink_all, try_find,
    },
    utpm_bail, utpm_log,
};

use super::LinkArgs;

/// Links the current project to the local package directory, either by copying or symlinking.
#[instrument(skip(cmd))]
pub async fn run(cmd: &LinkArgs, path: &Option<String>, pt: bool) -> Result<bool> {
    utpm_log!(trace, "executing link command");
    // Determine the source directory for the link operation.
    let curr = if let Some(p) = path {
        PathBuf::from(p)
    } else {
        get_current_dir()?
    };

    // Load the manifest and determine the namespace.
    let config = try_find(&curr)?;
    let namespace = cmd.namespace.clone().unwrap_or("local".into());

    // Construct the destination path for the package.
    let name = config.package.name;
    let version = config.package.version;
    let destination = if namespace != "preview" {
        path!(
            package_path()?,
            &namespace,
            name.as_ref(),
            version.to_string()
        )
    } else {
        path!(
            package_cache_path()?,
            &namespace,
            name.as_ref(),
            version.to_string()
        )
    };

    // Check if the package already exists at the destination.
    let exists = check_path_dir(&destination);
    if exists && !cmd.force {
        utpm_bail!(AlreadyExist, name.to_string(), version, "Info:".to_string());
    }

    if !get_dry_run() {
        if exists {
            fs::remove_dir_all(&destination)?;
        }
        fs::create_dir_all(destination.parent().unwrap())?;

        if cmd.no_copy {
            symlink_all(&curr, &destination)?;
        } else {
            copy_tree(&curr, &destination, cmd, Extra::from(config.tool).exclude)?;
        }
    }

    if pt {
        utpm_log!(
            info,
            "Project linked to: {}\nTry importing with: \n#import \"@{}/{}:{}\": *",
            destination.display(),
            namespace,
            name,
            version
        );
    }
    Ok(true)
}

/// Copies `curr` into `destination`, honoring ignore files and manifest excludes.
fn copy_tree(
    curr: &Path,
    destination: &Path,
    cmd: &LinkArgs,
    excludes: Option<EcoVec<String>>,
) -> Result<()> {
    let mut wb: WalkBuilder = WalkBuilder::new(curr);
    let mut overr: OverrideBuilder = OverrideBuilder::new(curr);

    if let Some(excludes) = excludes {
        for exclude in excludes.iter() {
            overr.add(&format!("!{}", exclude))?;
        }
    }
    wb.overrides(overr.build()?);

    wb.ignore(cmd.ignore)
        .git_ignore(cmd.git_ignore)
        .git_global(cmd.git_global_ignore)
        .git_exclude(cmd.git_exclude);
    utpm_log!(info,
        "git_ignore" => cmd.git_ignore,
        "git_global_ignore" => cmd.git_global_ignore,
        "git_exclude" => cmd.git_exclude
    );

    if cmd.typst_ignore {
        let pathbuf = curr.join(".typstignore");
        if check_path_file(pathbuf) {
            utpm_log!(info, "Added .typstignore");
            wb.add_custom_ignore_filename(".typstignore");
        }
    }

    for result in wb.build().collect::<std::result::Result<Vec<_>, _>>()? {
        if let Some(file_type) = result.file_type() {
            let path: &Path = result.path();
            let relative = path.strip_prefix(curr).unwrap();
            let dest_path = destination.join(relative);
            utpm_log!("{}", dest_path.display());
            if file_type.is_dir() {
                create_dir_all(&dest_path)?;
            } else {
                copy(path, &dest_path)?;
            }
        }
    }
    Ok(())
}
