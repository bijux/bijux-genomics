mod support;

use std::path::PathBuf;

#[test]
fn no_deep_modules_in_src() -> std::io::Result<()> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let src_dir = manifest_dir.join("src");
    support::guardrails::assert_module_depth(&src_dir)
}
