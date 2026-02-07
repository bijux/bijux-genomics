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
    let v1_mod_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("v1")
        .join("mod.rs");
    let v1_source = fs::read_to_string(v1_mod_path)?;
    let mut v1_modules = Vec::new();
    for line in v1_source.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("pub mod ") {
            if let Some(name) = rest.split([';', ' ']).next() {
                v1_modules.push(name.to_string());
            }
        }
    }
    v1_modules.sort();
    let mut v1_exports = Vec::new();
    for module in &v1_modules {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("v1")
            .join(format!("{module}.rs"));
        let contents = if path.exists() {
            fs::read_to_string(&path).ok()
        } else {
            let mod_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("src")
                .join("v1")
                .join(module)
                .join("mod.rs");
            fs::read_to_string(mod_path).ok()
        };
        let Some(contents) = contents else {
            continue;
        };
        for line in contents.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("pub use ") || trimmed.starts_with("pub fn ") {
                v1_exports.push(format!("{module}: {trimmed}"));
            }
        }
    }
    v1_exports.sort();

    let snapshot = format!(
        "{snapshot}\\n\\nv1 mod\\n  {}\\n\\nv1 exports\\n  {}",
        v1_modules.join("\\n  "),
        v1_exports.join("\\n  ")
    );
    insta::assert_snapshot!("public_surface", snapshot);
    Ok(())
}
