//! Git integration helpers built on shelling out to the `git` CLI. A global
//! state holds the project directory.

use crate::{
    utils::{paths::get_current_dir, state::Result},
    utpm_bail,
};
use std::{
    io,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::{Mutex, OnceLock},
};

/// Shared project state.
/// Currently holds only the current working directory (String).
pub struct State(pub PathBuf);

impl Default for State {
    fn default() -> Self {
        Self::new()
    }
}

impl State {
    /// Build a new state using the process current directory.
    /// Panics if `get_current_dir()` fails.
    pub fn new() -> Self {
        Self(get_current_dir().unwrap())
    }
}

/// Return a global singleton `Mutex<State>`.
/// Lazily initialized via `OnceLock` on first use.
pub fn project() -> &'static Mutex<State> {
    static STRING: OnceLock<Mutex<State>> = OnceLock::new();
    STRING.get_or_init(|| Mutex::new(State::new()))
}

/// Check if Git is available in PATH by running `git --version`.
///
/// Returns:
/// - Ok(true) if the command could be spawned (does not inspect exit status).
/// - Err(GitNotFound) if the `git` executable is missing.
/// - Err(Git(...)) for any other I/O error.
pub fn exist_git() -> Result<bool> {
    match Command::new("git").arg("--version").status() {
        Ok(_) => Ok(true),
        Err(e) if e.kind() == io::ErrorKind::NotFound => {
            utpm_bail!(GitNotFound)
        },
        Err(e) => {
            utpm_bail!(Git, e.to_string())
        },
    }
}

/// Runs `git <args>` in `dir`, failing unless git exits successfully.
pub fn git(dir: &Path, args: &[&str]) -> Result<()> {
    git_output(dir, args).map(drop)
}

/// Like [`git`], but returns git's trimmed standard output.
pub fn git_output(dir: &Path, args: &[&str]) -> Result<String> {
    let output = match Command::new("git")
        .current_dir(dir)
        .args(args)
        .stderr(Stdio::inherit())
        .output()
    {
        Ok(output) => output,
        Err(e) => utpm_bail!(Git, e.to_string()),
    };
    if !output.status.success() {
        utpm_bail!(
            Git,
            format!("'git {}' failed: {}", args.join(" "), output.status)
        );
    }
    Ok(String::from_utf8(output.stdout)?.trim().to_string())
}

/// Runs `git <args>` in `dir` without output or prompts, and reports whether
/// git exited successfully.
pub fn git_succeeds(dir: &Path, args: &[&str]) -> Result<bool> {
    match Command::new("git")
        .current_dir(dir)
        .args(args)
        .env("GIT_TERMINAL_PROMPT", "0")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
    {
        Ok(status) => Ok(status.success()),
        Err(e) => utpm_bail!(Git, e.to_string()),
    }
}

/// Runs `git clone [-b <branch>] <url> <path>` in the project directory.
///
/// Arguments:
/// - `repository`: repository URL.
/// - `path`: destination path (relative to project dir or absolute).
/// - `branch`: optional branch to clone from.
pub fn clone_git(repository: &str, path: &str, branch: Option<&str>) -> Result<()> {
    let dir = project().lock().unwrap().0.clone();
    let mut args = vec!["clone"];
    if let Some(branch) = branch {
        args.extend(["-b", branch, "--single-branch"]);
    }
    args.extend([repository, path]);
    git(&dir, &args)
}

/// Clones only `subdir` of a repository, using `git clone --filter` and
/// `git sparse-checkout`, in the project directory.
///
/// Arguments:
/// - `repository`: repository URL.
/// - `path`: destination path (relative to project dir or absolute).
/// - `subdir`: subdirectory to clone.
/// - `branch`: specific branch to checkout from.
pub fn sparse_checkout_git(
    repository: &str,
    path: &str,
    subdir: &str,
    branch: Option<&str>,
) -> Result<()> {
    // https://stackoverflow.com/questions/600079/how-do-i-clone-a-subdirectory-only-of-a-git-repository/52269934#52269934
    let dir = project().lock().unwrap().0.clone();
    let mut args = vec!["clone", "--no-checkout", "--depth=1", "--filter=tree:0"];
    if let Some(branch) = branch {
        args.extend(["-b", branch, "--single-branch"]);
    }
    args.extend([repository, path]);
    git(&dir, &args)?;

    let repo = dir.join(path);
    git(&repo, &["sparse-checkout", "set", "--no-cone", subdir])?;
    git(&repo, &["checkout"])
}
