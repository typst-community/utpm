use crate::harness::*;

#[test]
fn metadata_prints_single_field() {
    let sb = Sandbox::new();
    sb.write("typst.toml", &manifest_toml("my-pkg", "0.1.0"));

    let out = sb
        .utpm(&["prj", "metadata", "--field", "version"])
        .success()
        .stdout();

    assert_eq!(out, "0.1.0\n");
}

#[test]
fn metadata_prints_json() {
    let sb = Sandbox::new();
    sb.write("typst.toml", &manifest_toml("my-pkg", "0.1.0"));

    let out = sb
        .utpm(&["--output-format", "json", "prj", "metadata"])
        .success()
        .stdout();

    assert!(out.contains(r#""name": "my-pkg""#), "{out}");
    assert!(out.contains(r#""version": "0.1.0""#), "{out}");
}
