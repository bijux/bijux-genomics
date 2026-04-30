#![allow(non_snake_case)]
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use serde_json::Value;

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace root")
        .to_path_buf()
}

#[test]
fn policy__contracts__refusal_bundle_policy__required_refusal_bundles_exist_and_are_index_free() {
    let root = workspace_root();
    let failures_root = root.join("examples/failures");
    let expected = BTreeSet::from([
        "broken-fastq".to_string(),
        "missing-bam-index".to_string(),
        "reference-mismatch".to_string(),
        "malformed-vcf-header".to_string(),
        "unsafe-cache-hit".to_string(),
        "simulation-as-production".to_string(),
    ]);
    let observed = std::fs::read_dir(&failures_root)
        .unwrap_or_else(|_| panic!("read {}", failures_root.display()))
        .flatten()
        .filter(|entry| entry.path().is_dir())
        .map(|entry| entry.file_name().to_string_lossy().to_string())
        .collect::<BTreeSet<_>>();

    let mut offenders = Vec::new();
    if observed != expected {
        offenders.push(format!("failure bundle set mismatch: expected {:?}, observed {:?}", expected, observed));
    }

    let examples_index =
        std::fs::read_to_string(root.join("examples/index.yaml")).expect("read examples/index.yaml");
    if examples_index.contains("examples/failures/") {
        offenders.push("examples/index.yaml must not list failure bundle directories".to_string());
    }

    let policy =
        std::fs::read_to_string(root.join("examples/POLICY.md")).expect("read examples/POLICY.md");
    for needle in ["examples/failures/", "refusal-bundle.json", "must not be listed in `examples/index.yaml`"] {
        if !policy.contains(needle) {
            offenders.push(format!("examples/POLICY.md missing `{needle}`"));
        }
    }

    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "refusal bundle root policy violations:\n{}",
        offenders.join("\n")
    );
}

#[test]
fn policy__contracts__refusal_bundle_policy__refusal_bundle_contracts_are_complete() {
    let root = workspace_root();
    let failures_root = root.join("examples/failures");
    let mut offenders = Vec::new();

    for bundle in [
        "broken-fastq",
        "missing-bam-index",
        "reference-mismatch",
        "malformed-vcf-header",
        "unsafe-cache-hit",
        "simulation-as-production",
    ] {
        let dir = failures_root.join(bundle);
        let readme = dir.join("README.md");
        let refusal = dir.join("refusal-bundle.json");
        if !readme.is_file() {
            offenders.push(format!("{bundle}: missing README.md"));
        }
        if !refusal.is_file() {
            offenders.push(format!("{bundle}: missing refusal-bundle.json"));
            continue;
        }
        let value: Value = serde_json::from_str(
            &std::fs::read_to_string(&refusal).unwrap_or_else(|_| panic!("read {}", refusal.display())),
        )
        .unwrap_or_else(|_| panic!("parse {}", refusal.display()));
        for key in ["schema_version", "bundle_id", "category", "expected_refusal_codes", "operating_mode", "summary"] {
            if value.get(key).is_none() {
                offenders.push(format!("{bundle}: refusal-bundle.json missing `{key}`"));
            }
        }
        if value.get("expected_refusal_codes").and_then(Value::as_array).is_none() {
            offenders.push(format!("{bundle}: expected_refusal_codes must be an array"));
        }
        if value.get("operating_mode").and_then(Value::as_str) != Some("enforced") {
            offenders.push(format!("{bundle}: operating_mode must stay `enforced`"));
        }
    }

    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "refusal bundle contract violations:\n{}",
        offenders.join("\n")
    );
}
