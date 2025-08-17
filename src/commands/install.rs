use std::fs;

use crate::{
    commands::LinkArgs,
    load_manifest,
    utils::{
        copy_dir_all,
        dryrun::get_dry_run,
        git::{clone_git, exist_git, project},
        paths::{MANIFEST_PATH, check_path_dir, check_path_file, d_packages, datalocalutpm},
        state::Result,
    },
    utpm_log,
};

use super::{InstallArgs, link};
use tracing::instrument;

#[instrument]
pub async fn run(cmd: &InstallArgs) -> Result<bool> {
    if get_dry_run() {
        utpm_log!(warn, "Dry-run, can't do anything");
        return Ok(true);
    }

    exist_git()?;

    utpm_log!(trace, "executing init command for install");

    let path = format!("{}/tmp", datalocalutpm()?);
    if check_path_dir(&path) && !get_dry_run() {
        fs::remove_dir_all(&path)?;
    }

    // Determine the source path for the installation.
    utpm_log!(debug, "url is set to {}, creating {}", &cmd.url, path);

    // If a URL is provided, clone or copy the repository.
    fs::create_dir_all(&path)?;

    project().lock().unwrap().0 = path.clone();

    let url = cmd.url.clone();

    // Handle git and http(s) URLs.
    if url.starts_with("git") || url.starts_with("http") {
        clone_git(url.as_str(), path.as_str())?;
    } else {
        // Handle local paths.
        copy_dir_all(&url, &path)?;
    }
    // Check for a manifest file in the source directory.
    let typstfile = path.clone() + MANIFEST_PATH;
    if !check_path_file(&typstfile) {
        utpm_log!("{}", format!("x {}", url));
        return Ok(false);
    }

    utpm_log!(trace, "Before loading manifest...", "path" => path);
    // Load the manifest and extract UTPM-specific configurations.
    let file = load_manifest!(&path);
    let namespace = cmd.namespace.clone().unwrap_or("local".into());
    utpm_log!(trace, "After loading manifest...");
    // Check if the package is already installed.
    if check_path_dir(format!(
        "{}/{}/{}/{}",
        d_packages()?,
        namespace,
        &file.package.name,
        &file.package.version
    )) {
        utpm_log!(
            "{}",
            format!("~ {}:{}", file.package.name, file.package.version)
        );
        return Ok(true);
    }

    utpm_log!("{}", format!("Installing {}...", file.package.name));

    // Link the installed package and clean up temporary files.
    let lnk = LinkArgs {
        force: false,
        no_copy: false,
        namespace: cmd.namespace.clone(),
    };

    link::run(&lnk, Some(path.clone()), false).await?;
    fs::remove_dir_all(&path)?;

    utpm_log!(info, "+ {}:{}", file.package.name, file.package.version);
    Ok(true)
}
