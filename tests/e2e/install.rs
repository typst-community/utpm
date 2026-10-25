use crate::harness::*;

#[test]
fn install_from_local_path() {
    let sb = Sandbox::new();
    sb.write("typst.toml", &manifest_toml("my-pkg", "0.1.0"));
    sb.write("main.typ", "");

    sb.utpm(&["pkg", "install", "."]).success();

    assert_eq!(
        files_in(sb.packages().join("local/my-pkg/0.1.0")),
        ["main.typ", "typst.toml"]
    );
}
