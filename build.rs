use shadow_rs::{self, BuildPattern, ShadowBuilder};

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    ShadowBuilder::builder()
        .build_pattern(BuildPattern::Lazy)
        .build()
        .unwrap();
}
