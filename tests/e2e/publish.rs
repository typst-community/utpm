use std::path::{Path, PathBuf};
use std::process::Command;

use crate::harness::*;

const GIT_IDENTITY: &[(&str, &str)] = &[
    ("GIT_AUTHOR_NAME", "Test"),
    ("GIT_AUTHOR_EMAIL", "test@example.com"),
    ("GIT_COMMITTER_NAME", "Test"),
    ("GIT_COMMITTER_EMAIL", "test@example.com"),
];

fn git(dir: &Path, args: &[&str]) -> String {
    let output = Command::new("git")
        .current_dir(dir)
        .args(args)
        .envs(GIT_IDENTITY.iter().copied())
        .output()
        .expect("failed to run git");
    assert!(
        output.status.success(),
        "git {args:?} failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout).trim().to_string()
}

/// Creates a bare repository standing in for the user's fork of typst/packages.
fn fake_fork(sb: &Sandbox) -> PathBuf {
    let seed = sb.home().join("seed");
    std::fs::create_dir_all(seed.join(".github")).unwrap();
    std::fs::write(seed.join("README.md"), "packages").unwrap();
    std::fs::write(seed.join(".github/pull_request_template.md"), "checklist").unwrap();
    git(&seed, &["init", "-q", "-b", "main"]);
    git(&seed, &["add", "."]);
    git(&seed, &["commit", "-q", "-m", "init"]);

    let fork = sb.home().join("fork.git");
    git(
        &sb.home(),
        &["clone", "-q", "--bare", "seed", fork.to_str().unwrap()],
    );
    fork
}

fn publish(sb: &Sandbox, args: &[&str]) -> Run {
    sb.utpm_with(args, |cmd| {
        cmd.envs(GIT_IDENTITY.iter().copied())
            .env_remove("UTPM_GITHUB_TOKEN");
    })
}

fn write_package(sb: &Sandbox) {
    sb.write("typst.toml", &manifest_toml("my-pkg", "0.1.0"));
    sb.write("main.typ", "v1");
    sb.write("notes.md", "excluded");
}

#[test]
fn publish_dry_run_lists_files_without_side_effects() {
    let sb = Sandbox::new();
    write_package(&sb);

    let run = publish(&sb, &["--dry-run", "prj", "publish"]);
    run.success();

    let output = run.stdout() + &run.stderr();
    assert!(output.contains("main.typ") && output.contains("typst.toml"));
    assert!(!output.contains("notes.md"));
    assert_eq!(files_in(sb.data()), Vec::<String>::new());
}

#[test]
fn publish_rejects_package_without_entrypoint() {
    let sb = Sandbox::new();
    sb.write("typst.toml", &manifest_toml("my-pkg", "0.1.0"));

    let run = publish(&sb, &["--dry-run", "prj", "publish"]);
    run.failure();
    assert!(run.stdout().contains("Can't find main.typ file"));
}

#[test]
fn publish_without_fork_or_token_fails() {
    let sb = Sandbox::new();
    write_package(&sb);

    let run = publish(&sb, &["prj", "publish"]);
    run.failure();
    assert!(run.stdout().contains("No fork of typst/packages found"));
}

#[test]
fn publish_create_pr_requires_token() {
    let sb = Sandbox::new();
    write_package(&sb);
    let fork = fake_fork(&sb);

    let run = publish(
        &sb,
        &[
            "prj",
            "publish",
            "--create-pr",
            "--fork",
            fork.to_str().unwrap(),
        ],
    );
    run.failure();
    assert!(run.stdout().contains("--create-pr needs a GitHub token"));
    assert_eq!(git(&fork, &["branch", "--list", "my-pkg-0.1.0"]), "");
}

#[test]
fn publish_pushes_package_branch_and_updates_it_on_rerun() {
    let sb = Sandbox::new();
    write_package(&sb);
    let fork = fake_fork(&sb);

    publish(&sb, &["prj", "publish", "--fork", fork.to_str().unwrap()]).success();

    let branch = "my-pkg-0.1.0";
    assert_eq!(
        git(&fork, &["ls-tree", "-r", "--name-only", branch, "packages"]),
        "packages/preview/my-pkg/0.1.0/main.typ\npackages/preview/my-pkg/0.1.0/typst.toml"
    );

    // The second run finds the fork through the existing checkout.
    sb.write("main.typ", "v2");
    publish(&sb, &["prj", "publish"]).success();

    let main_typ = format!("{branch}:packages/preview/my-pkg/0.1.0/main.typ");
    assert_eq!(git(&fork, &["show", &main_typ]), "v2");
    assert_eq!(
        git(&fork, &["rev-list", "--count", &format!("main..{branch}")]),
        "1"
    );
    assert_eq!(
        git(&fork, &["log", "-1", "--format=%s", branch]),
        "my-pkg:0.1.0"
    );
}
