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
fn write_local_filter_reads_smoke_report_materializes_governed_outputs() -> Result<()> {
    let repo_root = repo_root()?;
    let _guard = RepoRootOverrideGuard::install(&repo_root);
    let output_dir = repo_root.join("runs/bench/local-smoke/fastq.filter_reads");
    if output_dir.exists() {
        std::fs::remove_dir_all(&output_dir)?;
    }

    let report_path = bijux_dna_api::v1::api::fastq::write_local_filter_reads_smoke_report()?;
    assert_eq!(
        report_path,
        repo_root.join("runs/bench/local-smoke/fastq.filter_reads/report.json")
    );
    assert!(report_path.is_file(), "local filter report must exist");

    let payload: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&report_path)?)?;
    assert_eq!(payload["stage_id"], serde_json::json!("fastq.filter_reads"));
    assert_eq!(payload["sample_id"], serde_json::json!("n-and-complexity-se"));
    assert_eq!(payload["tool_id"], serde_json::json!("fastp"));
    assert_eq!(payload["input_reads"], serde_json::json!(3));
    assert_eq!(payload["output_reads"], serde_json::json!(1));
    assert_eq!(payload["reads_dropped"], serde_json::json!(2));
    assert_eq!(payload["reads_removed_by_n"], serde_json::json!(1));
    assert_eq!(payload["reads_removed_low_complexity"], serde_json::json!(1));
    assert_eq!(payload["reads_removed_by_entropy"], serde_json::json!(0));
    assert_eq!(payload["reads_removed_by_kmer"], serde_json::json!(0));
    assert_eq!(payload["reads_removed_by_length"], serde_json::json!(0));

    let filtered_fastq = repo_root.join(
        payload["filtered_fastq_gz"]
            .as_str()
            .unwrap_or_else(|| panic!("filtered_fastq_gz missing")),
    );
    assert!(filtered_fastq.is_file(), "top-level filtered FASTQ must exist");
    assert_eq!(read_gz_fastq_sequences(&filtered_fastq)?, vec!["ACGTACGT".to_string()]);

    let stage_report_path = repo_root
        .join(payload["report_json"].as_str().unwrap_or_else(|| panic!("report_json missing")));
    assert!(stage_report_path.is_file(), "per-case filter report must exist");
    let stage_report: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&stage_report_path)?)?;
    assert_eq!(stage_report["stage_id"], serde_json::json!("fastq.filter_reads"));
    assert_eq!(stage_report["tool_id"], serde_json::json!("fastp"));
    assert_eq!(stage_report["reads_in"], serde_json::json!(3));
    assert_eq!(stage_report["reads_out"], serde_json::json!(1));
    assert_eq!(stage_report["reads_dropped"], serde_json::json!(2));
    assert_eq!(stage_report["reads_removed_by_n"], serde_json::json!(1));
    assert_eq!(stage_report["reads_removed_low_complexity"], serde_json::json!(1));
    assert_eq!(stage_report["reads_removed_by_entropy"], serde_json::json!(0));
    assert_eq!(stage_report["raw_backend_report_format"], serde_json::json!("fastp_json"));

    let raw_backend_report = repo_root.join(
        payload["raw_backend_report"]
            .as_str()
            .unwrap_or_else(|| panic!("raw_backend_report missing")),
    );
    assert!(raw_backend_report.is_file(), "raw backend report must exist");

    Ok(())
}
