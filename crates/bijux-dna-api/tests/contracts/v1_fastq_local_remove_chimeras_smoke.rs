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
fn write_local_remove_chimeras_smoke_report_materializes_governed_outputs() -> Result<()> {
    let repo_root = repo_root()?;
    let _guard = RepoRootOverrideGuard::install(&repo_root);
    let output_dir = repo_root.join("target/local-smoke/fastq.remove_chimeras");
    if output_dir.exists() {
        std::fs::remove_dir_all(&output_dir)?;
    }

    let primary_artifact =
        bijux_dna_api::v1::api::fastq::write_local_remove_chimeras_smoke_report()?;
    assert_eq!(
        primary_artifact,
        repo_root.join("target/local-smoke/fastq.remove_chimeras/non_chimeric.fasta")
    );
    assert!(primary_artifact.is_file(), "top-level non-chimeric FASTA must exist");

    let fasta = std::fs::read_to_string(&primary_artifact)?;
    assert!(fasta.contains(">amplicon_consensus_1"));
    assert!(fasta.contains("ACGTTGCAACGTTGCA"));
    assert!(fasta.contains(">amplicon_consensus_2"));
    assert!(fasta.contains("TTGCAACGTTTGCAAC"));
    assert!(!fasta.contains("amplicon_chimera_1"));

    let report_path = output_dir.join("report.json");
    assert!(report_path.is_file(), "local remove-chimeras summary must exist");
    let payload: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&report_path)?)?;
    assert_eq!(payload["stage_id"], serde_json::json!("fastq.remove_chimeras"));
    assert_eq!(payload["sample_id"], serde_json::json!("chimera-control-se"));
    assert_eq!(payload["planned_tool_id"], serde_json::json!("vsearch"));
    assert_eq!(payload["report_tool_id"], serde_json::json!("bijux"));
    assert_eq!(payload["checked_sequence_count"], serde_json::json!(3));
    assert_eq!(payload["chimera_count"], serde_json::json!(1));
    assert_eq!(payload["non_chimera_count"], serde_json::json!(2));

    let chimera_tsv = repo_root
        .join(payload["chimeras_tsv"].as_str().ok_or_else(|| anyhow!("chimeras_tsv missing"))?);
    assert!(chimera_tsv.is_file(), "top-level chimera TSV must exist");
    let chimera_rows = std::fs::read_to_string(&chimera_tsv)?;
    assert!(chimera_rows.contains("record_id\tchimera"));
    assert!(chimera_rows.contains("amplicon_chimera_1\tyes"));

    let case_report_path = repo_root.join(
        payload["case_report_json"].as_str().ok_or_else(|| anyhow!("case_report_json missing"))?,
    );
    assert!(case_report_path.is_file(), "governed chimera case report must exist");
    let case_report: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&case_report_path)?)?;
    assert_eq!(case_report["stage_id"], serde_json::json!("fastq.remove_chimeras"));
    assert_eq!(case_report["tool_id"], serde_json::json!("bijux"));
    assert_eq!(case_report["reads_in"], serde_json::json!(3));
    assert_eq!(case_report["reads_out"], serde_json::json!(2));
    assert_eq!(case_report["chimeras_removed"], serde_json::json!(1));

    Ok(())
}
