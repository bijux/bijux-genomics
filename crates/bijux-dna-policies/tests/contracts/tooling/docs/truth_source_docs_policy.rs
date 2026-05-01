#![allow(non_snake_case)]
use std::path::{Path, PathBuf};

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace root")
        .to_path_buf()
}

#[test]
fn policy__contracts__truth_source_docs_policy__operations_docs_point_to_governed_truth() {
    let root = workspace_root();
    let command_map = std::fs::read_to_string(root.join("docs/30-operations/COMMAND_MAP.md"))
        .expect("read command map");
    let scoreboard = std::fs::read_to_string(root.join("docs/30-operations/BACKLOG_SCOREBOARD.md"))
        .expect("read backlog scoreboard doc");

    let mut offenders = Vec::new();
    for needle in [
        "../cli/command_snapshot.txt",
        "../../examples/index.yaml",
        "../../artifacts/planning/scoreboard.yaml",
        "../../artifacts/planning/cards.yaml",
    ] {
        if !command_map.contains(needle) {
            offenders.push(format!("docs/30-operations/COMMAND_MAP.md missing `{needle}`"));
        }
    }
    for needle in [
        "../../artifacts/planning/scoreboard.yaml",
        "../../artifacts/planning/cards.yaml",
        "../../artifacts/planning/issue_labels.yaml",
        "does not duplicate goal rows by hand",
    ] {
        if !scoreboard.contains(needle) {
            offenders.push(format!("docs/30-operations/BACKLOG_SCOREBOARD.md missing `{needle}`"));
        }
    }
    for forbidden in ["fastq_essential_qc", "bam_essential_alignment_qc", "vcf_essential_qc_filter"]
    {
        if command_map.contains(forbidden) || scoreboard.contains(forbidden) {
            offenders.push(format!(
                "operations truth docs must not hardcode canonical example ids such as `{forbidden}`"
            ));
        }
    }

    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "operations truth-source doc violations:\n{}",
        offenders.join("\n")
    );
}

#[test]
fn policy__contracts__truth_source_docs_policy__example_docs_reference_index_and_contract_files() {
    let root = workspace_root();
    let examples_readme =
        std::fs::read_to_string(root.join("examples/README.md")).expect("read examples/README.md");
    let examples_ref = std::fs::read_to_string(root.join("docs/50-reference/EXAMPLES.md"))
        .expect("read examples ref");

    let mut offenders = Vec::new();
    for raw in [&examples_readme, &examples_ref] {
        for needle in [
            "examples/index.yaml",
            "tiny-inputs.json",
            "workflow-manifest.json",
            "expected-evidence.json",
        ] {
            if !raw.contains(needle) {
                offenders.push(format!("examples docs missing `{needle}`"));
            }
        }
    }
    for forbidden in ["fastq_essential_qc", "bam_essential_alignment_qc", "vcf_essential_qc_filter"]
    {
        if examples_readme.contains(forbidden) || examples_ref.contains(forbidden) {
            offenders.push(format!(
                "example navigation docs must not list canonical example ids directly: `{forbidden}`"
            ));
        }
    }

    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "example truth-source doc violations:\n{}",
        offenders.join("\n")
    );
}
