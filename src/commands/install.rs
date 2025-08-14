use std::{env, fs, path::Path};

use crate::{
    commands::LinkArgs,
    load_manifest,
    utils::{
        copy_dir_all,
        dryrun::get_dry_run,
        paths::{check_path_dir, check_path_file, d_packages, datalocalutpm, get_ssh_dir},
        state::Result,
    },
    utpm_log,
};

use git2::{Cred, FetchOptions, RemoteCallbacks, build::RepoBuilder};
use tracing::instrument;

use super::{InstallArgs, link};


#[instrument]
pub async fn run(cmd: &InstallArgs) -> Result<bool> {
    if get_dry_run() {
        utpm_log!(warn, "Dry-run, can't do anything");
        return Ok(true);
    }
    utpm_log!(trace, "executing init command for install");

    let path = format!("{}/tmp", datalocalutpm()?);
    if check_path_dir(&path) && !get_dry_run() {
        fs::remove_dir_all(&path)?;
    }

    // Determine the source path for the installation.
    utpm_log!(debug, "url is set to {}, creating {}", &cmd.url, path);

    // If a URL is provided, clone or copy the repository.
    // TODO: Too bloated here, everything needs to be passed on git directly
    fs::create_dir_all(&path)?;
    let sshpath = get_ssh_dir()?;
    let ed: String = sshpath.clone() + "/id_ed25519";
    let rsa: String = sshpath + "/id_rsa";
    // Determine the SSH key to use.
    let val: String = match env::var("UTPM_KEYPATH") {
        Ok(val) => val,
        Err(_) => {
            if check_path_file(&ed) {
                ed
            } else {
                rsa
            }
        }
    };

    let url = cmd.url.clone();

    // Handle git and http(s) URLs.
    if url.starts_with("git") || url.starts_with("http") {
        let mut callbacks = RemoteCallbacks::new();
        callbacks.credentials(|_, username_from_url, _| {
            let binding =
                env::var("UTPM_USERNAME").unwrap_or(username_from_url.unwrap_or("git").to_string());
            let username = binding.as_str();
            match Cred::ssh_key_from_agent(username) {
                Ok(cred) => Ok(cred),
                Err(_) => Cred::ssh_key(
                    username,
                    None,
                    Path::new(&val),
                    Some(
                        env::var("UTPM_PASSPHRASE")
                            .unwrap_or(String::new())
                            .as_str(),
                    ),
                ),
            }
        });

        let mut fo = FetchOptions::new();
        fo.remote_callbacks(callbacks);

        let mut builder = RepoBuilder::new();
        builder.fetch_options(fo);
        builder.clone(&url, Path::new(&path))?;
    } else {
        // Handle local paths.
        copy_dir_all(&url, &path)?;
    }
    // Check for a manifest file in the source directory.
    let typstfile = path.clone() + "/typst.toml";
    if !check_path_file(&typstfile) {
        utpm_log!("{}", format!("x {}", url));
        return Ok(false);
    }

    // Load the manifest and extract UTPM-specific configurations.
    let file = load_manifest!(&path);
    let namespace = cmd.namespace.clone().unwrap_or("local".into());

    // Check if the package is already installed.
    if check_path_dir(&format!(
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
        namespace: cmd.namespace.clone()
    };

    link::run(&lnk, Some(path.clone()), false).await?; //TODO: change here too
    fs::remove_dir_all(&path)?;

    utpm_log!(info, "+ {}:{}", file.package.name, file.package.version);
    Ok(true)
}
