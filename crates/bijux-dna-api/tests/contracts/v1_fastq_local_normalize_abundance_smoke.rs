use anyhow::{anyhow, Result};
use std::path::{Path, PathBuf};

struct RepoRootOverrideGuard {
    previous: Option<std::ffi::OsString>,
}

impl RepoRootOverrideGuard {
    fn install(root: &Path) -> Self {
        let previous = std::env::var_os("BIJUX_REPO_ROOT");
        std::env::set_var("BIJUX_REPO_ROOT", root);
        Self { previous }
    }
}

impl Drop for RepoRootOverrideGuard {
    fn drop(&mut self) {
        if let Some(previous) = self.previous.take() {
            std::env::set_var("BIJUX_REPO_ROOT", previous);
        } else {
            std::env::remove_var("BIJUX_REPO_ROOT");
        }
    }
}

fn repo_root() -> Result<PathBuf> {
    crate::support::repo_root()
}

#[test]
fn write_local_normalize_abundance_smoke_report_materializes_governed_outputs() -> Result<()> {
    let repo_root = repo_root()?;
    let _guard = RepoRootOverrideGuard::install(&repo_root);
    let output_dir = repo_root.join("runs/bench/local-smoke/fastq.normalize_abundance");
    if output_dir.exists() {
        std::fs::remove_dir_all(&output_dir)?;
    }

    let primary_artifact =
        bijux_dna_api::v1::api::fastq::write_local_normalize_abundance_smoke_report()?;
    assert_eq!(
        primary_artifact,
        repo_root.join("runs/bench/local-smoke/fastq.normalize_abundance/normalized_abundance.tsv")
    );
    assert!(primary_artifact.is_file(), "top-level normalized abundance table must exist");

    let table = std::fs::read_to_string(&primary_artifact)?;
    assert!(table.starts_with("sample_id\tfeature_id\tnormalized_abundance\n"));
    assert!(table.contains("corpus-03-amplicon-se\totu_001\t0.25000000"));
    assert!(table.contains("corpus-03-amplicon-se\totu_002\t0.75000000"));
    assert!(table.contains("corpus-03-otu-cluster-se\totu_001\t0.50000000"));
    assert!(table.contains("corpus-03-otu-cluster-se\totu_003\t0.50000000"));

    let report_path = output_dir.join("report.json");
    assert!(report_path.is_file(), "local normalize-abundance summary must exist");
    let payload: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&report_path)?)?;
    assert_eq!(payload["stage_id"], serde_json::json!("fastq.normalize_abundance"));
    assert_eq!(payload["sample_id"], serde_json::json!("corpus-03-otu-abundance-table"));
    assert_eq!(payload["planned_tool_id"], serde_json::json!("seqkit"));
    assert_eq!(payload["report_tool_id"], serde_json::json!("bijux"));
    assert_eq!(payload["method"], serde_json::json!("relative_abundance"));
    assert_eq!(payload["normalization_method"], serde_json::json!("relative_abundance"));
    assert_eq!(payload["table_rows"], serde_json::json!(4));
    assert_eq!(payload["sample_count"], serde_json::json!(2));
    assert_eq!(payload["feature_count"], serde_json::json!(3));
    assert_eq!(
        payload["sample_totals"],
        serde_json::json!([["corpus-03-amplicon-se", 1.0], ["corpus-03-otu-cluster-se", 1.0]])
    );
    assert_eq!(payload["numeric_output_valid"], serde_json::json!(true));

    let case_report_path = repo_root.join(
        payload["case_report_json"].as_str().ok_or_else(|| anyhow!("case_report_json missing"))?,
    );
    assert!(case_report_path.is_file(), "governed normalize-abundance case report must exist");
    let case_report: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&case_report_path)?)?;
    assert_eq!(case_report["stage_id"], serde_json::json!("fastq.normalize_abundance"));
    assert_eq!(case_report["tool_id"], serde_json::json!("bijux"));
    assert_eq!(case_report["method"], serde_json::json!("relative_abundance"));
    assert_eq!(case_report["table_rows"], serde_json::json!(4));
    assert_eq!(case_report["sample_count"], serde_json::json!(2));
    assert_eq!(case_report["feature_count"], serde_json::json!(3));
    assert_eq!(
        case_report["per_sample_sums"],
        serde_json::json!([["corpus-03-amplicon-se", 1.0], ["corpus-03-otu-cluster-se", 1.0]])
    );
    assert_eq!(
        case_report["normalized_abundance_tsv"],
        serde_json::json!(
            "runs/bench/local-smoke/fastq.normalize_abundance/corpus-03-otu-abundance-table/seqkit/abundance_normalized.tsv"
        )
    );

    Ok(())
}
