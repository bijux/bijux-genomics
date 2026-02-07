use std::fs;
use std::path::PathBuf;

#[test]
fn public_module_tree_snapshot() -> anyhow::Result<()> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let lib_path = manifest_dir.join("src").join("lib.rs");
    let raw = fs::read_to_string(&lib_path)?;
    let mut mods = Vec::new();
    for line in raw.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("pub mod ") {
            mods.push(trimmed.to_string());
        }
    }
    let rendered = mods.join("\n");
    let snapshot_path = manifest_dir
        .join("tests")
        .join("snapshots")
        .join("public_module_tree.txt");
    let snapshot = fs::read_to_string(&snapshot_path)?;
    assert_eq!(rendered.trim(), snapshot.trim());
    Ok(())
}
