//! Submits a package to Typst Universe

use std::env;
use std::fs::{copy, create_dir_all, read_to_string, remove_dir_all};
use std::io::IsTerminal;
use std::path::{Component, Path, PathBuf};
use std::str::FromStr;
use std::time::{Duration, Instant};

use base64::Engine;
use base64::prelude::BASE64_STANDARD;
use ignore::WalkBuilder;
use ignore::overrides::OverrideBuilder;
use octocrab::Octocrab;
use octocrab::params::State;
use tracing::instrument;
use typst_syntax::package::{PackageManifest, PackageSpec};
use url::Url;

use super::PublishArgs;
use crate::utils::dryrun::get_dry_run;
use crate::utils::git::{exist_git, git, git_env, git_output, git_succeeds};
use crate::utils::paths::{MANIFEST_FILE, check_path_file, get_current_dir, local_package_path};
use crate::utils::state::{Result, UtpmError};
use crate::utils::try_find;
use crate::{utpm_bail, utpm_log};

const UPSTREAM_OWNER: &str = "typst";
const UPSTREAM_REPO: &str = "packages";
const UPSTREAM_URL_ENV: &str = "UTPM_PACKAGES_UPSTREAM";
const UPSTREAM_URL: &str = "https://github.com/typst/packages.git";
const TOKEN_ENV: &str = "UTPM_GITHUB_TOKEN";
const FORK_READY_TIMEOUT: Duration = Duration::from_secs(60);

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
    let identity = commit_identity(&project_dir)?;
    let token = env::var(TOKEN_ENV).ok();
    let auth = github_auth(token.as_deref());
    if cmd.create_pr && token.is_none() {
        utpm_bail!(MissingGithubToken);
    }

    let checkout = local_package_path()?;
    let fork = resolve_fork(cmd.fork.as_deref(), &checkout, token.as_deref()).await?;
    let owner = github_repo(&fork).map(|(owner, _)| owner);
    if cmd.create_pr && owner.is_none() {
        utpm_bail!(ForkNotOnGithub, fork);
    }

    let title = format!("{}:{}", spec.name, spec.version);
    let branch = format!("{}-{}", spec.name, spec.version);
    let package_dir = format!("packages/preview/{}/{}", spec.name, spec.version);

    prepare_branch(
        &checkout,
        &fork,
        &package_dir,
        &branch,
        token.as_deref(),
        &auth,
    )
    .await?;

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
    let mut commit: Vec<&str> = identity.iter().map(String::as_str).collect();
    commit.extend(["commit", "-m", cmd.message.as_deref().unwrap_or(&title)]);
    git(&checkout, &commit)?;
    // Each run rebuilds the branch, so a later run replaces the earlier commit.
    git_env(
        &checkout,
        &[
            "push",
            "--force-with-lease",
            "--set-upstream",
            "origin",
            &branch,
        ],
        &auth,
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
    let crab = github(token)?;
    let repo = crab
        .repos(UPSTREAM_OWNER, UPSTREAM_REPO)
        .create_fork()
        .send()
        .await?;
    let url = repo.clone_url.ok_or(UtpmError::NoFork)?.to_string();
    wait_for_fork(&url, token).await?;
    Ok(url)
}

/// Waits until `url` serves a `main` branch.
async fn wait_for_fork(url: &str, token: &str) -> Result<()> {
    let args = ["ls-remote", "--exit-code", url, "refs/heads/main"];
    let auth = github_auth(Some(token));
    let dir = env::temp_dir();
    let deadline = Instant::now() + FORK_READY_TIMEOUT;
    let mut delay = Duration::from_secs(1);
    let mut waiting = false;
    while Instant::now() < deadline {
        if git_succeeds(&dir, &args, &auth)? {
            return Ok(());
        }
        if !waiting {
            utpm_log!(info, "Waiting for GitHub to create your fork...");
            waiting = true;
        }
        tokio::time::sleep(delay).await;
        delay = (delay * 2).min(Duration::from_secs(8));
    }
    // One last try that shows git error.
    git_env(&dir, &args, &auth)
}

