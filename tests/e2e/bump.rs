use crate::harness::*;

#[test]
fn bump_updates_manifest_and_tagged_files() {
    let sb = Sandbox::new();
    sb.write("typst.toml", &manifest_toml("my-pkg", "0.1.0"));
    sb.write("README.md", "<version>0.1.0</version> since 0.1.0");

    sb.utpm(&[
        "prj",
        "bump",
        "0.2.0",
        "--tag",
        "version",
        "--include",
        "README.md",
    ])
    .success();

    assert_eq!(sb.read("typst.toml"), manifest_toml("my-pkg", "0.2.0"));
    assert_eq!(sb.read("README.md"), "<version>0.2.0</version> since 0.1.0");
}
