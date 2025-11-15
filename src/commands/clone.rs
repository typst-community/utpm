use std::path::PathBuf;

use regex::Regex;
use tracing::instrument;
use typst_kit::{download::Downloader, package::PackageStorage};

use crate::{
    build,
    commands::get::get_packages_name_version,
    utils::{
        ProgressPrint, copy_dir_all,
        dryrun::get_dry_run,
        paths::{c_packages, check_path_dir, d_packages, get_current_dir, has_content},
        state::{Result, UtpmError},
        symlink_all,
    },
    utpm_bail, utpm_log,
};

use typst_syntax::package::{PackageSpec, PackageVersion};

use super::CloneArgs;

struct RawPck<'a> {
    pub namespace: &'a str,
    pub package: &'a str,
    pub version: &'a str,
}

impl<'b> RawPck<'b> {
    pub fn all<'a: 'b>(namespace: &'a str, package: &'a str, version: &'a str) -> Self {
        Self {
            namespace,
            package,
            version,
        }
    }
    pub fn pkg<'a: 'b>(package: &'a str, version: &'a str) -> Self {
        Self {
            namespace: "preview",
            package,
            version,
        }
    }
    pub async fn name<'a: 'b>(package: &'a str) -> Result<Self> {
        let packages = get_packages_name_version().await?;
        let version = match packages.get(package) {
            Some(e) => e.version.clone(),
            None => return Err(UtpmError::PackageNotExist),
        };
        let version = Box::leak(version.into_boxed_str());
        Ok(Self {
            namespace: "preview",
            package,
            version,
        })
    }
}

/// Clones a typst package from the official repository or a local path.
#[instrument(skip(cmd))]
pub async fn run<'a>(cmd: &'a CloneArgs) -> Result<bool> {
    utpm_log!(trace, "executing clone command");
    // Determine the target path for the clone operation.
    let path: PathBuf = cmd
        .path
        .clone()
        .unwrap_or_else(|| get_current_dir().unwrap().into());

    // Check if the target directory already has content.
    if has_content(&path)? {
        utpm_log!(debug, "found content");
        if cmd.force {
            utpm_log!(warn, "force used, ignore content");
        } else {
            utpm_bail!(ContentFound);
        }
    }

    // Use regex to parse the package specification string.
    let package: &String = &cmd.package;
    let pkg: RawPck;
    let re_all = Regex::new(r"^@(\w+)\/(\w+):(\d\.\d\.\d)$").unwrap();
    let re_name = Regex::new(r"^(\w+):(\d\.\d\.\d)$").unwrap();
    let re_namespace = Regex::new(r"^(\w+)$").unwrap();

    if let Some(cap) = re_all.captures(package.as_str()) {
        let (_, [namespace, packaged, version]) = cap.extract();
        pkg = RawPck::all(namespace, packaged, version)
    } else if let Some(cap) = re_name.captures(package.as_str()) {
        let (_, [packaged, version]) = cap.extract();
        pkg = RawPck::pkg(packaged, version);
    } else if let Some(cap) = re_namespace.captures(package.as_str()) {
        let (_, [packaged]) = cap.extract();
        pkg = RawPck::name(packaged).await?;
    } else {
        utpm_bail!(PackageNotValid);
    }

    // Determine the local path for the package based on its namespace.
    let val = format!(
        "{}/{}/{}/{}",
        pkg.namespace,
        pkg.package,
        pkg.version,
        if pkg.namespace == "preview" {
            utpm_log!(info, "preview found, cache dir use");
            c_packages()?
        } else {
            utpm_log!(info, "no preview found, data dir use");
            d_packages()?
        }
    );

    // If the package already exists locally, copy or symlink it.
    if check_path_dir(&val) {
        utpm_log!(info, "Package found locally at {}", val);
        if cmd.download_only {
            utpm_log!(info, "download only, nothing to do.");
            return Ok(true);
        }
        if !cmd.redownload || pkg.namespace != "preview" {
            utpm_log!(info,
                "namespace" => pkg.namespace,
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

    // If the package needs to be downloaded.
    if cmd.redownload {
        utpm_log!(warn, "REDOWNLOAD IN WIP");
        // TODO: Implement removal of the existing directory for redownload.
    }

    // Prepare to download the package.
    let pkg_sto = PackageStorage::new(
        Some(c_packages()?.into()),
        Some(d_packages()?.into()),
        Downloader::new(format!("utpm/{}", build::COMMIT_HASH)),
    );
    let printer = &mut ProgressPrint {};

    let (_, [major, minor, patch]) = Regex::new(r"^(\d+)\.(\d+)\.(\d+)$")
        .unwrap()
        .captures(pkg.version)
        .unwrap()
        .extract();

    // Download the package.
    let result_download = if !get_dry_run() {
        pkg_sto.prepare_package(
            &PackageSpec {
                namespace: pkg.namespace.into(),
                name: package.into(),
                version: PackageVersion {
                    major: major.parse::<u32>().unwrap(),
                    minor: minor.parse::<u32>().unwrap(),
                    patch: patch.parse::<u32>().unwrap(),
                },
            },
            printer,
        )
    } else {
        Ok(PathBuf::new())
    };

    return match result_download {
        Ok(val) => {
            utpm_log!(info, "package downloaded", "path" => val.to_str().unwrap());
            if cmd.download_only {
                utpm_log!(debug, "download complete, nothing to do");
                return Ok(true);
            }

            // Copy or symlink the downloaded package to the target path.
            if cmd.symlink {
                if !get_dry_run() {
                    symlink_all(val, path)?;
                }
                utpm_log!(info, "symlinked!");
            } else {
                if !get_dry_run() {
                    copy_dir_all(val, path)?;
                }
                utpm_log!(info, "copied!");
            }

            Ok(true)
        },
        Err(_) => {
            utpm_bail!(PackageNotExist);
        },
    };
}
