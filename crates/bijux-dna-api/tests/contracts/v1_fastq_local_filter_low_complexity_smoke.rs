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
fn write_local_filter_low_complexity_smoke_report_materializes_governed_outputs() -> Result<()> {
    let repo_root = repo_root()?;
    let _guard = RepoRootOverrideGuard::install(&repo_root);
    let output_dir = repo_root.join("target/local-smoke/fastq.filter_low_complexity");
    if output_dir.exists() {
        std::fs::remove_dir_all(&output_dir)?;
    }

    let report_path =
        bijux_dna_api::v1::api::fastq::write_local_filter_low_complexity_smoke_report()?;
    assert_eq!(
        report_path,
        repo_root.join("target/local-smoke/fastq.filter_low_complexity/report.json")
    );
    assert!(report_path.is_file(), "local filter-low-complexity summary must exist");

    let payload: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&report_path)?)?;
    assert_eq!(payload["stage_id"], serde_json::json!("fastq.filter_low_complexity"));
    assert_eq!(payload["sample_id"], serde_json::json!("low-complexity-se"));
    assert_eq!(payload["planned_tool_id"], serde_json::json!("bbduk"));
    assert_eq!(payload["report_tool_id"], serde_json::json!("bijux"));
    assert_eq!(payload["entropy_threshold"], serde_json::json!(0.6));
    assert_eq!(payload["polyx_threshold"], serde_json::json!(8));
    assert_eq!(payload["input_reads"], serde_json::json!(3));
    assert_eq!(payload["reads_removed_low_complexity"], serde_json::json!(2));
    assert_eq!(payload["output_reads"], serde_json::json!(1));

    let filtered_fastq = repo_root.join(
        payload["filtered_fastq_gz"]
            .as_str()
            .unwrap_or_else(|| panic!("filtered_fastq_gz missing")),
    );
    assert!(filtered_fastq.is_file(), "top-level filtered FASTQ must exist");
    assert_eq!(read_gz_fastq_sequences(&filtered_fastq)?, vec!["ACGTTGCAAGTC".to_string()]);

    let case_report_path = repo_root.join(
        payload["case_report_json"].as_str().unwrap_or_else(|| panic!("case_report_json missing")),
    );
    assert!(case_report_path.is_file(), "per-case low-complexity report must exist");
    let case_report: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&case_report_path)?)?;
    assert_eq!(case_report["stage_id"], serde_json::json!("fastq.filter_low_complexity"));
    assert_eq!(case_report["tool_id"], serde_json::json!("bijux"));
    assert_eq!(case_report["reads_in"], serde_json::json!(3));
    assert_eq!(case_report["reads_out"], serde_json::json!(1));
    assert_eq!(case_report["reads_removed_low_complexity"], serde_json::json!(2));
    assert_eq!(payload["input_reads"], case_report["reads_in"]);
    assert_eq!(
        payload["reads_removed_low_complexity"],
        case_report["reads_removed_low_complexity"]
    );
    assert_eq!(payload["output_reads"], case_report["reads_out"]);
    assert_eq!(case_report["entropy_threshold"], serde_json::json!(0.6));
    assert_eq!(case_report["polyx_threshold"], serde_json::json!(8));
    assert_eq!(
        case_report["raw_backend_report_format"],
        serde_json::json!("bijux_filter_low_complexity_trace")
    );
    let case_filtered_fastq = repo_root
        .join(case_report["output_r1"].as_str().unwrap_or_else(|| panic!("output_r1 missing")));
    assert!(case_filtered_fastq.is_file(), "case-level filtered FASTQ must exist");
    assert_eq!(
        read_gz_fastq_sequences(&filtered_fastq)?,
        read_gz_fastq_sequences(&case_filtered_fastq)?
    );

    let raw_backend_report = repo_root.join(
        payload["raw_backend_report"]
            .as_str()
            .unwrap_or_else(|| panic!("raw_backend_report missing")),
    );
    assert!(raw_backend_report.is_file(), "raw backend report must exist");

    Ok(())
}
