//! Submits a package to Typst Universe

use std::env;
use std::fs::{copy, create_dir_all, read_to_string, remove_dir_all};
use std::io::IsTerminal;
use std::path::{Component, Path, PathBuf};
use std::str::FromStr;

use ignore::WalkBuilder;
use ignore::overrides::OverrideBuilder;
use octocrab::Octocrab;
use octocrab::params::State;
use tracing::instrument;
use typst_syntax::package::{PackageManifest, PackageSpec};
use url::Url;

use super::PublishArgs;
use crate::utils::dryrun::get_dry_run;
use crate::utils::git::{exist_git, git, git_output};
use crate::utils::paths::{MANIFEST_FILE, check_path_file, get_current_dir, local_package_path};
use crate::utils::state::{Result, UtpmError};
use crate::utils::try_find;
use crate::{utpm_bail, utpm_log};

const UPSTREAM_OWNER: &str = "typst";
const UPSTREAM_REPO: &str = "packages";

#[instrument(skip(cmd))]
pub async fn run(cmd: &PublishArgs) -> Result<bool> {
    let project_dir = match &cmd.path {
        Some(path) => path.clone(),
        None => get_current_dir()?,
    };
    let manifest: PackageManifest = try_find(&project_dir)?;
    let spec = PackageSpec::from_str(&format!(
        "@preview/{}:{}",
        manifest.package.name, manifest.package.version
    ))
    .map_err(|_| UtpmError::PackageFormatError)?;

    let files = package_files(&project_dir, &manifest, cmd)?;
    check_package(&project_dir, &files, &manifest.package.entrypoint)?;

    if get_dry_run() {
        utpm_log!(info, "Would submit {} with these files:", spec);
        for file in &files {
            utpm_log!(info, "  {}", file.display());
        }
        return Ok(true);
    }

    exist_git()?;
    let token = env::var("UTPM_GITHUB_TOKEN").ok();
    if cmd.create_pr && token.is_none() {
        utpm_bail!(MissingGithubToken);
    }

    let checkout = local_package_path()?;
    let fork = resolve_fork(cmd.fork.as_deref(), &checkout, token.as_deref()).await?;
    let owner = github_owner(&fork);
    if cmd.create_pr && owner.is_none() {
        utpm_bail!(ForkNotOnGithub, fork);
    }

    let title = format!("{}:{}", spec.name, spec.version);
    let branch = format!("{}-{}", spec.name, spec.version);
    let package_dir = format!("packages/preview/{}/{}", spec.name, spec.version);

    prepare_branch(&checkout, &fork, &package_dir, &branch)?;

    let target = checkout.join(&package_dir);
    if target.exists() {
        remove_dir_all(&target)?;
    }
    for file in &files {
        let dest = target.join(file);
        create_dir_all(dest.parent().unwrap_or(&target))?;
        copy(project_dir.join(file), dest)?;
    }

    git(&checkout, &["add", &package_dir])?;
    let identity = commit_identity(&project_dir);
    let mut commit: Vec<&str> = identity.iter().map(String::as_str).collect();
    commit.extend(["commit", "-m", cmd.message.as_deref().unwrap_or(&title)]);
    git(&checkout, &commit)?;
    // Each run rebuilds the branch, so a later run replaces the earlier commit.
    git(
        &checkout,
        &[
            "push",
            "--force-with-lease",
            "--set-upstream",
            "origin",
            &branch,
        ],
    )?;

    let Some(owner) = owner else {
        utpm_log!(
            info,
            "Pushed branch {} to {}. Open a pull request from it against typst/packages.",
            branch,
            fork
        );
        return Ok(true);
    };
    let head = format!("{owner}:{branch}");

    if let Some(token) = token.filter(|_| cmd.create_pr) {
        let body =
            read_to_string(checkout.join(".github/pull_request_template.md")).unwrap_or_default();
        let url = create_pull_request(&token, &head, &title, &body).await?;
        utpm_log!(info, "Pull request: {}", url);
    } else {
        let url = Url::parse_with_params(
            &format!("https://github.com/{UPSTREAM_OWNER}/{UPSTREAM_REPO}/compare/main...{head}"),
            &[("expand", "1"), ("title", &title)],
        )
        .map_err(anyhow::Error::from)?;
        utpm_log!(
            info,
            "Pushed. Complete the checklist and create the pull request at:\n{}",
            url
        );
        if std::io::stdout().is_terminal() {
            let _ = open::that_detached(url.as_str());
        }
    }

    Ok(true)
}

