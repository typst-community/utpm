use std::{fs::write, path::Path};

use ignore::{WalkBuilder, overrides::OverrideBuilder};
use regex::Regex;
use tracing::instrument;

use std::result::Result as R;

use crate::{
    commands::get::get_packages_name_version,
    path,
    utils::{
        dryrun::get_dry_run,
        paths::{get_current_dir, package_path},
        state::{Result, UtpmError},
    },
    utpm_bail, utpm_log,
};

use super::SyncArgs;

/// Synchronizes package dependencies to their latest versions.
///
/// Can either sync all `.typ` files in the current directory, or only specified files.
/// When in check-only mode, reports available updates without modifying files.
#[instrument(skip(cmd))]
pub async fn run<'a>(cmd: &'a SyncArgs) -> Result<bool> {
    if cmd.files.is_empty() {
        utpm_log!(trace, "Running default check...");
        default_run(cmd.check_only).await?;
        Ok(true)
    } else {
        utpm_log!(trace, "Running specific check...", "files" => cmd.files.join(","));
        files_run(&cmd.files, cmd.check_only).await?;
        Ok(true)
    }
}

/// Runs sync on all `.typ` files in the current directory.
///
/// # Arguments
/// * `cmd` - If true, only checks for updates without modifying files
async fn default_run(cmd: bool) -> Result<bool> {
    let path = &get_current_dir()?;
    let wb: WalkBuilder = WalkBuilder::new(path);
    let mut overr: OverrideBuilder = OverrideBuilder::new(path);
    overr.add("*.typ")?;
    for result in wb.build().collect::<R<Vec<_>, _>>()? {
        if let Some(file_type) = result.file_type()
            && !file_type.is_dir()
        {
            utpm_log!(info, "Syncing {}...", result.file_name().to_string_lossy());
            file_run(result.path(), cmd).await?;
        }
    }
    Ok(true)
}

// TODO: Comments using utpm_log
async fn file_run(path: impl AsRef<Path>, comment_only: bool) -> Result<bool> {
    let path = path.as_ref();
    let re = Regex::new(
        r#"\#import \"@([a-zA-Z]+)\/([a-zA-Z]+(?:\-[a-zA-Z]+)?)\:(\d+)\.(\d+)\.(\d+)\""#,
    )
    .unwrap();
    let content_bytes = match std::fs::read(path) {
        Ok(bytes) => Ok(bytes),
        Err(e) => {
            utpm_log!(warn, "Skipping file {:?}, could not read: {}", path, e);
            Err(e)
        },
    }?;

    let mut string = match String::from_utf8(content_bytes) {
        Ok(s) => Ok(s),
        Err(e) => {
            utpm_log!(warn, "Skipping non-UTF-8 file: {:?}", path);
            Err(e)
        },
    }?;

    // Creating offset if there is multiple import and they don't have the same length
    let mut offset: isize = 0;
    let mut modified = false;

    // Matching regex
    let nws = string.clone();
    let res = re.find_iter(nws.as_str());
    let res2 = re.captures_iter(nws.as_str());
    utpm_log!(info, "Found imports");
    for e in res.zip(res2) {
        modified = true;
        // Set positions to rewrite the version
        let start = (e.0.start() as isize + offset) as usize;
        let end = (e.0.end() as isize + offset) as usize;
        let range = start..end;
        utpm_log!(trace, "Range: {:?}", range);

        // Extract infos from the import
        let (_, [namespace, package, major, minor, patch]) = e.1.extract();
        utpm_log!(
            trace,
            "Last import: @{namespace}/{package}:{major}.{minor}.{}",
            patch
        );
        let version = if namespace == "preview" {
            let pkgs = get_packages_name_version().await?;
            if let Some(pkg) = pkgs.get(package) {
                Ok::<std::string::String, UtpmError>(pkg.version.clone())
            } else {
                utpm_bail!(PackageNotExist);
            }
        } else {
            let r = std::fs::read_dir(path!(package_path()?, namespace, package))?;
            let mut list_dir = r
                .into_iter()
                .filter_map(|a| a.ok())
                .filter_map(|a| a.file_name().into_string().ok())
                .collect::<Vec<_>>();
            list_dir.sort();
            let var = match list_dir.last() {
                Some(e) => Ok::<std::string::String, UtpmError>(e.clone()),
                None => utpm_bail!(PackageNotExist),
            }?;
            Ok(var)
        }?;

        // Replace the import by the new

        let new_import: String = format!(
            "#import \"@{namespace}/{package}:{}\" {}",
            if comment_only {
                format!("{major}.{minor}.{patch}")
            } else {
                version.clone()
            },
            if comment_only {
                format!("/* New version available: {} */", version)
            } else {
                format!("/* From {major}.{minor}.{patch} */")
            }
        );
        utpm_log!(info, new_import);
        let old_len = string.len();
        if !comment_only {
            string.replace_range(range, &new_import);
        }

        // Set the offet and the new string.
        offset += (string.len() as isize) - (old_len as isize);
    }

    if modified {
        if !get_dry_run() {
            write(path, &string)?
        };
        utpm_log!(info, "{} written", path.display());
    }
    Ok(true)
}

async fn files_run(files: &Vec<String>, cmd: bool) -> Result<bool> {
    utpm_log!(trace, "executing files_run for sync command");
    for file in files {
        let path = Path::new(file.as_str());
        file_run(path, cmd).await?;
    }
    Ok(true)
}
