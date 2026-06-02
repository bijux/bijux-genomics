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
fn write_local_cluster_otus_smoke_report_materializes_governed_outputs() -> Result<()> {
    let repo_root = repo_root()?;
    let _guard = RepoRootOverrideGuard::install(&repo_root);
    let output_dir = repo_root.join("target/local-smoke/fastq.cluster_otus");
    if output_dir.exists() {
        std::fs::remove_dir_all(&output_dir)?;
    }

    let primary_artifact = bijux_dna_api::v1::api::fastq::write_local_cluster_otus_smoke_report()?;
    assert_eq!(
        primary_artifact,
        repo_root.join("target/local-smoke/fastq.cluster_otus/otu_table.tsv")
    );
    assert!(primary_artifact.is_file(), "top-level OTU table must exist");

    let table = std::fs::read_to_string(&primary_artifact)?;
    assert!(table.contains("sample_id\totu_id\tabundance\trepresentative_id\trepresentative_fasta"));
    assert!(table.contains("sample_1\tOTU00001\t1\tOTU00001\ttarget/local-smoke/fastq.cluster_otus/otu_representatives.fasta"));
    assert!(table.contains("sample_1\tOTU00002\t1\tOTU00002\ttarget/local-smoke/fastq.cluster_otus/otu_representatives.fasta"));
    assert!(table.contains("sample_1\tOTU00003\t1\tOTU00003\ttarget/local-smoke/fastq.cluster_otus/otu_representatives.fasta"));

    let representatives = output_dir.join("otu_representatives.fasta");
    assert!(representatives.is_file(), "top-level OTU representative FASTA must exist");
    let fasta = std::fs::read_to_string(&representatives)?;
    assert!(fasta.contains(">OTU00001"));
    assert!(fasta.contains(">OTU00002"));
    assert!(fasta.contains(">OTU00003"));

    let report_path = output_dir.join("report.json");
    assert!(report_path.is_file(), "local cluster-otus summary must exist");
    let payload: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&report_path)?)?;
    assert_eq!(payload["stage_id"], serde_json::json!("fastq.cluster_otus"));
    assert_eq!(payload["sample_id"], serde_json::json!("corpus-03-otu-cluster-se"));
    assert_eq!(payload["planned_tool_id"], serde_json::json!("vsearch"));
    assert_eq!(payload["report_tool_id"], serde_json::json!("bijux"));
    assert_eq!(payload["clustering_threshold"], serde_json::json!(0.97));
    assert_eq!(payload["otu_count"], serde_json::json!(3));
    assert_eq!(payload["sample_count"], serde_json::json!(1));
    assert_eq!(payload["representative_sequence_count"], serde_json::json!(3));
    assert_eq!(
        payload["otu_table_tsv"],
        serde_json::json!("target/local-smoke/fastq.cluster_otus/otu_table.tsv")
    );
    assert_eq!(
        payload["representative_sequences_fasta"],
        serde_json::json!("target/local-smoke/fastq.cluster_otus/otu_representatives.fasta")
    );

    let case_report_path = repo_root.join(
        payload["case_report_json"].as_str().ok_or_else(|| anyhow!("case_report_json missing"))?,
    );
    assert!(case_report_path.is_file(), "governed cluster-otus case report must exist");
    let case_report: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&case_report_path)?)?;
    assert_eq!(case_report["stage_id"], serde_json::json!("fastq.cluster_otus"));
    assert_eq!(case_report["tool_id"], serde_json::json!("bijux"));
    assert_eq!(case_report["otu_identity"], serde_json::json!(0.97));
    assert_eq!(case_report["otu_count"], serde_json::json!(3));
    assert_eq!(case_report["sample_count"], serde_json::json!(1));
    assert_eq!(case_report["representative_sequence_count"], serde_json::json!(3));

    let raw_backend_report = repo_root.join(
        payload["raw_backend_report"]
            .as_str()
            .ok_or_else(|| anyhow!("raw_backend_report missing"))?,
    );
    assert!(raw_backend_report.is_file(), "governed raw backend OTU report must exist");

    Ok(())
}
