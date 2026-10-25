use std::{
    borrow::Cow,
    path::{Path, PathBuf},
    str::FromStr,
};

use regex::Regex;
use tracing::instrument;
use typst_kit::{download::Downloader, package::PackageStorage};

use crate::{
    build,
    commands::get::get_packages_name_version,
    path,
    utils::{
        ProgressPrint, copy_dir_all,
        dryrun::get_dry_run,
        paths::{check_path_dir, get_current_dir, has_content, package_path},
        state::{Result, UtpmError},
        symlink_all,
    },
    utpm_bail, utpm_log,
};

use typst_syntax::package::{PackageSpec, PackageVersion};

use super::CloneArgs;

struct RawPkg<'a> {
    pub namespace: &'a str,
    pub package: &'a str,
    pub version: &'a str,
}

impl<'b> RawPkg<'b> {
    pub async fn from_str<'a: 'b>(s: &'a str) -> Result<Self> {
        // Use regex to parse the package specification string.
        // Package names may contain hyphens (e.g. `@preview/my-package:1.0.0`)
        // and version components may hold multiple digits (e.g. `1.10.0`).
        let re_all =
            Regex::new(r"^@([a-zA-Z]+)\/([a-zA-Z]+(?:\-[a-zA-Z]+)*):(\d+\.\d+\.\d+)$").unwrap();
        let re_name = Regex::new(r"^([a-zA-Z]+(?:\-[a-zA-Z]+)*):(\d+\.\d+\.\d+)$").unwrap();
        let re_namespace = Regex::new(r"^([a-zA-Z]+(?:\-[a-zA-Z]+)*)$").unwrap();

        if let Some(cap) = re_all.captures(s) {
            let (_, [namespace, package, version]) = cap.extract();

            Ok(Self::all(namespace, package, version))
        } else if let Some(cap) = re_name.captures(s) {
            let (_, [package, version]) = cap.extract();
            Ok(Self::pkg(package, version))
        } else if let Some(cap) = re_namespace.captures(s) {
            let (_, [package]) = cap.extract();
            Ok(Self::name(package).await?)
        } else {
            utpm_bail!(PackageNotValid);
        }
    }

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

    pub fn parse_version(&self) -> std::result::Result<PackageVersion, ecow::EcoString> {
        PackageVersion::from_str(self.version)
    }
}

/// Clones a typst package from the official repository or a local path.
#[instrument(skip(cmd))]
pub async fn run(cmd: &CloneArgs) -> Result<bool> {
    utpm_log!(trace, "executing clone command");
    // Determine the target path for the clone operation.
    let dst: Cow<'_, Path> = if let Some(path) = &cmd.path {
        Cow::Borrowed(path)
    } else {
        Cow::Owned(get_current_dir()?)
    };

    // Check if the target directory already has content.
    if has_content(&dst)? {
        utpm_log!(debug, "found content");
        if cmd.force {
            utpm_log!(warn, "force used, ignore content");
        } else {
            utpm_bail!(ContentFound);
        }
    }

    let package = &cmd.package;
    let pkg = RawPkg::from_str(package).await?;

    // Determine the local path for the package. UTPM only ever reads from
    // and downloads to the local package directory; the package cache
    // belongs to the Typst compiler and is never touched here.
    let package_dir = package_path()?;

    // Warn about shadowing the official registry when targeting @preview.
    if pkg.namespace == "preview" {
        utpm_log!(
            warn,
            "warning: cloning from the @preview namespace\n\nThe @preview namespace is the official Typst Universe registry. This package is downloaded from the registry into the local package directory ({}), which the Typst compiler searches before the package cache.\n\nAny local modification of that copy would take precedence over the official package for every project on this system. Track local packages with `utpm pkg list`.",
            package_dir.display()
        );
    }

    let local_path = path!(package_dir, pkg.namespace, pkg.package, pkg.version);

    // If the package already exists locally, copy or symlink it.
    if check_path_dir(&local_path) {
        utpm_log!(info, "Package found locally at {}", local_path.display());
        if cmd.download_only {
            utpm_log!(info, "download only, nothing to do.");
        } else if !cmd.redownload || pkg.namespace != "preview" {
            utpm_log!(info,
                "namespace" => pkg.namespace,
                "redownload" => cmd.redownload
            );
            if cmd.symlink {
                symlink_all(local_path, dst)?;
                utpm_log!(info, "symlinked!");
            } else {
                copy_dir_all(local_path, dst)?;
                utpm_log!(info, "copied!");
            }
        }
        return Ok(true);
    }

    // If the package needs to be downloaded.
    if cmd.redownload {
        utpm_log!(warn, "REDOWNLOAD IN WIP");
        // TODO: Implement removal of the existing directory for redownload.
    }

    // Prepare to download the package. Both the package path and the cache
    // point to the local package directory: downloads from the registry
    // land there, and the Typst package cache stays untouched.
    let pkg_sto = PackageStorage::new(
        Some(package_path()?),
        Some(package_path()?),
        Downloader::new(format!("utpm/{}", build::COMMIT_HASH)),
    );
    let printer = &mut ProgressPrint {};

    // Download the package.
    let cloned_path = if !get_dry_run() {
        pkg_sto.prepare_package(
            &PackageSpec {
                namespace: pkg.namespace.into(),
                name: pkg.package.into(),
                version: pkg.parse_version().unwrap(),
            },
            printer,
        )
    } else {
        Ok(PathBuf::new())
    };

    let Ok(cloned_path) = cloned_path else {
        utpm_bail!(PackageNotExist);
    };

    utpm_log!(info, "package downloaded", "path" => cloned_path.display().to_string());
    if cmd.download_only {
        utpm_log!(debug, "download complete, nothing to do");
        return Ok(true);
    }

    // Copy or symlink the downloaded package to the target path.
    if cmd.symlink {
        if !get_dry_run() {
            symlink_all(cloned_path, dst)?;
        }
        utpm_log!(info, "symlinked!");
    } else {
        if !get_dry_run() {
            copy_dir_all(cloned_path, dst)?;
        }
        utpm_log!(info, "copied!");
    }

    Ok(true)
}
