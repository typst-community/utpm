use typst_syntax::package::PackageManifest;

use crate::harness::*;

fn read_manifest(sb: &Sandbox) -> PackageManifest {
    toml::from_str(&sb.read("typst.toml")).expect("invalid manifest")
}

#[test]
fn init_cli_writes_manifest() {
    let sb = Sandbox::new();

    sb.utpm(&[
        "prj",
        "init",
        "--cli",
        "--name",
        "my-pkg",
        "--version",
        "0.1.0",
        "--authors",
        "Alice,Bob",
        "--license",
        "MIT",
    ])
    .success();

    let manifest = read_manifest(&sb).package;
    assert_eq!(manifest.name, "my-pkg");
    assert_eq!(manifest.version.to_string(), "0.1.0");
    assert_eq!(manifest.entrypoint, "main.typ");
    assert_eq!(manifest.authors, ["Alice", "Bob"]);
    assert_eq!(manifest.license.as_deref(), Some("MIT"));
}

#[test]
fn init_without_force_keeps_existing_manifest() {
    let sb = Sandbox::new();
    sb.write("typst.toml", &manifest_toml("existing", "1.0.0"));

    sb.utpm(&["prj", "init", "--cli", "--name", "my-pkg"]);

    assert_eq!(sb.read("typst.toml"), manifest_toml("existing", "1.0.0"));
}

#[test]
fn init_force_overwrites_existing_manifest() {
    let sb = Sandbox::new();
    sb.write("typst.toml", &manifest_toml("existing", "1.0.0"));

    sb.utpm(&["prj", "init", "--cli", "--force", "--name", "my-pkg"])
        .success();

    assert_eq!(read_manifest(&sb).package.name, "my-pkg");
}

#[test]
fn init_writes_manifest_to_utpm_current_dir() {
    let sb = Sandbox::new();
    let target = sb.project().join("sub");
    std::fs::create_dir(&target).unwrap();

    sb.utpm_with(&["prj", "init", "--cli", "--name", "my-pkg"], |cmd| {
        cmd.env("UTPM_CURRENT_DIR", &target);
    })
    .success();

    assert_eq!(files_in(sb.project()), ["sub/typst.toml"]);
}
