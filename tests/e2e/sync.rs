use crate::harness::*;

#[test]
fn sync_compares_versions_numerically() {
    let sb = Sandbox::new();
    sb.add_package("local", "dep", "0.9.0");
    sb.add_package("local", "dep", "0.10.0");
    sb.write("main.typ", "#import \"@local/dep:0.9.0\": *\n");

    sb.utpm(&["prj", "sync"]).success();

    assert_eq!(
        sb.read("main.typ"),
        "#import \"@local/dep:0.10.0\" /* From 0.9.0 */: *\n"
    );
}

#[test]
fn sync_updates_local_imports_to_latest() {
    let sb = Sandbox::new();
    sb.add_package("local", "dep", "0.1.0");
    sb.add_package("local", "dep", "0.2.0");
    sb.write("main.typ", "#import \"@local/dep:0.1.0\": *\n");

    sb.utpm(&["prj", "sync"]).success();

    assert_eq!(
        sb.read("main.typ"),
        "#import \"@local/dep:0.2.0\" /* From 0.1.0 */: *\n"
    );
}

#[test]
fn sync_check_only_leaves_files_unchanged() {
    let sb = Sandbox::new();
    sb.add_package("local", "dep", "0.1.0");
    sb.add_package("local", "dep", "0.2.0");
    sb.write("main.typ", "#import \"@local/dep:0.1.0\": *\n");

    sb.utpm(&["prj", "sync", "--check-only"]).success();

    assert_eq!(sb.read("main.typ"), "#import \"@local/dep:0.1.0\": *\n");
}
