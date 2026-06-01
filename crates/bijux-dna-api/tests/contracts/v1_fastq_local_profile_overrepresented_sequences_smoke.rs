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
fn write_local_profile_overrepresented_sequences_smoke_summary_materializes_governed_outputs(
) -> Result<()> {
    let repo_root = repo_root()?;
    let _guard = RepoRootOverrideGuard::install(&repo_root);
    let output_dir = repo_root.join("target/local-smoke/fastq.profile_overrepresented_sequences");
    if output_dir.exists() {
        std::fs::remove_dir_all(&output_dir)?;
    }

    let summary_path =
        bijux_dna_api::v1::api::fastq::write_local_profile_overrepresented_sequences_smoke_summary(
        )?;
    assert_eq!(
        summary_path,
        repo_root.join("target/local-smoke/fastq.profile_overrepresented_sequences/overrepresented.tsv")
    );
    assert!(summary_path.is_file(), "top-level overrepresented TSV must exist");

    let tsv = std::fs::read_to_string(&summary_path)?;
    assert!(tsv.contains("ACGTACGTACGT\t3\t0.600000\toverrepresented"));

    let report_path = output_dir.join("report.json");
    assert!(report_path.is_file(), "top-level summary report must exist");
    let payload: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&report_path)?)?;
    assert_eq!(
        payload["stage_id"],
        serde_json::json!("fastq.profile_overrepresented_sequences")
    );
    assert_eq!(payload["sample_id"], serde_json::json!("known-repeat-se"));
    assert_eq!(payload["planned_tool_id"], serde_json::json!("seqkit"));
    assert_eq!(payload["report_tool_id"], serde_json::json!("bijux"));
    assert_eq!(payload["top_sequence"], serde_json::json!("ACGTACGTACGT"));
    assert_eq!(payload["top_count"], serde_json::json!(3));
    assert_eq!(payload["top_fraction"], serde_json::json!(0.6));
    assert_eq!(payload["flagged_sequences"], serde_json::json!(1));

    let case_report_path = repo_root.join(
        payload["case_report_json"]
            .as_str()
            .ok_or_else(|| anyhow!("case_report_json missing"))?,
    );
    assert!(case_report_path.is_file(), "case report JSON must exist");
    let case_report: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&case_report_path)?)?;
    assert_eq!(case_report["tool_id"], serde_json::json!("bijux"));
    assert_eq!(case_report["flagged_sequences"], serde_json::json!(1));
    assert_eq!(case_report["top_fraction"], serde_json::json!(0.6));

    Ok(())
}
