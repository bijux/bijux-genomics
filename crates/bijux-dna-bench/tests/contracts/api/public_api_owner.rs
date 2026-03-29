use std::fs;

#[test]
fn public_api_has_owner_doc() -> anyhow::Result<()> {
    let manifest_dir = crate::support::crate_root("bijux-dna-bench")?;
    let lib_path = manifest_dir.join("src").join("lib.rs");
    let raw = fs::read_to_string(&lib_path)?;
    assert!(
        raw.lines()
            .any(|line| line.contains("Owner: bijux-dna-bench")),
        "lib.rs must include an Owner doc comment"
    );
    Ok(())
}
