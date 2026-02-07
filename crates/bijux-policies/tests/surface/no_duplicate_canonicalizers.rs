#[path = "../support/fs.rs"]
mod support;

use walkdir::WalkDir;

#[test]
fn core_is_only_canonicalizer() {
    let root = support::workspace_root();
    let mut offenders = Vec::new();
    for entry in WalkDir::new(root.join("crates"))
        .into_iter()
        .filter_map(|entry| entry.ok())
    {
        if !entry.file_type().is_file() {
            continue;
        }
        if entry.path().extension().and_then(|ext| ext.to_str()) != Some("rs") {
            continue;
        }
        let path_str = entry.path().to_string_lossy();
        if path_str.contains("/crates/bijux-core/") {
            continue;
        }
        let content = support::read_to_string(entry.path());
        if content.contains("canonicalize_json_value")
            || content.contains("normalize_numbers_and_paths")
            || content.contains("to_canonical_json_bytes")
        {
            offenders.push(entry.path().display().to_string());
        }
    }

    assert!(
        offenders.is_empty(),
        "Canonicalization must live in bijux-core only.\n\
Use bijux_core::contract::canonical instead of re-implementing.\n\
See STYLE.md.\n\
Offenders:\n{}",
        offenders.join("\n")
    );
}
