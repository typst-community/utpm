use crate::harness::*;

#[test]
fn clone_into_new_directory() {
    let sb = Sandbox::new();
    sb.add_cached_package("preview", "foo", "1.0.0");

    sb.utpm(&["prj", "clone", "@preview/foo:1.0.0", "out"])
        .success();

    assert_eq!(files_in(sb.project()), ["out/main.typ", "out/typst.toml"]);
}

#[test]
fn clone_copies_cached_package() {
    let sb = Sandbox::new();
    sb.add_cached_package("preview", "foo", "1.0.0");

    sb.utpm(&["prj", "clone", "@preview/foo:1.0.0"]).success();

    assert_eq!(files_in(sb.project()), ["main.typ", "typst.toml"]);
    assert_eq!(sb.read("typst.toml"), manifest_toml("foo", "1.0.0"));
}
