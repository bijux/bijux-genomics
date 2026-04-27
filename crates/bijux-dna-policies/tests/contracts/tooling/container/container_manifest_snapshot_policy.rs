#![allow(non_snake_case)]
#[path = "../../../support/fs.rs"]
mod support;

use std::process::Command;

fn normalized_json(input: &str) -> String {
    let value: serde_json::Value =
        serde_json::from_str(input).unwrap_or_else(|err| panic!("parse manifest json: {err}"));
    serde_json::to_string_pretty(&value)
        .unwrap_or_else(|err| panic!("pretty-print manifest json: {err}"))
}

fn cargo_target_dir(root: &std::path::Path) -> std::path::PathBuf {
    std::env::var_os("CARGO_TARGET_DIR")
        .map_or_else(|| root.join("artifacts/rust/target"), std::path::PathBuf::from)
}

#[test]
fn slow__policy__contracts__container_manifest_snapshot_policy__generated_manifest_matches_committed_snapshot(
) {
    let root = support::workspace_root();
    let snapshot_path = root.join(
        "crates/bijux-dna-policies/tests/fixtures/container_manifest_snapshot/manifest.snapshot.json",
    );
    let expected = std::fs::read_to_string(&snapshot_path)
        .unwrap_or_else(|err| panic!("read {}: {err}", snapshot_path.display()));

    let build = Command::new("cargo")
        .arg("build")
        .arg("--release")
        .arg("-p")
        .arg("bijux-dna")
        .arg("--bin")
        .arg("bijux-dna")
        .current_dir(&root)
        .status()
        .unwrap_or_else(|err| panic!("build release bijux binary: {err}"));
    assert!(build.success(), "release build failed");

    let bijux_bin = cargo_target_dir(&root).join("release/bijux-dna");
    let output = Command::new(&bijux_bin)
        .arg("registry")
        .arg("export-containers")
        .arg("--json")
        .current_dir(&root)
        .output()
        .unwrap_or_else(|err| {
            panic!("run registry export-containers --json with {}: {err}", bijux_bin.display())
        });
    assert!(
        output.status.success(),
        "registry export-containers failed: status={} stderr={}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );
    let actual = String::from_utf8(output.stdout).expect("manifest output utf8");

    assert_eq!(
        normalized_json(&actual).trim(),
        normalized_json(&expected).trim(),
        "container manifest snapshot is stale. Regenerate with: bijux-dna registry export-containers --json > crates/bijux-dna-policies/tests/fixtures/container_manifest_snapshot/manifest.snapshot.json"
    );
}
