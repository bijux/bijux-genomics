#![allow(non_snake_case)]
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace root")
        .to_path_buf()
}

fn canonical_entries_from_index(root: &Path) -> Vec<(String, String)> {
    let mut rows = Vec::new();
    let mut current_id = None::<String>;
    let mut current_path = None::<String>;
    let mut canonical = false;

    for line in std::fs::read_to_string(root.join("examples/index.yaml"))
        .expect("read examples/index.yaml")
        .lines()
    {
        let trimmed = line.trim();
        if let Some(value) = trimmed.strip_prefix("- id: ") {
            if canonical {
                rows.push((
                    current_id.clone().expect("current id"),
                    current_path.clone().expect("current path"),
                ));
            }
            current_id = Some(value.to_string());
            current_path = None;
            canonical = false;
        } else if let Some(value) = trimmed.strip_prefix("canonical_example: ") {
            canonical = value == "true";
        } else if let Some(value) = trimmed.strip_prefix("path: ") {
            current_path = Some(value.to_string());
        }
    }

    if canonical {
        rows.push((current_id.expect("current id"), current_path.expect("current path")));
    }

    rows
}

#[test]
fn policy__contracts__canonical_examples_policy__canonical_examples_are_complete_and_indexed() {
    let root = workspace_root();
    let canonical_entries = canonical_entries_from_index(&root);
    let expected = BTreeSet::from([
        (
            "bam_essential_alignment_qc".to_string(),
            "examples/bam/essential-alignment-qc".to_string(),
        ),
        ("fastq_essential_qc".to_string(), "examples/fastq/essential-qc".to_string()),
        ("vcf_essential_qc_filter".to_string(), "examples/vcf/essential-qc-filter".to_string()),
    ]);
    let observed = canonical_entries.iter().cloned().collect::<BTreeSet<_>>();

    let mut offenders = Vec::new();
    if observed != expected {
        offenders
            .push(format!("canonical examples must be {:?} but observed {:?}", expected, observed));
    }

    for (example_id, rel) in canonical_entries {
        let dir = root.join(&rel);
        for rel_file in [
            "README.md",
            "example.toml",
            "golden/plan.json",
            "golden/explain.json",
            "golden/report.json",
            "tiny-inputs.json",
            "workflow-manifest.json",
            "expected-evidence.json",
        ] {
            if !dir.join(rel_file).is_file() {
                offenders.push(format!("{rel}: missing canonical file `{rel_file}`"));
            }
        }
        let example_toml = std::fs::read_to_string(dir.join("example.toml"))
            .unwrap_or_else(|_| panic!("read {rel}/example.toml"));
        for required in [
            "canonical_example = true",
            "workflow_manifest = \"workflow-manifest.json\"",
            "tiny_inputs_contract = \"tiny-inputs.json\"",
            "expected_plan = \"golden/plan.json\"",
            "expected_evidence = \"expected-evidence.json\"",
        ] {
            if !example_toml.contains(required) {
                offenders.push(format!("{rel}: missing example.toml contract `{required}`"));
            }
        }
        if !std::fs::read_to_string(dir.join("README.md"))
            .unwrap_or_else(|_| panic!("read {rel}/README.md"))
            .contains(&example_id)
        {
            offenders.push(format!("{rel}: README.md must mention `{example_id}`"));
        }
    }

    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "canonical example policy violations:\n{}",
        offenders.join("\n")
    );
}
