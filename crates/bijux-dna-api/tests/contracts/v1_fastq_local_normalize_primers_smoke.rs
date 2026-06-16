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
    let output_dir = repo_root.join("runs/bench/local-smoke/fastq.normalize_primers");
    if output_dir.exists() {
        std::fs::remove_dir_all(&output_dir)?;
    }

    let report_path = bijux_dna_api::v1::api::fastq::write_local_normalize_primers_smoke_report()?;
    assert_eq!(
        report_path,
        repo_root.join("runs/bench/local-smoke/fastq.normalize_primers/report.json")
    );
    assert!(report_path.is_file(), "local normalize-primers report must exist");

    let payload: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&report_path)?)?;
    assert_eq!(payload["stage_id"], serde_json::json!("fastq.normalize_primers"));
    assert_eq!(payload["case_count"], serde_json::json!(2));

    let cases = payload["cases"].as_array().unwrap_or_else(|| panic!("cases array missing"));
    let se_case = cases
        .iter()
        .find(|case| case["sample_id"] == serde_json::json!("amplicon-16s-se"))
        .unwrap_or_else(|| panic!("single-end normalize-primers case missing"));
    assert_eq!(se_case["tool_id"], serde_json::json!("cutadapt"));
    assert_eq!(se_case["primer_set_id"], serde_json::json!("16S_universal_v1"));
    assert_eq!(se_case["marker_id"], serde_json::json!("16S"));
    assert_eq!(se_case["input_reads"], serde_json::json!(3));
    assert_eq!(se_case["matched_reads"], serde_json::json!(2));
    assert_eq!(se_case["unmatched_reads"], serde_json::json!(1));
    assert_eq!(se_case["output_reads"], serde_json::json!(3));
    assert_eq!(se_case["used_fallback"], serde_json::json!(true));

    let normalized_fastq = repo_root.join(
        se_case["normalized_reads_r1"]
            .as_str()
            .unwrap_or_else(|| panic!("normalized_reads_r1 missing")),
    );
    assert!(normalized_fastq.is_file(), "normalized R1 FASTQ must exist");
    let sequences = read_gz_fastq_sequences(&normalized_fastq)?;
    assert_eq!(
        sequences,
        vec!["ACGTACGT".to_string(), "ACGTACGT".to_string(), "TTTTACGTACGT".to_string(),]
    );

    let governed_report = repo_root
        .join(se_case["report_json"].as_str().unwrap_or_else(|| panic!("report_json missing")));
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
        se_case["primer_orientation_report"]
            .as_str()
            .unwrap_or_else(|| panic!("primer_orientation_report missing")),
    );
    let primer_stats_json = repo_root.join(
        se_case["primer_stats_json"]
            .as_str()
            .unwrap_or_else(|| panic!("primer_stats_json missing")),
    );
    assert!(primer_orientation_report.is_file(), "primer orientation report must exist");
    assert!(primer_stats_json.is_file(), "primer stats JSON must exist");

    let paired_case = cases
        .iter()
        .find(|case| case["sample_id"] == serde_json::json!("amplicon-16s-pe"))
        .unwrap_or_else(|| panic!("paired-end normalize-primers case missing"));
    assert_eq!(paired_case["layout"], serde_json::json!("paired_end"));
    assert_eq!(paired_case["input_reads"], serde_json::json!(4));
    assert_eq!(paired_case["matched_reads"], serde_json::json!(2));
    assert_eq!(paired_case["unmatched_reads"], serde_json::json!(2));
    assert_eq!(paired_case["output_reads"], serde_json::json!(4));
    let paired_r2 = repo_root.join(
        paired_case["normalized_reads_r2"]
            .as_str()
            .unwrap_or_else(|| panic!("paired normalized_reads_r2 missing")),
    );
    assert!(paired_r2.is_file(), "paired normalized R2 FASTQ must exist");
    let paired_r2_sequences = read_gz_fastq_sequences(&paired_r2)?;
    assert_eq!(paired_r2_sequences, vec!["GGTT".to_string(), "CCCCAAAA".to_string()]);

    Ok(())
}
