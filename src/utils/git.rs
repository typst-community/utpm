//! Git integration helpers (detect, clone/pull/push, add/commit) built on
//! shelling out to the `git` CLI. A global state holds the project directory.

use crate::{
    utils::{paths::get_current_dir, state::Result},
    utpm_bail,
};
use std::{
    io,
    path::PathBuf,
    process::Command,
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

/// Run `git push origin main` in the project directory.
///
/// Returns Ok(true) if the process launches; bubbles categorized errors otherwise.
pub fn push_git() -> Result<bool> {
    match Command::new("git")
        .current_dir(&project().lock().unwrap().0)
        .arg("push")
        .arg("origin")
        .arg("main")
        .status()
    {
        Ok(_) => Ok(true),
        Err(e) => {
            utpm_bail!(Git, e.to_string())
        },
    }
}

/// Run `git pull origin main` in the project directory.
///
/// Returns Ok(true) if the process launches; bubbles errors otherwise.
pub fn pull_git() -> Result<bool> {
    match Command::new("git")
        .current_dir(&project().lock().unwrap().0)
        .arg("pull")
        .arg("origin")
        .arg("main")
        .status()
    {
        Ok(_) => Ok(true),
        Err(e) => {
            utpm_bail!(Git, e.to_string())
        },
    }
}

/// Run `git clone <url> <path>` in the project directory.
///
/// Arguments:
/// - `string`: repository URL.
/// - `path`: destination path (relative to project dir or absolute).
///
/// Returns Ok(true) if the process launches; categorized errors otherwise.
pub fn clone_git(string: &str, path: &str) -> Result<bool> {
    match Command::new("git")
        .current_dir(&project().lock().unwrap().0)
        .arg("clone")
        .arg(string)
        .arg(path)
        .status()
    {
        Ok(_) => Ok(true),
        Err(e) => {
            utpm_bail!(Git, e.to_string())
        },
    }
}

/// Run `git add <path>` in the project directory.
///
/// Returns Ok(true) if the process launches; categorized errors otherwise.
pub fn add_git(path: &str) -> Result<bool> {
    match Command::new("git")
        .current_dir(&project().lock().unwrap().0)
        .arg("add")
        .arg(path)
        .status()
    {
        Ok(_) => Ok(true),
        Err(e) => {
            utpm_bail!(Git, e.to_string())
        },
    }
}

/// Run `git commit -m <msg>` in the project directory.
///
/// Returns Ok(true) if the process launches; categorized errors otherwise.
pub fn commit_git(msg: &str) -> Result<bool> {
    match Command::new("git")
        .current_dir(&project().lock().unwrap().0)
        .arg("commit")
        .arg("-m")
        .arg(msg)
        .status()
    {
        Ok(_) => Ok(true),
        Err(e) => {
            utpm_bail!(Git, e.to_string())
        },
    }
}
