//! A sandbox for running the `utpm` binary end-to-end.
//!
//! Every invocation is a separate process whose environment points all of UTPM's
//! directories into a fresh temporary directory, so tests can run in parallel and
//! never touch the real package storage of the machine running them.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use tempfile::TempDir;

/// Environment variables of the test runner that would leak into the binary.
const SCRUBBED_ENV: &[&str] = &["UTPM_CURRENT_DIR", "UTPM_DEBUG"];

/// An isolated set of directories for one test.
///
/// Layout below the temporary root:
/// - `project/`: the working directory `utpm` is run in
/// - `packages/`: local package storage (`TYPST_PACKAGE_PATH`)
/// - `cache/`: downloaded package storage (`TYPST_PACKAGE_CACHE_PATH`)
/// - `data/`: UTPM's own data directory (`UTPM_DATA_PATH`)
/// - `home/`: home directory, so no global git or ignore config is picked up
pub struct Sandbox {
    root: TempDir,
}

impl Sandbox {
    pub fn new() -> Self {
        let root = TempDir::new().expect("failed to create sandbox");
        for dir in ["project", "packages", "cache", "data", "home"] {
            fs::create_dir(root.path().join(dir)).expect("failed to create sandbox dir");
        }
        Self { root }
    }

    pub fn project(&self) -> PathBuf {
        self.root.path().join("project")
    }

    pub fn packages(&self) -> PathBuf {
        self.root.path().join("packages")
    }

    pub fn cache(&self) -> PathBuf {
        self.root.path().join("cache")
    }

    pub fn data(&self) -> PathBuf {
        self.root.path().join("data")
    }

    pub fn home(&self) -> PathBuf {
        self.root.path().join("home")
    }

    /// Writes a file relative to the project directory, creating parent directories.
    pub fn write(&self, relative: impl AsRef<Path>, content: &str) {
        let path = self.project().join(relative);
        fs::create_dir_all(path.parent().unwrap()).expect("failed to create parent dir");
        fs::write(path, content).expect("failed to write file");
    }

    /// Reads a file relative to the project directory.
    pub fn read(&self, relative: impl AsRef<Path>) -> String {
        fs::read_to_string(self.project().join(relative)).expect("failed to read file")
    }

    /// Creates a minimal package at `<namespace>/<name>/<version>` in local package storage.
    pub fn add_package(&self, namespace: &str, name: &str, version: &str) {
        create_package(&self.packages(), namespace, name, version);
    }

    /// Creates a minimal package at `<namespace>/<name>/<version>` in the package cache.
    pub fn add_cached_package(&self, namespace: &str, name: &str, version: &str) {
        create_package(&self.cache(), namespace, name, version);
    }

    /// Returns a `utpm` command running in the project directory with the sandboxed environment.
    fn cmd(&self, args: &[&str]) -> Command {
        let mut cmd = Command::new(env!("CARGO_BIN_EXE_utpm"));
        for var in SCRUBBED_ENV {
            cmd.env_remove(var);
        }
        cmd.args(args)
            .current_dir(self.project())
            .env("TYPST_PACKAGE_PATH", self.packages())
            .env("TYPST_PACKAGE_CACHE_PATH", self.cache())
            .env("UTPM_DATA_PATH", self.data())
            .env("HOME", self.home())
            .env("USERPROFILE", self.home())
            .env("XDG_CONFIG_HOME", self.home().join(".config"))
            .env("NO_COLOR", "1");
        cmd
    }

    /// Runs `utpm` with the given arguments in the project directory.
    pub fn utpm(&self, args: &[&str]) -> Run {
        Run::new(self.cmd(args))
    }
}

/// The captured result of one `utpm` invocation.
pub struct Run {
    args: String,
    pub output: Output,
}

impl Run {
    fn new(mut cmd: Command) -> Self {
        let args = cmd
            .get_args()
            .map(|arg| arg.to_string_lossy())
            .collect::<Vec<_>>()
            .join(" ");
        let output = cmd.output().expect("failed to run utpm");
        Self { args, output }
    }

    pub fn stdout(&self) -> String {
        String::from_utf8_lossy(&self.output.stdout).into_owned()
    }

    pub fn stderr(&self) -> String {
        String::from_utf8_lossy(&self.output.stderr).into_owned()
    }

    /// Asserts that the process exited successfully.
    #[track_caller]
    pub fn success(&self) -> &Self {
        assert!(
            self.output.status.success(),
            "`utpm {}` failed with {}\n{}",
            self.args,
            self.output.status,
            self.dump(),
        );
        self
    }

    fn dump(&self) -> String {
        format!(
            "--- stdout\n{}\n--- stderr\n{}",
            self.stdout(),
            self.stderr()
        )
    }
}

/// Lists all files below `dir` as sorted, `/`-separated relative paths.
pub fn files_in(dir: impl AsRef<Path>) -> Vec<String> {
    fn walk(base: &Path, dir: &Path, files: &mut Vec<String>) {
        for entry in fs::read_dir(dir).expect("failed to read dir") {
            let path = entry.expect("failed to read dir entry").path();
            if path.is_dir() {
                walk(base, &path, files);
            } else {
                let relative = path.strip_prefix(base).unwrap();
                let parts: Vec<_> = relative
                    .components()
                    .map(|c| c.as_os_str().to_string_lossy().into_owned())
                    .collect();
                files.push(parts.join("/"));
            }
        }
    }

    let dir = dir.as_ref();
    let mut files = vec![];
    walk(dir, dir, &mut files);
    files.sort();
    files
}

/// Returns a minimal `typst.toml` for the given package.
///
/// The manifest excludes `notes.md`, so tests can check that `exclude` is honored.
pub fn manifest_toml(name: &str, version: &str) -> String {
    format!(
        r#"[package]
name = "{name}"
version = "{version}"
entrypoint = "main.typ"
exclude = ["notes.md"]
"#
    )
}

/// Creates a minimal package at `<root>/<namespace>/<name>/<version>`.
fn create_package(root: &Path, namespace: &str, name: &str, version: &str) {
    let dir = root.join(namespace).join(name).join(version);
    fs::create_dir_all(&dir).expect("failed to create package dir");
    fs::write(dir.join("typst.toml"), manifest_toml(name, version))
        .expect("failed to write manifest");
    fs::write(dir.join("main.typ"), "").expect("failed to write entrypoint");
}
