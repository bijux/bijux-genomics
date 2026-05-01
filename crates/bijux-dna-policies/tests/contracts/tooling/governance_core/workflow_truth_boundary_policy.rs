#![allow(non_snake_case)]

use std::path::{Path, PathBuf};

use walkdir::WalkDir;

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .unwrap_or_else(|| bijux_dna_policies::policy_panic!("resolve repository root"))
        .to_path_buf()
}

#[test]
fn policy__contracts__workflow_truth_boundary_policy__cli_and_dev_do_not_define_plan_sidecar_schemas(
) {
    let root = repo_root();
    let mut offenders = Vec::new();

    for rel in ["crates/bijux-dna/src", "crates/bijux-dna-dev/src"] {
        for entry in WalkDir::new(root.join(rel))
            .into_iter()
            .filter_map(Result::ok)
            .filter(|entry| entry.file_type().is_file())
            .filter(|entry| entry.path().extension().and_then(|ext| ext.to_str()) == Some("rs"))
        {
            let raw = std::fs::read_to_string(entry.path()).unwrap_or_else(|err| {
                bijux_dna_policies::policy_panic!("read {}: {err}", entry.path().display())
            });
            for needle in
                ["bijux.plan_artifacts.v1", "bijux.decision_trace.v1", "bijux.policy_snapshot.v1"]
            {
                if raw.contains(needle) {
                    offenders.push(format!("{} defines `{needle}`", entry.path().display()));
                }
            }
        }
    }

    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "CLI and dev crates must consume plan sidecar truth from the API surface:\n{}",
        offenders.join("\n")
    );
}

#[test]
fn policy__contracts__workflow_truth_boundary_policy__migration_note_documents_api_authority() {
    let root = repo_root();
    let path = root.join("docs/10-architecture/WORKFLOW_TRUTH_MIGRATION.md");
    let raw = std::fs::read_to_string(&path)
        .unwrap_or_else(|err| bijux_dna_policies::policy_panic!("read {}: {err}", path.display()));

    for needle in [
        "Move workflow truth out of `bijux-dna-dev` and CLI helper modules.",
        "`crates/bijux-dna-api/src/v1/plan.rs`",
        "`crates/bijux-dna-api/src/v1/run/runtime_support.rs`",
        "`crates/bijux-dna/tests/contracts/cli_library_parity.rs`",
    ] {
        bijux_dna_policies::policy_assert!(
            raw.contains(needle),
            "{} must mention `{needle}`",
            path.display()
        );
    }
}
