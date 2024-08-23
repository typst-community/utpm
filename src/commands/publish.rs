use crate::utils::regex_package;
use crate::utils::specs::Extra;
use crate::utils::state::{Error, ErrorKind};
use std::env;
use std::ffi::OsStr;
use std::fmt::Debug;
use std::fs::{copy, create_dir_all};
use std::path::{Path, PathBuf};
use std::result::Result as R;

use crate::load_manifest;
use crate::utils::paths::{
    check_path_file, default_typst_packages, get_ssh_dir, has_content, TYPST_PACKAGE_URL,
};
use crate::utils::{paths::get_current_dir, state::Result};
use git2::build::RepoBuilder;
use git2::{Cred, FetchOptions, RemoteCallbacks, Repository};
use ignore::overrides::OverrideBuilder;
use tracing::{error, info, instrument};
use typst_project::manifest::Manifest;

use super::PublishArgs;

use ignore::WalkBuilder;

#[instrument(skip(cmd))]
pub fn run(cmd: &PublishArgs) -> Result<bool> {
    //todo: github create fork if not exist (checkout and everything), link to local packages, create PR, git push
    //todo: Check dependencies, a way to add them?
    //todo: check if there are files in the package...

    let config: Manifest = load_manifest!();

    info!("Manifest load");

    let path_curr: &PathBuf = if let Some(path) = &cmd.path {
        path
    } else {
        &get_current_dir()?.into()
    };

    info!("Path: {}", path_curr.to_str().unwrap());

    let version = config.package.version.to_string();
    let name: String = config.package.name.into();
    let re = regex_package();

    let package_format = format!("@preview/{name}:{version}");

    info!("Package: {package_format}");

    if !re.is_match(package_format.as_str()) {
        error!("Package didn't match, the name or the version is incorrect.");
        return Err(Error::empty(ErrorKind::UnknowError("todo".into()))); // todo: u k
    }

    let path_curr_str: &str = path_curr.to_str().unwrap();

    let path_packages: String = default_typst_packages()?;
    let path_packages_new: String = format!("{path_packages}/packages/preview/{name}/{version}");

    update_git_packages(path_packages)?;

    info!("Path to the new package {}", path_packages_new);

    let mut wb: WalkBuilder = WalkBuilder::new(path_curr);

    let mut overr: OverrideBuilder = OverrideBuilder::new(path_curr);

    for exclude in Extra::from(config.tool).exclude.unwrap_or(vec![]) {
        overr.add(("!".to_string() + &exclude).as_str())?;
    }

    wb.overrides(overr.build()?);

    wb.ignore(cmd.ignore)
        .git_ignore(cmd.git_ignore)
        .git_global(cmd.git_global_ignore)
        .git_exclude(cmd.git_exclude);

    info!(
        git_ignore = cmd.git_ignore,
        git_global_ignore = cmd.git_global_ignore,
        git_exclude = cmd.git_exclude
    );

    let mut path_check = path_curr.clone().into_os_string();
    path_check.push("/.typstignore");
    if check_path_file(path_check) {
        info!("Added .typstignore");
        wb.add_custom_ignore_filename(".typstignore");
    }

    if let Some(custom_ignore) = &cmd.custom_ignore {
        let filename = custom_ignore.file_name().unwrap().to_str().unwrap();
        info!(custom_ignore = filename, "Trying a new ignore file");
        if check_path_file(custom_ignore) {
            info!(custom_ignore = filename, "File exist, adding it");
            wb.add_custom_ignore_filename(filename);
        }
    }

    for result in wb.build().collect::<R<Vec<_>, _>>()? {
        if let Some(file_type) = result.file_type() {
            let path: &Path = result.path();
            let name: String = path.to_str().unwrap().to_string();
            let l: String = name.replace::<&str>(path_curr_str, &path_packages_new);
            println!("{l}");
            if file_type.is_dir() {
                create_dir_all(l)?;
            } else {
                copy(path, l)?;
            }
        }
    }

    Ok(true)
}

#[instrument]
fn update_git_packages<P>(path_packages: P) -> Result<Repository>
where
    P: AsRef<Path> + AsRef<OsStr> + Debug,
{
    create_dir_all(&path_packages)?;
    let repo: Repository;
    if has_content(&path_packages)? {
        info!("Content found, starting a 'git pull origin main'");
        repo = Repository::open(path_packages)?;
        repo.find_remote("origin")?.fetch(&["main"], None, None)?;
        let fetch_head = repo.find_reference("FETCH_HEAD")?;
        let fetch_commit = repo.reference_to_annotated_commit(&fetch_head)?;
        let analysis = repo.merge_analysis(&[&fetch_commit])?;
        if analysis.0.is_up_to_date() {
            info!("up to date, nothing to do");
        } else if analysis.0.is_fast_forward() {
            let refname = format!("refs/heads/{}", "main");
            let mut reference = repo.find_reference(&refname)?;
            reference.set_target(fetch_commit.id(), "Fast-Forward")?;
            repo.set_head(&refname)?;
            repo.checkout_head(Some(git2::build::CheckoutBuilder::default().force()))?;
            info!("fast forward done");
        } else {
            error!("Can't rebase for now.");
            return Err(Error::empty(ErrorKind::UnknowError("todo".into())));
        }
    } else {
        info!("Start cloning");
        let sshpath = get_ssh_dir()?;
        let ed: String = sshpath.clone() + "/id_ed25519";
        let rsa: String = sshpath + "/id_rsa";
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

        info!(path = val);
        let mut callbacks = RemoteCallbacks::new();
        callbacks.credentials(|_, username_from_url, _| {
            let binding: String =
                env::var("UTPM_USERNAME").unwrap_or(username_from_url.unwrap_or("git").to_string());
            let username: &str = binding.as_str();
            match Cred::ssh_key_from_agent(username) {
                Ok(cred) => Ok(cred),
                Err(_) => Ok(match env::var("UTPM_PASSPHRASE") {
                    Ok(s) => {
                        info!(passphrase = true);
                        Cred::ssh_key(username, None, Path::new(&val), Some(s.as_str()))?
                    }
                    Err(_) => {
                        info!(passphrase = false);
                        Cred::ssh_key(username, None, Path::new(&val), None)?
                    }
                }),
            }
        });

        let mut fo = FetchOptions::new();
        fo.remote_callbacks(callbacks);

        let mut builder = RepoBuilder::new();
        builder.fetch_options(fo);
        repo = builder.clone(TYPST_PACKAGE_URL, Path::new(&path_packages))?;
        info!("Package cloned");
    };
    Ok(repo)
}
