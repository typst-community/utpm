use crate::utils::git::{add_git, clone_git, commit_git, exist_git, project, pull_git, push_git};
use crate::utils::specs::Extra;
use crate::utils::state::Result;
use crate::utils::{regex_package, try_find};
use crate::utpm_log;
use std::env;
use std::fs::{copy, create_dir_all};
use std::path::PathBuf;
use std::result::Result as R;

use crate::path;
use crate::utils::paths::{MANIFEST_FILE, get_current_dir};
use crate::utils::paths::{TYPST_PACKAGE_URL, check_path_file, has_content, local_package_path};
use crate::utpm_bail;
use ignore::overrides::OverrideBuilder;
use octocrab::Octocrab;
use octocrab::models::{Author, UserProfile};
use tracing::instrument;
use typst_syntax::package::PackageManifest;

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
#[instrument(skip(cmd))]
pub async fn run(cmd: &PublishArgs) -> Result<bool> {
    // TODO: Dry run
    utpm_log!(trace, "executing publish command");
    // TODO: Ensure there are files in the package before publishing.
    exist_git()?;
    let config: PackageManifest = try_find(get_current_dir()?)?;
    utpm_log!(info, "Manifest load");

    let path_curr: &PathBuf = if let Some(path) = &cmd.path {
        path
    } else {
        &get_current_dir()?
    };
    utpm_log!(info, "Path: {}", path_curr.display());

    let version: String = config.package.version.to_string();
    let name: String = config.package.name.into();
    let re: regex::Regex = regex_package();

    let package_format = format!("@preview/{name}:{version}");
    utpm_log!(info, "Package: {}", package_format);

    if !re.is_match(package_format.as_str()) {
        utpm_bail!(PackageFormatError); // TODO: Improve error handling.
    }

    let packages_path = local_package_path()?;
    let new_package_path = path!(&packages_path, "packages", "preview", &name, &version);

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
    let name_package = format!("{}-{}", name.clone(), config.package.version);

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
            },
        };
    }

    project().lock().unwrap().0 = packages_path.clone();

    // --- File Preparation ---
    // Download or update the typst/packages repository.

    match pull_git() {
        Ok(_) => Ok(true),
        Err(_) => clone_git(&packages_path.to_string_lossy(), fork.as_str()),
    }?;
    utpm_log!(
        info,
        "Path to the new package {}",
        new_package_path.display()
    );

    // Use WalkBuilder to respect ignore files.
    let mut wb: WalkBuilder = WalkBuilder::new(path_curr);
    let mut overr: OverrideBuilder = OverrideBuilder::new(path_curr);

    // Add excludes from the manifest to the override builder.
    if let Some(excludes) = Extra::from(config.tool).exclude {
        for exclude in excludes.iter() {
            overr.add(&format!("!{}", exclude))?;
        }
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

    // Add .typstignore if it exists and is enabled.
    if cmd.typst_ignore && check_path_file(path!(path_curr, ".typstignore")) {
        utpm_log!(info, "Added .typstignore");
        wb.add_custom_ignore_filename(".typstignore");
    }

    // Add custom ignore file if specified.
    if let Some(custom_ignore) = &cmd.custom_ignore
        && let Some(filename) = custom_ignore.file_name().and_then(|f| f.to_str())
    {
        utpm_log!(info, "Trying a new ignore file", "custom_ignore" => filename);
        if check_path_file(custom_ignore) {
            utpm_log!(info, "File exist, adding it", "custom_ignore" => filename);
            wb.add_custom_ignore_filename(filename);
        }
    }

    // --- Copy Files ---
    for result in wb.build().collect::<R<Vec<_>, _>>()? {
        if let Some(file_type) = result.file_type() {
            let path = result.path();
            let relative = path
                .strip_prefix(path_curr)
                .map_err(|e| anyhow::anyhow!("Failed to strip prefix: {}", e))?;
            let dest_path = path!(&new_package_path, relative);
            utpm_log!("{}", dest_path.display());
            if file_type.is_dir() {
                create_dir_all(&dest_path)?;
            } else {
                copy(path, &dest_path)?;
            }
        }
    }

    // --- Validation ---
    if !has_content(&new_package_path)? {
        utpm_bail!(NoFiles);
    }
    let manifest_check = path!(&new_package_path, MANIFEST_FILE);
    if !check_path_file(&manifest_check) {
        utpm_bail!(OmitedTypstFile, new_package_path.display().to_string());
    }
    let entry = config.package.entrypoint;
    let entryfile = path!(&new_package_path, entry.as_str());
    let entrystr = entry.to_string();
    utpm_log!(trace, "entryfile" => entrystr);
    if !check_path_file(&entryfile) {
        utpm_bail!(
            OmitedEntryfile,
            entrystr,
            new_package_path.display().to_string()
        );
    }
    utpm_log!(info, "files copied to {}", new_package_path.display());

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

    project().lock().unwrap().0 = new_package_path;

    add_git(".")?;
    commit_git(&msg)?;
    push_git()?;
    utpm_log!(info, "Ended push");

    // --- Pull Request ---
    crab.pulls("typst", "packages")
        .create(name_replaced.as_str(), format!("{}:main", us.name.clone().unwrap()), "base")
        .body(r#"I am submitting
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
"#) // TODO: Improve PR body.
        .send()
        .await?;

    Ok(true)
}
