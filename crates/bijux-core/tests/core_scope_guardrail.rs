use std::path::Path;

use anyhow::Result;

#[test]
fn core_scope_only_allows_contracts_and_foundation() -> Result<()> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let src = root.join("src");
    let allow_dirs = ["contract", "foundation", "metrics"];
    let allow_files = ["lib.rs", "boundaries.md", "ids.rs", "prelude.rs"];

    for entry in std::fs::read_dir(&src)? {
        let entry = entry?;
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();
        if path.is_dir() {
            assert!(
                allow_dirs.contains(&name.as_str()),
                "core scope violation: unexpected dir {name}"
            );
            continue;
        }
        if path.is_file() {
            assert!(
                allow_files.contains(&name.as_str()),
                "core scope violation: unexpected file {name}"
            );
        }
    }

    let lib_path = src.join("lib.rs");
    let content = std::fs::read_to_string(&lib_path)?;
    let mut pub_mods = Vec::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("pub mod ") {
            if let Some(name) = rest.split([';', ' ']).next() {
                pub_mods.push(name.to_string());
            }
        }
    }
    pub_mods.sort();
    let allowed_pub_mods = ["contract", "foundation", "ids", "metrics", "prelude"];
    for module in pub_mods {
        assert!(
            allowed_pub_mods.contains(&module.as_str()),
            "core public module not allowed: {module}"
        );
    }
    Ok(())
}