/// Environment variables that make git log in to github.com over https with
/// `token`, if there is one.
fn github_auth(token: Option<&str>) -> Vec<(String, String)> {
    let Some(token) = token else {
        return vec![];
    };
    let credentials = BASE64_STANDARD.encode(format!("x-access-token:{token}"));
    let config = [
        // Never ask other credential helpers, which might prompt.
        ("credential.https://github.com.helper", String::new()),
        (
            "http.https://github.com/.extraheader",
            format!("AUTHORIZATION: basic {credentials}"),
        ),
    ];
    let start: usize = env::var("GIT_CONFIG_COUNT")
        .ok()
        .and_then(|count| count.parse().ok())
        .unwrap_or(0);
    let mut env = vec![(
        "GIT_CONFIG_COUNT".to_string(),
        (start + config.len()).to_string(),
    )];
    for (i, (key, value)) in config.into_iter().enumerate() {
        env.push((format!("GIT_CONFIG_KEY_{}", start + i), key.to_string()));
        env.push((format!("GIT_CONFIG_VALUE_{}", start + i), value));
    }
    env
}

/// Checks out a fresh `branch` from the fork's `main` in `checkout`, limited to
/// `package_dir` and the PR template.
async fn prepare_branch(
    checkout: &Path,
    fork: &str,
    package_dir: &str,
    branch: &str,
    token: Option<&str>,
    auth: &[(String, String)],
) -> Result<()> {
    if checkout.join(".git").exists() {
        git(checkout, &["remote", "set-url", "origin", fork])?;
    } else {
        create_dir_all(checkout)?;
        git_env(
            checkout,
            &["clone", "--filter=tree:0", "--no-checkout", fork, "."],
            auth,
        )?;
    }
    git_env(checkout, &["fetch", "--prune", "origin"], auth)?;
    update_fork(checkout, fork, token, auth).await?;
    git(
        checkout,
        &[
            "sparse-checkout",
            "set",
            "--sparse-index",
            package_dir,
            ".github",
        ],
    )?;
    git(
        checkout,
        &["checkout", "--force", "-B", branch, "origin/main"],
    )
}

/// Ensures the fork's `main` only has commits from typst/packages, and brings it
/// up to date through the GitHub API when a token allows it.
async fn update_fork(
    checkout: &Path,
    fork: &str,
    token: Option<&str>,
    auth: &[(String, String)],
) -> Result<()> {
    let upstream = env::var(UPSTREAM_URL_ENV).unwrap_or_else(|_| UPSTREAM_URL.to_string());
    git(
        checkout,
        &["fetch", "--filter=tree:0", "--no-tags", &upstream, "main"],
    )?;
    let count = |range: &str| -> Result<usize> {
        let count = git_output(checkout, &["rev-list", "--count", range])?;
        Ok(count.parse().map_err(anyhow::Error::from)?)
    };

    let own = count("FETCH_HEAD..origin/main")?;
    if own > 0 {
        utpm_bail!(ForkDiverged, own);
    }
    let behind = count("origin/main..FETCH_HEAD")?;
    if behind == 0 {
        return Ok(());
    }

    match (token, github_repo(fork)) {
        (Some(token), Some((owner, repo))) => {
            #[derive(serde::Serialize)]
            struct MergeUpstream {
                branch: &'static str,
            }
            let crab = github(token)?;
            let _: serde::de::IgnoredAny = crab
                .post(
                    format!("/repos/{owner}/{repo}/merge-upstream"),
                    Some(&MergeUpstream { branch: "main" }),
                )
                .await?;
            utpm_log!(info, "Synced your fork with typst/packages.");
            git_env(checkout, &["fetch", "--prune", "origin"], auth)
        },
        _ => {
            utpm_log!(
                warn,
                "Your fork is {} commits behind typst/packages, so the new branch is based on old files. Sync the fork on GitHub, or set UTPM_GITHUB_TOKEN to let UTPM do it.",
                behind
            );
            Ok(())
        },
    }
}

