use crate::harness::*;

#[test]
fn link_copies_project_into_local_namespace() {
    let sb = Sandbox::new();
    sb.write("typst.toml", &manifest_toml("my-pkg", "0.1.0"));
    sb.write("main.typ", "#import \"lib/util.typ\": *");
    sb.write("lib/util.typ", "#let answer = 42");
    sb.write("notes.md", "not part of the package");

    sb.utpm(&["prj", "link"]).success();

    let linked = sb.packages().join("local/my-pkg/0.1.0");
    assert_eq!(
        files_in(&linked),
        ["lib/util.typ", "main.typ", "typst.toml"]
    );
    assert_eq!(
        std::fs::read_to_string(linked.join("lib/util.typ")).unwrap(),
        "#let answer = 42"
    );
}

#[test]
fn link_force_replaces_existing_package() {
    let sb = Sandbox::new();
    sb.write("typst.toml", &manifest_toml("my-pkg", "0.1.0"));
    sb.write("main.typ", "old");
    sb.write("lib/util.typ", "old");
    sb.utpm(&["prj", "link"]).success();

    std::fs::remove_dir_all(sb.project().join("lib")).unwrap();
    sb.write("main.typ", "new");
    sb.utpm(&["prj", "link", "--force"]).success();

    let linked = sb.packages().join("local/my-pkg/0.1.0");
    assert_eq!(files_in(&linked), ["main.typ", "typst.toml"]);
    assert_eq!(
        std::fs::read_to_string(linked.join("main.typ")).unwrap(),
        "new"
    );
}

#[test]
fn link_without_force_keeps_existing_package() {
    let sb = Sandbox::new();
    sb.write("typst.toml", &manifest_toml("my-pkg", "0.1.0"));
    sb.write("main.typ", "old");
    sb.utpm(&["prj", "link"]).success();

    sb.write("main.typ", "new");
    sb.utpm(&["prj", "link"]).failure();

    let linked = sb.packages().join("local/my-pkg/0.1.0");
    assert_eq!(
        std::fs::read_to_string(linked.join("main.typ")).unwrap(),
        "old"
    );
}

#[test]
fn link_respects_typstignore() {
    let sb = Sandbox::new();
    sb.write("typst.toml", &manifest_toml("my-pkg", "0.1.0"));
    sb.write("main.typ", "");
    sb.write(".typstignore", "drafts/\n");
    sb.write("drafts/wip.typ", "");

    sb.utpm(&["prj", "link"]).success();

    let linked = sb.packages().join("local/my-pkg/0.1.0");
    assert_eq!(files_in(&linked), ["main.typ", "typst.toml"]);
}

// Creating directory symlinks on Windows requires developer mode or elevated privileges.
#[cfg(unix)]
#[test]
fn link_no_copy_creates_symlink() {
    let sb = Sandbox::new();
    sb.write("typst.toml", &manifest_toml("my-pkg", "0.1.0"));
    sb.write("main.typ", "");

    sb.utpm(&["prj", "link", "--no-copy"]).success();

    let linked = sb.packages().join("local/my-pkg/0.1.0");
    assert!(linked.symlink_metadata().unwrap().is_symlink());
    assert_eq!(
        linked.canonicalize().unwrap(),
        sb.project().canonicalize().unwrap()
    );
}

#[test]
fn link_dry_run_writes_nothing() {
    let sb = Sandbox::new();
    sb.write("typst.toml", &manifest_toml("my-pkg", "0.1.0"));
    sb.write("main.typ", "");

    sb.utpm(&["--dry-run", "prj", "link"]).success();

    assert!(files_in(sb.packages()).is_empty());
}

#[test]
fn link_without_manifest_fails() {
    let sb = Sandbox::new();
    sb.write("main.typ", "");

    sb.utpm(&["prj", "link"]).failure();

    assert!(files_in(sb.packages()).is_empty());
}
