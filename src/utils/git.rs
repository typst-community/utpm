use crate::{
    utils::{paths::get_current_dir, state::Result},
    utpm_bail,
};
use std::{
    io,
    process::Command,
    sync::{Mutex, OnceLock},
};

pub struct State(pub String);

impl State {
    pub fn new() -> Self {
        Self {
            0: get_current_dir().unwrap(),
        }
    }
}

pub fn workspace() -> &'static Mutex<State> {
    static STRING: OnceLock<Mutex<State>> = OnceLock::new();
    STRING.get_or_init(|| (Mutex::new(State::new())))
}

pub fn exist_git() -> Result<bool> {
    match Command::new("git").arg("--version").status() {
        Ok(_) => Ok(true),
        Err(e) if e.kind() == io::ErrorKind::NotFound => {
            utpm_bail!(GitNotFound)
        }
        Err(e) => {
            utpm_bail!(Git, e.to_string())
        }
    }
}

pub fn push_git() -> Result<bool> {
    match Command::new("git")
        .current_dir(workspace().lock().unwrap().0.as_str())
        .arg("push")
        .arg("origin")
        .arg("main")
        .status()
    {
        Ok(_) => Ok(true),
        Err(e) if e.kind() == io::ErrorKind::NotFound => {
            utpm_bail!(GitNotFound)
        }
        Err(e) => {
            utpm_bail!(Git, e.to_string())
        }
    }
}

pub fn pull_git() -> Result<bool> {
    match Command::new("git")
        .current_dir(workspace().lock().unwrap().0.as_str())
        .arg("pull")
        .arg("origin")
        .arg("main")
        .status()
    {
        Ok(_) => Ok(true),
        Err(e) => {
            utpm_bail!(Git, e.to_string())
        }
    }
}

pub fn clone_git(string: &str, path: &str) -> Result<bool> {
    match Command::new("git")
        .current_dir(workspace().lock().unwrap().0.as_str())
        .arg("clone")
        .arg(string)
        .arg(path)
        .status()
    {
        Ok(_) => Ok(true),
        Err(e) if e.kind() == io::ErrorKind::NotFound => {
            utpm_bail!(GitNotFound)
        }
        Err(e) => {
            utpm_bail!(Git, e.to_string())
        }
    }
}

pub fn add_git(path: &str) -> Result<bool> {
    match Command::new("git")
        .current_dir(workspace().lock().unwrap().0.as_str())
        .arg("add")
        .arg(path)
        .status()
    {
        Ok(_) => Ok(true),
        Err(e) if e.kind() == io::ErrorKind::NotFound => {
            utpm_bail!(GitNotFound)
        }
        Err(e) => {
            utpm_bail!(Git, e.to_string())
        }
    }
}

pub fn commit_git(msg: &str) -> Result<bool> {
    match Command::new("git")
        .current_dir(workspace().lock().unwrap().0.as_str())
        .arg("commit")
        .arg("-m")
        .arg(msg)
        .status()
    {
        Ok(_) => Ok(true),
        Err(e) if e.kind() == io::ErrorKind::NotFound => {
            utpm_bail!(GitNotFound)
        }
        Err(e) => {
            utpm_bail!(Git, e.to_string())
        }
    }
}