/// `-c` options that commit as the git identity of `project_dir`.
///
/// Without a configured identity, e.g. in CI, this
/// falls back to the author of the last
/// commit of the project.
fn commit_identity(project_dir: &Path) -> Result<Vec<String>> {
    let options = |name: &str, email: &str| {
        vec![
            "-c".to_string(),
            format!("user.name={name}"),
            "-c".to_string(),
            format!("user.email={email}"),
        ]
    };
    let config = |key| git_output(project_dir, &["config", key]).ok();
    let mut identity = match (config("user.name"), config("user.email")) {
        (Some(name), Some(email)) => options(&name, &email),
        _ => vec![],
    };
    let known = |identity: &[String]| -> Result<bool> {
        for var in ["GIT_AUTHOR_IDENT", "GIT_COMMITTER_IDENT"] {
            let mut args: Vec<&str> = identity.iter().map(String::as_str).collect();
            args.extend(["var", var]);
            if !git_succeeds(project_dir, &args, &[])? {
                return Ok(false);
            }
        }
        Ok(true)
    };
    if known(&identity)? {
        return Ok(identity);
    }
    if let Ok(last) = git_output(project_dir, &["log", "-1", "--format=%an%n%ae"])
        && let Some((name, email)) = last.split_once('\n')
    {
        identity = options(name, email);
        if known(&identity)? {
            utpm_log!(
                info,
                "No git identity configured, committing as {} <{}>, the author of the last commit.",
                name,
                email
            );
            return Ok(identity);
        }
    }
    utpm_bail!(MissingGitIdentity);
}

/// Opens the pull request, or returns the open one for `head`.
async fn create_pull_request(token: &str, head: &str, title: &str, body: &str) -> Result<String> {
    let crab = github(token)?;
    let pulls = crab.pulls(UPSTREAM_OWNER, UPSTREAM_REPO);
    let existing = pulls.list().head(head).state(State::Open).send().await?;
    let pr = match existing.items.into_iter().next() {
        Some(pr) => pr,
        None => pulls.create(title, head, "main").body(body).send().await?,
    };
    Ok(pr.html_url.map(String::from).unwrap_or_default())
}

/// A GitHub API client that authenticates with `token`.
fn github(token: &str) -> Result<Octocrab> {
    Ok(Octocrab::builder()
        .personal_token(token.to_string())
        .build()?)
}

/// Owner and name of `fork`, if it is a github.com URL.
fn github_repo(fork: &str) -> Option<(&str, &str)> {
    let path = fork
        .strip_prefix("git@github.com:")
        .or_else(|| fork.strip_prefix("ssh://git@github.com/"))
        .or_else(|| fork.strip_prefix("https://github.com/"))?;
    let (owner, repo) = path.trim_end_matches('/').split_once('/')?;
    let repo = repo.strip_suffix(".git").unwrap_or(repo);
    (!owner.is_empty() && !repo.is_empty() && !repo.contains('/')).then_some((owner, repo))
}

#[cfg(test)]
mod tests {
    use super::{github_auth, github_repo};

    #[test]
    fn github_auth_sends_token_as_basic_auth_header() {
        assert!(github_auth(None).is_empty());
        let env = github_auth(Some("secret"));
        let value = |key: &str| {
            let (index, _) = env
                .iter()
                .find(|(_, value)| value == key)
                .and_then(|(var, _)| var.strip_prefix("GIT_CONFIG_KEY_").map(|i| (i, ())))
                .expect(key);
            env.iter()
                .find(|(var, _)| *var == format!("GIT_CONFIG_VALUE_{index}"))
                .map(|(_, value)| value.as_str())
                .unwrap()
        };
        // base64 of "x-access-token:secret"
        assert_eq!(
            value("http.https://github.com/.extraheader"),
            "AUTHORIZATION: basic eC1hY2Nlc3MtdG9rZW46c2VjcmV0"
        );
        assert_eq!(value("credential.https://github.com.helper"), "");
    }

    #[test]
    fn github_repo_handles_git_url_notations() {
        for url in [
            "git@github.com:you/packages.git",
            "ssh://git@github.com/you/packages.git",
            "https://github.com/you/packages",
        ] {
            assert_eq!(github_repo(url), Some(("you", "packages")), "{url}");
        }
        assert_eq!(github_repo("/tmp/packages.git"), None);
        assert_eq!(github_repo("https://gitlab.com/you/packages"), None);
    }
}
