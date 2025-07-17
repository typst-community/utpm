use crate::utils::specs::Extra;
use crate::utils::state::Result;
use crate::utils::{push_git_packages, regex_package, update_git_packages};
use crate::utpm_log;
use std::env;
use std::fs::{copy, create_dir_all};
use std::path::{Path, PathBuf};
use std::result::Result as R;
use std::str::FromStr;

use crate::utils::paths::get_current_dir;
use crate::utils::paths::{
    check_path_file, default_typst_packages, has_content, TYPST_PACKAGE_URL,
};
use crate::{load_manifest, utils::state::UtpmError, utpm_bail};
use ignore::overrides::OverrideBuilder;
use octocrab::models::{Author, UserProfile};
use octocrab::Octocrab;
use tracing::instrument;

use typst_project::manifest::Manifest;

use super::PublishArgs;

use ignore::WalkBuilder;

/// Publishes a package to the typst universe.
///
/// This involves:
/// - Forking the `typst/packages` repository if not already forked.
/// - Cloning or updating the forked repository.
/// - Copying the package files to the repository.
/// - Committing and pushing the changes.
/// - Creating a pull request to the `typst/packages` repository.
#[tokio::main]
#[instrument(skip(cmd))]
pub async fn run(cmd: &PublishArgs) -> Result<bool> {
    // TODO: Implement GitHub fork creation, linking to local packages, PR creation, and git push.
    // TODO: Check for dependencies and provide a way to add them.
    // TODO: Ensure there are files in the package before publishing.

    let config: Manifest = load_manifest!();
    utpm_log!(info, "Manifest load");

    let path_curr: &PathBuf = if let Some(path) = &cmd.path {
        path
    } else {
        &get_current_dir()?.into()
    };
    utpm_log!(info, "Path: {}", path_curr.to_str().unwrap());

    let version: String = config.package.version.to_string();
    let name: String = config.package.name.into();
    let re: regex::Regex = regex_package();

    let package_format = format!("@preview/{name}:{version}");
    utpm_log!(info, "Package: {package_format}");

    if !re.is_match(package_format.as_str()) {
        utpm_log!(
            error,
            "Package didn't match, the name or the version is incorrect."
        );
        utpm_bail!(Unknown, "todo".into()); // TODO: Improve error handling.
    }

    let path_curr_str: &str = path_curr.to_str().unwrap();
    let path_packages: String = default_typst_packages()?;
    let path_packages_new: String = format!("{path_packages}/packages/preview/{name}/{version}");

    // --- GitHub Handling ---
    let crab = Octocrab::builder()
        .personal_token(
            env::var("UTPM_GITHUB_TOKEN")
                .expect("Should have a github token in \"UTPM_GITHUB_TOKEN\""),
        )
        .build()
        .unwrap();

    // Check if a fork of typst/packages already exists.
    let pages = match crab
        .current()
        .list_repos_for_authenticated_user()
        .visibility("public")
        .send()
        .await
    {
        Ok(a) => a,
        Err(_) => todo!(), // TODO: Better error handling
    };

    let repo: Option<&octocrab::models::Repository> = pages.items.iter().find(|f| match &f.forks_url {
        None=>"",
        Some(a) => a.as_str()
    } == TYPST_PACKAGE_URL );

    let fork: String;
    let name_package = format!("{}-{}", name.clone(), config.package.version.to_string());

    if let Some(rep) = repo {
        fork = rep.url.clone().into();
    } else {
        // If no fork exists, create one.
        // Format into: "mypackage-1.0.0" as GitHub doesn't allow ':' in repo names.
        match crab
            .repos("typst", "packages")
            .create_fork()
            .name(&name_package)
            .send()
            .await
        {
            Ok(val) => fork = format!("git@github.com:{}.git", val.full_name.expect("Didn't fork")),
            Err(err) => {
                utpm_log!("{:?}", err);
                utpm_bail!(OctoCrab, err);
            }
        };
    }

    // --- File Preparation ---
    // Download or update the typst/packages repository.
    let repos = update_git_packages(path_packages, fork.as_str())?;
    utpm_log!(info, "Path to the new package {}", path_packages_new);

    // Use WalkBuilder to respect ignore files.
    let mut wb: WalkBuilder = WalkBuilder::new(path_curr);
    let mut overr: OverrideBuilder = OverrideBuilder::new(path_curr);

    // Add excludes from the manifest to the override builder.
    for exclude in Extra::from(config.tool).exclude.unwrap_or(vec![]) {
        overr.add(("!".to_string() + &exclude).as_str())?;
    }
    wb.overrides(overr.build()?);

    // Configure which ignore files to use.
    wb.ignore(cmd.ignore)
        .git_ignore(cmd.git_ignore)
        .git_global(cmd.git_global_ignore)
        .git_exclude(cmd.git_exclude);
    utpm_log!(info,
        "git_ignore" => cmd.git_ignore,
        "git_global_ignore" => cmd.git_global_ignore,
        "git_exclude" => cmd.git_exclude
    );

    // Add .typstignore if it exists.
    let mut path_check = path_curr.clone().into_os_string();
    path_check.push("/.typstignore");
    if check_path_file(path_check) {
        utpm_log!(info, "Added .typstignore");
        wb.add_custom_ignore_filename(".typstignore");
    }

    // Add custom ignore file if specified.
    if let Some(custom_ignore) = &cmd.custom_ignore {
        let filename = custom_ignore.file_name().unwrap().to_str().unwrap();
        utpm_log!(info, "Trying a new ignore file", "custom_ignore" => filename);
        if check_path_file(custom_ignore) {
            utpm_log!(info, "File exist, adding it", "custom_ignore" => filename);
            wb.add_custom_ignore_filename(filename);
        }
    }

    // --- Copy Files ---
    for result in wb.build().collect::<R<Vec<_>, _>>()? {
        if let Some(file_type) = result.file_type() {
            let path: &Path = result.path();
            let name: String = path.to_str().unwrap().to_string();
            let l: String = name.replace::<&str>(path_curr_str, &path_packages_new);
            utpm_log!("{}", l);
            if file_type.is_dir() {
                create_dir_all(l)?;
            } else {
                copy(path, l)?;
            }
        }
    }

    // --- Validation ---
    if !has_content(&path_packages_new)? {
        utpm_bail!(
            Unknown,
            "There is no files in the new package. Consider to change your ignored files.".into()
        );
    }
    if !check_path_file(format!("{path_packages_new}/typst.toml")) {
        utpm_bail!(Unknown, format!("Can't find `typst.toml` file in {path_packages_new}. Did you omit it in your ignored files?"));
    }
    let entry = config.package.entrypoint;
    let mut entryfile = PathBuf::from_str(&path_packages_new).unwrap();
    entryfile.push(&entry);
    let entrystr = entry.to_str().unwrap();
    utpm_log!(trace, "entryfile" => entrystr);
    if !check_path_file(entryfile) {
        utpm_bail!(Unknown, format!("Can't find {entrystr} file in {path_packages_new}. Did you omit it in your ignored files?"));
    }
    utpm_log!(info, "files copied to {path_packages_new}");

    // --- Git Push ---
    utpm_log!(info, "Getting information from github");
    let author_user: Author = crab.current().user().await?;
    let user: UserProfile = crab.users_by_id(author_user.id).profile().await?;
    let us = &user;
    utpm_log!(info,
        "email" => us.email,
        "id" => us.id.to_string(),
        "name" => us.name.clone().unwrap()
    );

    let name_replaced = name_package.replace('-', ":");
    let msg = cmd
        .message
        .clone()
        .unwrap_or(format!("{} using utpm", &name_replaced));

    push_git_packages(repos, user.clone(), msg.as_str())?;
    utpm_log!(info, "Ended push");

    // --- Pull Request ---
    crab.pulls("typst", "packages")
        .create(name_replaced.as_str(), format!("{}:main", us.name.clone().unwrap()), "base")
        .body("
I am submitting
- [ ] a new package
- [ ] an update for a package


Description: Explain what the package does and why it's useful.

I have read and followed the submission guidelines and, in particular, I
- [ ] selected a name that isn't the most obvious or canonical name for what the package does
- [ ] added a `typst.toml` file with all required keys
- [ ] added a `README.md` with documentation for my package
- [ ] have chosen a license and added a `LICENSE` file or linked one in my `README.md`
- [ ] tested my package locally on my system and it worked
- [ ] `exclude`d PDFs or README images, if any, but not the LICENSE

- [ ] ensured that my package is licensed such that users can use and distribute the contents of its template directory without restriction, after modifying them through normal use.
") // TODO: Improve PR body.
        .send()
        .await?;

    Ok(true)
}
