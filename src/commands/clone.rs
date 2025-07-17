use std::path::PathBuf;

use tracing::instrument;
use typst_kit::{download::Downloader, package::PackageStorage};

use crate::{
    build,
    utils::{
        copy_dir_all,
        paths::{c_packages, check_path_dir, d_packages, get_current_dir, has_content},
        regex_package,
        state::Result,
        symlink_all, ProgressPrint,
    },
    utpm_bail, utpm_log,
};

use typst_syntax::package::{PackageSpec, PackageVersion};

use super::CloneArgs;

use regex::Regex;

/// Clones a typst package from the official repository or a local path.
#[instrument]
pub fn run(cmd: &CloneArgs) -> Result<bool> {
    let path: PathBuf = if let Some(path) = &cmd.path {
        path.clone()
    } else {
        get_current_dir()?.into()
    };
    if has_content(&path)? {
        utpm_log!(debug, "found content");
        if cmd.force {
            utpm_log!(warn, "force used, ignore content");
        } else {
            utpm_bail!(ContentFound);
        }
    }
    let re: Regex = regex_package();
    let package: &String = &cmd.package;
    if let Some(cap) = re.captures(package) {
        let (_, [namespace, package, major, minor, patch]) = cap.extract();
        let val = format!(
            "{}/{namespace}/{package}/{major}.{minor}.{patch}",
            if namespace == "preview" {
                utpm_log!(info, "preview found, cache dir use");
                c_packages()?
            } else {
                utpm_log!(info, "no preview found, data dir use");
                d_packages()?
            }
        );
        if check_path_dir(&val) {
            if cmd.download_only {
                utpm_log!(info, "download only, nothing to do.");
                return Ok(true);
            }
            if !cmd.redownload || namespace != "preview" {
                utpm_log!(info,
                    "namespace" => namespace,
                    "redownload" => cmd.redownload
                );
                if cmd.symlink {
                    symlink_all(val, path)?;
                    utpm_log!(info, "symlinked!");
                } else {
                    copy_dir_all(val, path)?;
                    utpm_log!(info, "copied!");
                }
                return Ok(true);
            }
        }

        if cmd.redownload {}

        let pkg_sto = PackageStorage::new(
            Some(c_packages()?.into()),
            Some(d_packages()?.into()),
            Downloader::new(format!("utpm/{}", build::COMMIT_HASH)),
        );
        let printer = &mut ProgressPrint {};
        //todo: redownload = rm dir;
        return match pkg_sto.prepare_package(
            &PackageSpec {
                namespace: namespace.into(),
                name: package.into(),
                version: PackageVersion {
                    major: major.parse::<u32>().unwrap(),
                    minor: minor.parse::<u32>().unwrap(),
                    patch: patch.parse::<u32>().unwrap(),
                },
            },
            printer,
        ) {
            Ok(val) => {
                utpm_log!(info, "package downloaded", "path" => val.to_str().unwrap());
                if cmd.download_only {
                    utpm_log!(debug, "download complete, nothing to do");
                    return Ok(true);
                }

                if cmd.symlink {
                    symlink_all(val, path)?;
                    utpm_log!(info, "symlinked!");
                } else {
                    copy_dir_all(val, path)?;
                    utpm_log!(info, "copied!");
                }

                Ok(true)
            }
            Err(_) => {
                utpm_bail!(PackageNotExist);
            }
        };
    } else {
        utpm_log!(error, "package not found, input: {}", package);
        utpm_bail!(PackageNotValid);
    }
}