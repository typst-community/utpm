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
