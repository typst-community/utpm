use crate::harness::*;

#[test]
fn generate_prints_bash_completions() {
    let sb = Sandbox::new();

    let out = sb.utpm(&["generate", "bash"]).success().stdout();

    assert!(out.contains("_utpm()"), "{out}");
}
