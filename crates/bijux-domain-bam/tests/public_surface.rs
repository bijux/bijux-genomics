use std::fs;
use std::path::PathBuf;

#[test]
fn public_surface_is_constrained() {
    let lib_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("lib.rs");
    let source = fs::read_to_string(lib_path).expect("read lib.rs");
    let mut pub_mods = Vec::new();
    let mut pub_use_lines = Vec::new();
    for line in source.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("pub mod ") {
            if let Some(name) = rest.split(|c| c == ';' || c == ' ').next() {
                pub_mods.push(name.to_string());
            }
        }
        if trimmed.starts_with("pub use ") {
            pub_use_lines.push(trimmed.to_string());
        }
    }

    let allowed_mods = ["metrics", "params", "types"];
    for name in &pub_mods {
        assert!(
            allowed_mods.contains(&name.as_str()),
            "unexpected public module: {name}"
        );
    }
    for line in &pub_use_lines {
        assert!(
            line.contains("bam_stage_registry"),
            "unexpected public re-export: {line}"
        );
    }
}
