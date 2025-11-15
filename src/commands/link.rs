use ignore::{WalkBuilder, overrides::OverrideBuilder};
use std::fs::{copy, create_dir_all};
use std::path::PathBuf;
use std::{fs, path::Path};
use tracing::instrument;

use crate::{
    utils::{
        dryrun::get_dry_run,
        paths::{c_packages, check_path_dir, check_path_file, d_packages, get_current_dir},
        specs::Extra,
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
    let path_str = path.to_str().unwrap();

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
            // Use WalkBuilder to respect ignore files.
            let mut wb: WalkBuilder = WalkBuilder::new(&curr);
            let mut overr: OverrideBuilder = OverrideBuilder::new(&curr);

            // Add excludes from the manifest to the override builder.
            for exclude in Extra::from(config.tool).exclude.unwrap_or(vec![]) {
                overr.add(("!".to_string() + &exclude).as_str())?;
            }
            wb.overrides(overr.build()?);

            // Configure which ignore files to use.
            wb.ignore(cmd.ignore)
                .git_ignore(cmd.git_ignore)
                .git_global(cmd.git_global_ignore)
                .git_exclude(cmd.git_exclude);
            utpm_log!(info,
                "git_ignore" => cmd.git_ignore,
                "git_global_ignore" => cmd.git_global_ignore,
                "git_exclude" => cmd.git_exclude
            );

            // Add .typstignore if it exists and is enabled.
            if cmd.typst_ignore {
                let path_check = Path::new(&curr);
                let pathbuf = path_check.join(".typstignore");
                if check_path_file(pathbuf) {
                    utpm_log!(info, "Added .typstignore");
                    wb.add_custom_ignore_filename(".typstignore");
                }
            }

            // --- Copy Files ---
            for result in wb.build().collect::<std::result::Result<Vec<_>, _>>()? {
                if let Some(file_type) = result.file_type() {
                    let path: &Path = result.path();
                    let name: String = path.to_str().unwrap().to_string();
                    let l: String = name.replace::<&str>(&curr, path_str);
                    utpm_log!("{}", l);
                    if file_type.is_dir() {
                        create_dir_all(l)?;
                    } else {
                        copy(path, l)?;
                    }
                }
            }
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
