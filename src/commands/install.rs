use std::fs;

use crate::{
    commands::LinkArgs,
    utils::{
        copy_dir_all,
        dryrun::get_dry_run,
        git::{clone_git, exist_git, project, sparse_checkout_git},
        paths::{MANIFEST_FILE, check_path_dir, check_path_file, package_path, utpm_data_path},
        state::Result,
        try_find,
    },
    utpm_log,
};

use super::{InstallArgs, link};
use tracing::instrument;

/// Installs a package from a git repository.
///
/// Clones the repository to a temporary directory, validates the package structure,
/// and then links it to the local package directory.
///
/// # Note
/// This command requires git to be installed and cannot run in dry-run mode.
#[instrument]
pub async fn run(cmd: &InstallArgs) -> Result<bool> {
    if get_dry_run() {
        utpm_log!(warn, "Dry-run, can't do anything");
        return Ok(true);
    }

    exist_git()?;

    utpm_log!(trace, "executing init command for install");

    let path = utpm_data_path()?.join("tmp");
    if check_path_dir(&path) && !get_dry_run() {
        fs::remove_dir_all(&path)?;
    }

    // Determine the source path for the installation.
    utpm_log!(
        debug,
        "url is set to {}, creating {}",
        &cmd.url,
        path.display().to_string()
    );

    // If a URL is provided, clone or copy the repository.
    fs::create_dir_all(&path)?;
    project().lock().unwrap().0 = path.clone();

    let url = &cmd.url;

    // Handle git and http(s) URLs.
    if url.starts_with("git") || url.starts_with("http") {
        // If a subdirectory was specified, perform a sparse checkout instead.
        if let Some(subdir) = &cmd.subdir {
            sparse_checkout_git(url, &path.to_string_lossy(), subdir, cmd.branch.as_deref())?;
        } else {
            clone_git(url, &path.to_string_lossy(), cmd.branch.as_deref())?;
        }
    } else {
        // Handle local paths.
        copy_dir_all(url, &path)?;
    }

    // Modify path if a subdirectory was specified so that manifest can be found.
    // Trimming starting slash is necessary otherwise Rust treats it as absolute path.
    let path = match &cmd.subdir {
        Some(s) => path.join(s.trim_start_matches('/')),
        _ => path,
    };

    // Check for a manifest file in the source directory.
    let typstfile = path.join(MANIFEST_FILE);
    if !check_path_file(&typstfile) {
        utpm_log!(
            "{} (no manifest found at {:?})",
            format!("x {}", url),
            typstfile
        );
        return Ok(false);
    }

    utpm_log!(trace, "Before loading manifest...", "path" => path.display().to_string());
    // Load the manifest and extract UTPM-specific configurations.
    let file = try_find(&path)?;
    let namespace = cmd.namespace.as_deref().unwrap_or("local");
    utpm_log!(trace, "After loading manifest...");
    // Check if the package is already installed.
    if check_path_dir(format!(
        "{}/{}/{}/{}",
        package_path()?.display(),
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
        git_exclude: false,
        git_global_ignore: false,
        git_ignore: false,
        ignore: false,
        typst_ignore: false,
    };

    link::run(&lnk, &Some(path.display().to_string()), false).await?;
    fs::remove_dir_all(&path)?;

    utpm_log!(info, "+ {}:{}", file.package.name, file.package.version);
    Ok(true)
}
