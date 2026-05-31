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
fn write_local_normalize_primers_smoke_report_materializes_governed_outputs() -> Result<()> {
    let repo_root = repo_root()?;
    let _guard = RepoRootOverrideGuard::install(&repo_root);
    let output_dir = repo_root.join("target/local-smoke/fastq.normalize_primers");
    if output_dir.exists() {
        std::fs::remove_dir_all(&output_dir)?;
    }

    let report_path = bijux_dna_api::v1::api::fastq::write_local_normalize_primers_smoke_report()?;
    assert_eq!(
        report_path,
        repo_root.join("target/local-smoke/fastq.normalize_primers/report.json")
    );
    assert!(report_path.is_file(), "local normalize-primers report must exist");

    let payload: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&report_path)?)?;
    assert_eq!(payload["stage_id"], serde_json::json!("fastq.normalize_primers"));
    assert_eq!(payload["sample_id"], serde_json::json!("amplicon-16s-se"));
    assert_eq!(payload["tool_id"], serde_json::json!("cutadapt"));
    assert_eq!(payload["primer_set_id"], serde_json::json!("16S_universal_v1"));
    assert_eq!(payload["marker_id"], serde_json::json!("16S"));
    assert_eq!(payload["input_reads"], serde_json::json!(3));
    assert_eq!(payload["matched_reads"], serde_json::json!(2));
    assert_eq!(payload["unmatched_reads"], serde_json::json!(1));
    assert_eq!(payload["output_reads"], serde_json::json!(3));
    assert_eq!(payload["used_fallback"], serde_json::json!(true));

    let normalized_fastq = repo_root.join(
        payload["normalized_fastq_gz"]
            .as_str()
            .unwrap_or_else(|| panic!("normalized_fastq_gz missing")),
    );
    assert!(normalized_fastq.is_file(), "top-level normalized FASTQ must exist");
    let sequences = read_gz_fastq_sequences(&normalized_fastq)?;
    assert_eq!(
        sequences,
        vec!["ACGTACGT".to_string(), "ACGTACGT".to_string(), "TTTTACGTACGT".to_string(),]
    );

    let governed_report = repo_root
        .join(payload["report_json"].as_str().unwrap_or_else(|| panic!("report_json missing")));
    assert!(governed_report.is_file(), "governed normalize-primers report must exist");
    let report: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&governed_report)?)?;
    assert_eq!(report["stage_id"], serde_json::json!("fastq.normalize_primers"));
    assert_eq!(report["tool_id"], serde_json::json!("cutadapt"));
    assert_eq!(report["primer_set_id"], serde_json::json!("16S_universal_v1"));
    assert_eq!(report["reads_out"], serde_json::json!(3));
    assert_eq!(report["primer_trimmed_reads"], serde_json::json!(2));
    assert_eq!(report["used_fallback"], serde_json::json!(true));

    let primer_orientation_report = repo_root.join(
        payload["primer_orientation_report"]
            .as_str()
            .unwrap_or_else(|| panic!("primer_orientation_report missing")),
    );
    let primer_stats_json = repo_root.join(
        payload["primer_stats_json"]
            .as_str()
            .unwrap_or_else(|| panic!("primer_stats_json missing")),
    );
    assert!(primer_orientation_report.is_file(), "primer orientation report must exist");
    assert!(primer_stats_json.is_file(), "primer stats JSON must exist");

    Ok(())
}
