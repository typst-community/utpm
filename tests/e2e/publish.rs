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

/// Working copy of the repository standing in for typst/packages.
fn upstream_seed(sb: &Sandbox) -> PathBuf {
    sb.home().join("seed")
}

/// The bare repository standing in for typst/packages, which `publish` reads
/// through `UTPM_PACKAGES_UPSTREAM`.
fn upstream(sb: &Sandbox) -> PathBuf {
    sb.home().join("upstream.git")
}

/// Commits a file to the upstream stand-in.
fn commit_upstream(sb: &Sandbox, file: &str) {
    let seed = upstream_seed(sb);
    std::fs::write(seed.join(file), file).unwrap();
    git(&seed, &["add", "."]);
    git(&seed, &["commit", "-q", "-m", file]);
    git(&seed, &["push", "-q", "origin", "main"]);
}

/// Creates the upstream stand-in and a bare repository standing in for the
/// user's fork of it.
fn fake_fork(sb: &Sandbox) -> PathBuf {
    let seed = upstream_seed(sb);
    std::fs::create_dir_all(seed.join(".github")).unwrap();
    std::fs::write(seed.join(".github/pull_request_template.md"), "checklist").unwrap();
    git(&seed, &["init", "-q", "-b", "main"]);
    git(
        &sb.home(),
        &["init", "-q", "--bare", "-b", "main", "upstream.git"],
    );
    git(
        &seed,
        &["remote", "add", "origin", upstream(sb).to_str().unwrap()],
    );
    commit_upstream(sb, "README.md");

    let fork = sb.home().join("fork.git");
    git(
        &sb.home(),
        &[
            "clone",
            "-q",
            "--bare",
            "upstream.git",
            fork.to_str().unwrap(),
        ],
    );
    fork
}

fn publish(sb: &Sandbox, args: &[&str]) -> Run {
    sb.utpm_with(args, |cmd| {
        cmd.envs(GIT_IDENTITY.iter().copied())
            .env("UTPM_PACKAGES_UPSTREAM", upstream(sb))
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

#[test]
fn publish_commits_with_the_identity_of_the_package_repository() {
    let sb = Sandbox::new();
    write_package(&sb);
    let fork = fake_fork(&sb);
    git(&sb.project(), &["init", "-q"]);
    git(&sb.project(), &["config", "user.name", "Package Author"]);
    git(
        &sb.project(),
        &["config", "user.email", "author@example.com"],
    );

    sb.utpm_with(
        &["prj", "publish", "--fork", fork.to_str().unwrap()],
        |cmd| {
            cmd.env("UTPM_PACKAGES_UPSTREAM", upstream(&sb))
                .env_remove("UTPM_GITHUB_TOKEN");
            for (var, _) in GIT_IDENTITY {
                cmd.env_remove(var);
            }
        },
    )
    .success();

    assert_eq!(
        git(&fork, &["log", "-1", "--format=%an <%ae>", "my-pkg-0.1.0"]),
        "Package Author <author@example.com>"
    );
}

/// Runs `publish` without any git identity, as on a fresh CI runner. Git is
/// told not to guess one from the host name, which it may do otherwise.
fn publish_without_identity(sb: &Sandbox, fork: &Path) -> Run {
    std::fs::write(
        sb.home().join(".gitconfig"),
        "[user]\n\tuseConfigOnly = true\n",
    )
    .unwrap();
    sb.utpm_with(
        &["prj", "publish", "--fork", fork.to_str().unwrap()],
        |cmd| {
            cmd.env("UTPM_PACKAGES_UPSTREAM", upstream(sb))
                .env("GIT_CONFIG_NOSYSTEM", "1")
                .env_remove("EMAIL")
                .env_remove("UTPM_GITHUB_TOKEN");
            for (var, _) in GIT_IDENTITY {
                cmd.env_remove(var);
            }
        },
    )
}

#[test]
fn publish_falls_back_to_the_author_of_the_last_commit() {
    let sb = Sandbox::new();
    write_package(&sb);
    let fork = fake_fork(&sb);
    git(&sb.project(), &["init", "-q"]);
    git(&sb.project(), &["add", "."]);
    git(
        &sb.project(),
        &[
            "commit",
            "-q",
            "--author=Last Author <last@example.com>",
            "-m",
            "init",
        ],
    );

    publish_without_identity(&sb, &fork).success();

    assert_eq!(
        git(&fork, &["log", "-1", "--format=%an <%ae>", "my-pkg-0.1.0"]),
        "Last Author <last@example.com>"
    );
}

#[test]
fn publish_without_identity_fails_before_touching_the_fork() {
    let sb = Sandbox::new();
    write_package(&sb);
    let fork = fake_fork(&sb);

    let run = publish_without_identity(&sb, &fork);

    run.failure();
    assert!(run.stdout().contains("No git identity"), "{}", run.stdout());
    assert!(!sb.data().join("git-packages").exists());
}

#[test]
fn publish_warns_when_fork_is_behind_without_token() {
    let sb = Sandbox::new();
    write_package(&sb);
    let fork = fake_fork(&sb);
    commit_upstream(&sb, "newer.md");

    let run = publish(&sb, &["prj", "publish", "--fork", fork.to_str().unwrap()]);
    run.success();
    assert!(
        run.stdout()
            .contains("Your fork is 1 commits behind typst/packages")
    );
}

#[test]
fn publish_rejects_fork_with_commits_of_its_own() {
    let sb = Sandbox::new();
    write_package(&sb);
    let fork = fake_fork(&sb);

    let clone = sb.home().join("fork-clone");
    git(
        &sb.home(),
        &[
            "clone",
            "-q",
            fork.to_str().unwrap(),
            clone.to_str().unwrap(),
        ],
    );
    std::fs::write(clone.join("own.md"), "own").unwrap();
    git(&clone, &["add", "."]);
    git(&clone, &["commit", "-q", "-m", "own"]);
    git(&clone, &["push", "-q", "origin", "main"]);

    let run = publish(&sb, &["prj", "publish", "--fork", fork.to_str().unwrap()]);
    run.failure();
    assert!(
        run.stdout()
            .contains("has 1 commits that are not in typst/packages")
    );
    assert_eq!(git(&fork, &["branch", "--list", "my-pkg-0.1.0"]), "");
}
