use std::fs;
use std::path::PathBuf;

#[test]
fn public_api_has_owner_doc() -> anyhow::Result<()> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let lib_path = manifest_dir.join("src").join("lib.rs");
    let raw = fs::read_to_string(&lib_path)?;
    assert!(
        raw.lines().any(|line| line.contains("Owner: bijux-bench")),
        "lib.rs must include an Owner doc comment"
    );
    Ok(())
}
