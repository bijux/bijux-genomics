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
fn write_local_infer_asvs_smoke_report_materializes_governed_outputs() -> Result<()> {
    let repo_root = repo_root()?;
    let _guard = RepoRootOverrideGuard::install(&repo_root);
    let output_dir = repo_root.join("runs/bench/local-smoke/fastq.infer_asvs");
    if output_dir.exists() {
        std::fs::remove_dir_all(&output_dir)?;
    }

    let primary_artifact = bijux_dna_api::v1::api::fastq::write_local_infer_asvs_smoke_report()?;
    assert_eq!(
        primary_artifact,
        repo_root.join("runs/bench/local-smoke/fastq.infer_asvs/asv_table.tsv")
    );
    assert!(primary_artifact.is_file(), "top-level ASV table must exist");

    let table = std::fs::read_to_string(&primary_artifact)?;
    assert!(table.contains("sample_id\tfeature_id\tabundance"));
    assert!(table.contains("sample_1\tASV00001\t1"));
    assert!(table.contains("sample_1\tASV00002\t1"));
    assert!(table.contains("sample_1\tASV00003\t1"));

    let representatives = output_dir.join("representatives.fasta");
    assert!(representatives.is_file(), "top-level representative FASTA must exist");
    let fasta = std::fs::read_to_string(&representatives)?;
    assert!(fasta.contains(">ASV00001"));
    assert!(fasta.contains(">ASV00002"));
    assert!(fasta.contains(">ASV00003"));

    let report_path = output_dir.join("report.json");
    assert!(report_path.is_file(), "local infer-asvs summary must exist");
    let payload: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&report_path)?)?;
    assert_eq!(payload["stage_id"], serde_json::json!("fastq.infer_asvs"));
    assert_eq!(payload["sample_id"], serde_json::json!("corpus-03-amplicon-se"));
    assert_eq!(payload["planned_tool_id"], serde_json::json!("dada2"));
    assert_eq!(payload["report_tool_id"], serde_json::json!("bijux"));
    assert_eq!(payload["asv_count"], serde_json::json!(3));
    assert_eq!(payload["sample_count"], serde_json::json!(1));
    assert_eq!(payload["representative_sequence_count"], serde_json::json!(3));
    let payload_asv_table = repo_root
        .join(payload["asv_table_tsv"].as_str().ok_or_else(|| anyhow!("asv_table_tsv missing"))?);
    let payload_representatives = repo_root.join(
        payload["representatives_fasta"]
            .as_str()
            .ok_or_else(|| anyhow!("representatives_fasta missing"))?,
    );
    let taxonomy_ready_fasta = repo_root.join(
        payload["taxonomy_ready_fasta"]
            .as_str()
            .ok_or_else(|| anyhow!("taxonomy_ready_fasta missing"))?,
    );
    let taxonomy_ready_reads_fastq = repo_root.join(
        payload["taxonomy_ready_fastq"]
            .as_str()
            .ok_or_else(|| anyhow!("taxonomy_ready_fastq missing"))?,
    );
    let raw_backend_report = repo_root.join(
        payload["raw_backend_report"]
            .as_str()
            .ok_or_else(|| anyhow!("raw_backend_report missing"))?,
    );
    assert!(payload_asv_table.is_file(), "summary must point at the copied top-level ASV table");
    assert!(
        payload_representatives.is_file(),
        "summary must point at the copied top-level representative FASTA"
    );
    assert!(taxonomy_ready_fasta.is_file(), "summary must point at the taxonomy-ready FASTA");
    assert!(
        taxonomy_ready_reads_fastq.is_file(),
        "summary must point at the taxonomy-ready FASTQ"
    );
    assert!(raw_backend_report.is_file(), "summary must point at the governed backend report");

    let case_report_path = repo_root.join(
        payload["case_report_json"].as_str().ok_or_else(|| anyhow!("case_report_json missing"))?,
    );
    assert!(case_report_path.is_file(), "governed infer-asvs case report must exist");
    let case_report: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&case_report_path)?)?;
    assert_eq!(case_report["stage_id"], serde_json::json!("fastq.infer_asvs"));
    assert_eq!(case_report["tool_id"], serde_json::json!("bijux"));
    assert_eq!(case_report["asv_count"], serde_json::json!(3));
    assert_eq!(case_report["sample_count"], serde_json::json!(1));
    assert_eq!(case_report["representative_sequence_count"], serde_json::json!(3));

    Ok(())
}
