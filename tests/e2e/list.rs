use crate::harness::*;

#[test]
fn list_shows_local_packages() {
    let sb = Sandbox::new();
    sb.add_package("local", "dep", "0.1.0");
    sb.add_package("local", "dep", "0.2.0");
    sb.add_cached_package("preview", "remote", "1.0.0");

    let out = sb.utpm(&["pkg", "list"]).success().stdout();

    assert!(out.contains("dep: 0.1.0, 0.2.0"), "{out}");
    assert!(!out.contains("remote"), "{out}");
}

#[test]
fn list_all_includes_cached_packages() {
    let sb = Sandbox::new();
    sb.add_package("local", "dep", "0.1.0");
    sb.add_cached_package("preview", "remote", "1.0.0");

    let out = sb.utpm(&["pkg", "list", "--all"]).success().stdout();

    assert!(out.contains("dep: 0.1.0"), "{out}");
    assert!(out.contains("remote: 1.0.0"), "{out}");
}
