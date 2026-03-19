#![allow(non_snake_case)]
#[path = "../../../support/fs.rs"]
mod support;

#[test]
fn policy__contracts__smoke_manifest_policy__container_smoke_manifests_include_image_identity() {
    let root = support::workspace_root();
    let source = root.join("crates/bijux-dev-dna/src/native/containers.rs");
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

    let mut offenders = Vec::new();
    let raw = std::fs::read_to_string(&source)
        .unwrap_or_else(|_| panic!("read {}", source.display()));
    for token in required_tokens {
        if !raw.contains(token) {
            offenders.push(format!(
                "{} missing manifest token {}",
                source.display(),
                token
            ));
        }
    }

    assert!(
        offenders.is_empty(),
        "container smoke manifest policy violations:\n{}",
        offenders.join("\n")
    );
}
