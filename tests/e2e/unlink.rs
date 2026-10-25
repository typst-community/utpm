use crate::harness::*;

#[test]
fn unlink_removes_single_version() {
    let sb = Sandbox::new();
    sb.add_package("local", "dep", "0.1.0");
    sb.add_package("local", "dep", "0.2.0");

    sb.utpm(&["pkg", "unlink", "--yes", "@local/dep:0.1.0"])
        .success();

    assert_eq!(
        files_in(sb.packages()),
        ["local/dep/0.2.0/main.typ", "local/dep/0.2.0/typst.toml"]
    );
}
