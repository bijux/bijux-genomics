use anyhow::Result;
use flate2::read::MultiGzDecoder;
use std::io::{BufRead, BufReader};
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

fn read_gz_fastq_sequences(path: &Path) -> Result<Vec<String>> {
    let file = std::fs::File::open(path)?;
    let reader = BufReader::new(MultiGzDecoder::new(file));
    let mut lines = reader.lines();
    let mut sequences = Vec::new();
    while lines.next().transpose()?.is_some() {
        let sequence = lines
            .next()
            .transpose()?
            .unwrap_or_else(|| panic!("sequence line missing in {}", path.display()));
        let _plus = lines
            .next()
            .transpose()?
            .unwrap_or_else(|| panic!("plus line missing in {}", path.display()));
        let _quality = lines
            .next()
            .transpose()?
            .unwrap_or_else(|| panic!("quality line missing in {}", path.display()));
        sequences.push(sequence);
    }
    Ok(sequences)
}

#[test]
fn write_local_remove_duplicates_smoke_report_materializes_governed_outputs() -> Result<()> {
    let repo_root = repo_root()?;
    let _guard = RepoRootOverrideGuard::install(&repo_root);
    let output_dir = repo_root.join("target/local-smoke/fastq.remove_duplicates");
    if output_dir.exists() {
        std::fs::remove_dir_all(&output_dir)?;
    }

    let report_path = bijux_dna_api::v1::api::fastq::write_local_remove_duplicates_smoke_report()?;
    assert_eq!(
        report_path,
        repo_root.join("target/local-smoke/fastq.remove_duplicates/report.json")
    );
    assert!(report_path.is_file(), "local remove-duplicates summary must exist");

    let payload: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&report_path)?)?;
    assert_eq!(payload["stage_id"], serde_json::json!("fastq.remove_duplicates"));
    assert_eq!(payload["sample_id"], serde_json::json!("duplicate-hit-se"));
    assert_eq!(payload["planned_tool_id"], serde_json::json!("clumpify"));
    assert_eq!(payload["report_tool_id"], serde_json::json!("bijux"));
    assert_eq!(payload["dedup_mode"], serde_json::json!("exact"));
    assert_eq!(payload["keep_order"], serde_json::json!(true));
    assert_eq!(payload["input_reads"], serde_json::json!(4));
    assert_eq!(payload["duplicate_reads"], serde_json::json!(1));
    assert_eq!(payload["unique_reads"], serde_json::json!(3));
    assert_eq!(payload["output_reads"], serde_json::json!(3));

    let dedup_fastq = repo_root.join(
        payload["dedup_fastq_gz"]
            .as_str()
            .unwrap_or_else(|| panic!("dedup_fastq_gz missing")),
    );
    assert!(dedup_fastq.is_file(), "top-level dedup FASTQ must exist");
    assert_eq!(
        read_gz_fastq_sequences(&dedup_fastq)?,
        vec![
            "ACGTACGT".to_string(),
            "GGGGTTTT".to_string(),
            "TTCCAAGG".to_string(),
        ]
    );

    let case_report_path = repo_root.join(
        payload["case_report_json"]
            .as_str()
            .unwrap_or_else(|| panic!("case_report_json missing")),
    );
    assert!(case_report_path.is_file(), "per-case dedup report must exist");
    let case_report: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&case_report_path)?)?;
    assert_eq!(case_report["stage_id"], serde_json::json!("fastq.remove_duplicates"));
    assert_eq!(case_report["tool_id"], serde_json::json!("bijux"));
    assert_eq!(case_report["reads_in"], serde_json::json!(4));
    assert_eq!(case_report["reads_out"], serde_json::json!(3));
    assert_eq!(case_report["duplicates_removed"], serde_json::json!(1));
    assert_eq!(case_report["dedup_mode"], serde_json::json!("exact"));
    assert_eq!(case_report["keep_order"], serde_json::json!(true));

    for key in ["duplicate_classes_tsv", "duplicate_provenance_json", "raw_backend_report"] {
        let path = repo_root.join(
            payload[key]
                .as_str()
                .unwrap_or_else(|| panic!("{key} missing from local remove-duplicates summary")),
        );
        assert!(path.is_file(), "{key} must exist");
    }

    Ok(())
}
