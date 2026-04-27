#![allow(non_snake_case)]
#[path = "../../../support/fs.rs"]
mod support;

use walkdir::WalkDir;

#[test]
fn policy__contracts__smoke_manifest_policy__container_smoke_manifests_include_image_identity() {
    let root = support::workspace_root();
    let source_dir = root.join("crates/bijux-dna-dev/src/commands/containers");
    let required_tokens = [
        "\"runtime\"",
        "\"image\"",
        "\"resolved_image_digest\"",
        "\"declared_version\"",
        "\"upstream\"",
        "\"upstream_checksum\"",
        "\"smoke_version_cmd\"",
        "\"version_output\"",
    ];

    let mut sources = WalkDir::new(&source_dir)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_file())
        .filter(|entry| entry.path().extension().is_some_and(|ext| ext == "rs"))
        .map(walkdir::DirEntry::into_path)
        .collect::<Vec<_>>();
    sources.sort();

    assert!(
        !sources.is_empty(),
        "expected container command sources under {}",
        source_dir.display()
    );

    let mut offenders = Vec::new();
    let raw =
        sources.iter().map(|path| support::read_to_string(path)).collect::<Vec<_>>().join("\n");
    for token in required_tokens {
        if !raw.contains(token) {
            offenders.push(format!("{} missing manifest token {}", source_dir.display(), token));
        }
    }

    assert!(
        offenders.is_empty(),
        "container smoke manifest policy violations:\n{}",
        offenders.join("\n")
    );
}
