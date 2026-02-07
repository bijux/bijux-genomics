use std::fs;
use std::path::PathBuf;

#[test]
fn public_surface_is_snapshotted() -> anyhow::Result<()> {
    let lib_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("lib.rs");
    let source = fs::read_to_string(lib_path)?;
    let mut pub_mods = Vec::new();
    let mut pub_use_lines = Vec::new();
    for line in source.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("pub mod ") {
            if let Some(name) = rest.split([';', ' ']).next() {
                pub_mods.push(name.to_string());
            }
        }
        if trimmed.starts_with("pub use ") {
            pub_use_lines.push(trimmed.to_string());
        }
    }
    pub_mods.sort();
    pub_use_lines.sort();

    let snapshot = format!(
        "pub mod\\n  {}\\n\\npub use\\n  {}",
        pub_mods.join("\\n  "),
        pub_use_lines.join("\\n  ")
    );
    insta::assert_snapshot!("public_surface", snapshot);
    Ok(())
}
