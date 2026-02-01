mod support;

use std::path::PathBuf;

#[test]
fn file_loc_budget() -> std::io::Result<()> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let src_dir = manifest_dir.join("src");
    let strict = std::env::var("BIJUX_STRICT").is_ok();
    support::guardrails::assert_loc_budget(&src_dir, strict)
}
