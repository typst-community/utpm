use std::{env, fs, path::Path};

use crate::{
    commands::LinkArgs,
    load_manifest,
    utils::{
        copy_dir_all,
        paths::{
            check_path_dir, check_path_file, d_packages, datalocalutpm, get_current_dir,
            get_ssh_dir,
        },
        specs::Extra,
        state::{Result, UtpmError},
    },
    utpm_log,
};

use git2::{build::RepoBuilder, Cred, FetchOptions, RemoteCallbacks};
use tracing::instrument;
use typst_project::heuristics::MANIFEST_FILE;

use super::{link, InstallArgs};

/// Entry point for the install command.
///
/// Cleans up temporary directories before starting the installation process.
#[instrument]
pub async fn run(cmd: &InstallArgs) -> Result<bool> {
    let path = format!("{}/tmp", datalocalutpm()?);
    if check_path_dir(&path) {
        fs::remove_dir_all(path)?;
    }
    init(cmd, 0).await?;
    Ok(true)
}

/// Installs dependencies recursively.
///
/// If a URL is provided, it clones the repository from the given URL.
/// Otherwise, it installs dependencies from the `typst.toml` manifest in the current directory.
#[instrument(skip(cmd))]
pub async fn init(cmd: &InstallArgs, i: usize) -> Result<bool> {
    utpm_log!(trace, "executing init command for install");
    let _ = Box::pin(async move {
        // Determine the source path for the installation.
        let path = if let Some(url) = &cmd.url {
            // If a URL is provided, create a temporary directory for cloning.
            let dir = format!("{}/tmp/{}", datalocalutpm()?, i);
            utpm_log!(debug, "url is set to {}, creating {}", url, dir);
            dir
        } else {
            // Otherwise, use the current directory.
            let dir = get_current_dir()?;
            utpm_log!(debug, "url is none, current dir: {}", dir);
            dir
        };

        // If a URL is provided, clone or copy the repository.
        if let Some(x) = &cmd.url {
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
            // Handle git and http(s) URLs.
            if x.starts_with("git") || x.starts_with("http") {
                let mut callbacks = RemoteCallbacks::new();
                callbacks.credentials(|_, username_from_url, _| {
                    let binding = env::var("UTPM_USERNAME")
                        .unwrap_or(username_from_url.unwrap_or("git").to_string());
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
                builder.clone(&x, Path::new(&path))?;
            } else {
                // Handle local paths.
                copy_dir_all(&x, &path)?;
            }
        };

        // Check for a manifest file in the source directory.
        let typstfile = path.clone() + "/" + MANIFEST_FILE;
        if !check_path_file(&typstfile) {
            let origin = cmd.url.clone().unwrap_or("/".into());
            utpm_log!("{}", format!("x {}", origin));
            return Ok::<bool, UtpmError>(false);
        }

        // Load the manifest and extract UTPM-specific configurations.
        let file = load_manifest!(&path);
        let utpm = if let Some(value) = file.tool {
            value.get_section("utpm")?.unwrap_or(Extra::default())
        } else {
            Extra::default()
        };
        let namespace = utpm.namespace.unwrap_or("local".into());

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

        // Recursively install dependencies.
        utpm_log!("{}", format!("Installing {}...", file.package.name));
        if let Some(vec_depend) = utpm.dependencies {
            let mut y = 0;

            for a in &vec_depend {
                y += 1;
                let ins = InstallArgs {
                    force: cmd.force,
                    url: Some(a.to_string()),
                };
                init(&ins, i * vec_depend.len() + y).await?;
            }
        }

        // Link the installed package and clean up temporary files.
        if !cmd.url.is_none() {
            let lnk = LinkArgs {
                force: cmd.force,
                no_copy: false,
            };
            link::run(&lnk, Some(path.clone()), false).await?; //TODO: change here too
            fs::remove_dir_all(&path)?;
            utpm_log!(
                info,
                "+ {}:{}", file.package.name, file.package.version
            );
        } else {
            utpm_log!(
                info,
                "* Installation complete! If you want to use it as a lib, just do a `utpm link`!"
            )
        }
        Ok(true)
    }).await;
    Ok(true)
}