/// Lists the files to submit, relative to `project_dir`.
fn package_files(
    project_dir: &Path,
    manifest: &PackageManifest,
    cmd: &PublishArgs,
) -> Result<Vec<PathBuf>> {
    let mut overrides = OverrideBuilder::new(project_dir);
    for exclude in manifest.package.exclude.iter() {
        overrides.add(&format!("!{exclude}"))?;
    }

    let mut walk = WalkBuilder::new(project_dir);
    walk.overrides(overrides.build()?)
        .ignore(cmd.ignore)
        .git_ignore(cmd.git_ignore)
        .git_global(cmd.git_global_ignore)
        .git_exclude(cmd.git_exclude);
    if cmd.typst_ignore {
        walk.add_custom_ignore_filename(".typstignore");
    }
    if let Some(custom_ignore) = &cmd.custom_ignore
        && check_path_file(custom_ignore)
        && let Some(name) = custom_ignore.file_name()
    {
        walk.add_custom_ignore_filename(name);
    }

    let mut files = vec![];
    for entry in walk.build() {
        let entry = entry?;
        if entry.file_type().is_some_and(|t| t.is_file()) {
            let relative = entry
                .path()
                .strip_prefix(project_dir)
                .map_err(anyhow::Error::from)?;
            files.push(relative.to_path_buf());
        }
    }
    files.sort();
    Ok(files)
}

fn check_package(project_dir: &Path, files: &[PathBuf], entrypoint: &str) -> Result<()> {
    let dir = project_dir.display().to_string();
    if files.is_empty() {
        utpm_bail!(NoFiles);
    }
    if !files.iter().any(|file| file == Path::new(MANIFEST_FILE)) {
        utpm_bail!(OmitedTypstFile, dir);
    }
    let entrypoint_path: PathBuf = Path::new(entrypoint)
        .components()
        .filter(|c| *c != Component::CurDir)
        .collect();
    if !files.contains(&entrypoint_path) {
        utpm_bail!(OmitedEntryfile, entrypoint.to_string(), dir);
    }
    Ok(())
}

/// Picks the fork from, in order: `--fork`, the existing checkout, the GitHub API.
async fn resolve_fork(fork: Option<&str>, checkout: &Path, token: Option<&str>) -> Result<String> {
    if let Some(fork) = fork {
        return Ok(fork.to_string());
    }
    if checkout.join(".git").exists() {
        return git_output(checkout, &["remote", "get-url", "origin"]);
    }
    let Some(token) = token else {
        utpm_bail!(NoFork);
    };
    let crab = Octocrab::builder()
        .personal_token(token.to_string())
        .build()?;
    let repo = crab
        .repos(UPSTREAM_OWNER, UPSTREAM_REPO)
        .create_fork()
        .send()
        .await?;
    repo.ssh_url.ok_or(UtpmError::NoFork)
}

/// Checks out a fresh `branch` from the fork's `main` in `checkout`, limited to
/// `package_dir` and the PR template.
fn prepare_branch(checkout: &Path, fork: &str, package_dir: &str, branch: &str) -> Result<()> {
    if checkout.join(".git").exists() {
        git(checkout, &["remote", "set-url", "origin", fork])?;
    } else {
        create_dir_all(checkout)?;
        git(
            checkout,
            &["clone", "--filter=tree:0", "--no-checkout", fork, "."],
        )?;
    }
    git(checkout, &["fetch", "origin"])?;
    git(
        checkout,
        &["sparse-checkout", "set", package_dir, ".github"],
    )?;
    git(
        checkout,
        &["checkout", "--force", "-B", branch, "origin/main"],
    )
}

/// `-c` options that commit as the git identity configured for `project_dir`.
///
/// The checkout lives in UTPM's data directory, where directory-based git
/// config (`includeIf "gitdir:..."`) of the user may not apply.
fn commit_identity(project_dir: &Path) -> Vec<String> {
    ["user.name", "user.email"]
        .into_iter()
        .filter_map(|key| {
            let value = git_output(project_dir, &["config", key]).ok()?;
            Some(["-c".to_string(), format!("{key}={value}")])
        })
        .flatten()
        .collect()
}

/// Opens the pull request, or returns the open one for `head`.
async fn create_pull_request(token: &str, head: &str, title: &str, body: &str) -> Result<String> {
    let crab = Octocrab::builder()
        .personal_token(token.to_string())
        .build()?;
    let pulls = crab.pulls(UPSTREAM_OWNER, UPSTREAM_REPO);
    let existing = pulls.list().head(head).state(State::Open).send().await?;
    let pr = match existing.items.into_iter().next() {
        Some(pr) => pr,
        None => pulls.create(title, head, "main").body(body).send().await?,
    };
    Ok(pr.html_url.map(String::from).unwrap_or_default())
}

/// The GitHub account owning `fork`, if it is a github.com URL.
fn github_owner(fork: &str) -> Option<&str> {
    let path = fork
        .strip_prefix("git@github.com:")
        .or_else(|| fork.strip_prefix("ssh://git@github.com/"))
        .or_else(|| fork.strip_prefix("https://github.com/"))?;
    path.split('/').next().filter(|owner| !owner.is_empty())
}

#[cfg(test)]
mod tests {
    use super::github_owner;

    #[test]
    fn github_owner_handles_git_url_notations() {
        for url in [
            "git@github.com:you/packages.git",
            "ssh://git@github.com/you/packages.git",
            "https://github.com/you/packages",
        ] {
            assert_eq!(github_owner(url), Some("you"), "{url}");
        }
        assert_eq!(github_owner("/tmp/packages.git"), None);
        assert_eq!(github_owner("https://gitlab.com/you/packages"), None);
    }
}
