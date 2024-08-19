use tracing::{debug, info, instrument, warn};
use typst_kit::{
    download::{Downloader, ProgressSink},
    package::PackageStorage,
};

use crate::utils::{
    copy_dir_all,
    paths::{c_packages, check_path_dir, d_packages, get_current_dir, has_content},
    state::{Error, ErrorKind, Result},
    symlink_all,
};

use typst_syntax::package::{PackageSpec, PackageVersion};

use super::CloneArgs;

use regex::Regex;

#[instrument]
pub fn run(cmd: &CloneArgs) -> Result<bool> {
    if has_content(get_current_dir()?)? {
        debug!("found content");
        if cmd.force {
            warn!("force used, ignore content");
        } else {
            return Err(Error::new(
                ErrorKind::ContentFound,
                "Content found, cancelled",
            ));
        }
    }
    let re = Regex::new(r"@([a-z]+)\/([a-z]+)\:(\d+)\.(\d+)\.(\d+)").unwrap();
    let package: &String = &cmd.package;
    if let Some(cap) = re.captures(package) {
        let (_, [namespace, package, major, minor, patch]) = cap.extract();
        let val = format!(
            "{}/{namespace}/{package}/{major}.{minor}.{patch}",
            if namespace == "preview" {
                c_packages()?
            } else {
                d_packages()?
            }
        );
        if check_path_dir(&val) {
            //todo: trace
            if cmd.download_only {
                return Ok(true);
            }
            if !cmd.redownload || namespace != "preview" {
                if cmd.symlink {
                    symlink_all(
                        val,
                        get_current_dir()?
                            + "/"
                            + package
                            + ":"
                            + major
                            + "."
                            + minor
                            + "."
                            + patch,
                    )?;
                    info!("symlinked!");
                } else {
                    copy_dir_all(val, get_current_dir()?)?;
                    info!("copied!");
                }
                return Ok(true);
            }
        }

        let pkg_sto = PackageStorage::new(None, None, Downloader::new("utpm/latest"));
        let sink = &mut ProgressSink {};
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
            sink,
        ) {
            Ok(val) => {
                info!("package downloaded");
                if cmd.download_only {
                    debug!("download complete, nothing to do");
                    return Ok(true);
                }

                if cmd.symlink {
                    symlink_all(val, get_current_dir()?)?;
                    info!("symlinked!");
                } else {
                    copy_dir_all(val, get_current_dir()?)?;
                    info!("copied!");
                }

                Ok(true)
            }
            Err(_) => {
                return Err(Error::new(
                    ErrorKind::PackageNotExist,
                    "This package doesn't exist. Verify on https://typst.app/universe to see if the package exist and/or the version is correct.", //todo
                ));
            }
        };
    } else {
        return Err(Error::new(
            ErrorKind::PackageNotValid,
            "Can't extract your package. Example of a package: @namespace/package:1.0.0",
        ));
    }
}
