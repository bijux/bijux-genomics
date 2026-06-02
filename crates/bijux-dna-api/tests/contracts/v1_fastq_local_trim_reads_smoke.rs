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
fn write_local_trim_reads_smoke_report_materializes_governed_outputs() -> Result<()> {
    let repo_root = repo_root()?;
    let _guard = RepoRootOverrideGuard::install(&repo_root);
    let output_dir = repo_root.join("target/local-smoke/fastq.trim_reads");
    if output_dir.exists() {
        std::fs::remove_dir_all(&output_dir)?;
    }

    let report_path = bijux_dna_api::v1::api::fastq::write_local_trim_reads_smoke_report()?;
    assert_eq!(report_path, repo_root.join("target/local-smoke/fastq.trim_reads/report.json"));
    assert!(report_path.is_file(), "local trim-reads report must exist");

    let payload: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&report_path)?)?;
    assert_eq!(payload["stage_id"], serde_json::json!("fastq.trim_reads"));
    assert_eq!(payload["case_count"], serde_json::json!(2));
    assert_eq!(payload["all_cases_passed"], serde_json::json!(true));

    let cases = payload["cases"].as_array().unwrap_or_else(|| panic!("cases array missing"));
    let se_case = cases
        .iter()
        .find(|case| case["sample_id"] == serde_json::json!("adapter-quality-se"))
        .unwrap_or_else(|| panic!("single-end trim smoke case missing"));
    assert_eq!(se_case["layout"], serde_json::json!("single_end"));
    assert_eq!(se_case["input_read_count_total"], serde_json::json!(2));
    assert_eq!(se_case["output_read_count_total"], serde_json::json!(2));
    assert_eq!(se_case["reads_retained"], serde_json::json!(2));
    assert_eq!(se_case["reads_dropped"], serde_json::json!(0));
    assert_eq!(se_case["read_count_not_greater_than_input"], serde_json::json!(true));
    assert_eq!(se_case["min_length"], serde_json::json!(4));
    assert_eq!(se_case["quality_cutoff"], serde_json::json!(20));
    assert_eq!(se_case["bases_removed"], serde_json::json!(17));

    let se_trimmed = repo_root.join(
        se_case["trimmed_reads_r1"]
            .as_str()
            .unwrap_or_else(|| panic!("trimmed_reads_r1 missing for se case")),
    );
    assert!(se_trimmed.is_file(), "single-end trimmed FASTQ must exist");
    assert_eq!(
        read_gz_fastq_sequences(&se_trimmed)?,
        vec!["ACGTACGT".to_string(), "TTTT".to_string()]
    );

    let se_report_json = repo_root.join(
        se_case["report_json"]
            .as_str()
            .unwrap_or_else(|| panic!("report_json missing for se case")),
    );
    assert!(se_report_json.is_file(), "single-end trim report must exist");
    let se_report: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&se_report_json)?)?;
    assert_eq!(se_report["stage_id"], serde_json::json!("fastq.trim_reads"));
    assert_eq!(se_report["tool_id"], serde_json::json!("fastp"));
    assert_eq!(se_report["reads_in"], serde_json::json!(2));
    assert_eq!(se_report["reads_out"], serde_json::json!(2));
    assert_eq!(se_report["min_length"], serde_json::json!(4));
    assert_eq!(se_report["quality_cutoff"], serde_json::json!(20));
    assert_eq!(se_report["raw_backend_report_format"], serde_json::json!("fastp_trim_report"));

    let se_raw_backend_report = repo_root.join(
        se_case["raw_backend_report"]
            .as_str()
            .unwrap_or_else(|| panic!("raw_backend_report missing for se case")),
    );
    assert!(se_raw_backend_report.is_file(), "single-end raw backend report must exist");

    let pe_case = cases
        .iter()
        .find(|case| case["sample_id"] == serde_json::json!("adapter-quality-pe"))
        .unwrap_or_else(|| panic!("paired-end trim smoke case missing"));
    assert_eq!(pe_case["layout"], serde_json::json!("paired_end"));
    assert_eq!(pe_case["input_read_count_total"], serde_json::json!(4));
    assert_eq!(pe_case["output_read_count_total"], serde_json::json!(4));
    assert_eq!(pe_case["input_pair_count"], serde_json::json!(2));
    assert_eq!(pe_case["output_pair_count"], serde_json::json!(2));
    assert_eq!(pe_case["reads_retained"], serde_json::json!(4));
    assert_eq!(pe_case["reads_dropped"], serde_json::json!(0));
    assert_eq!(pe_case["read_count_not_greater_than_input"], serde_json::json!(true));
    assert_eq!(pe_case["bases_removed"], serde_json::json!(17));

    let pe_trimmed_r1 = repo_root.join(
        pe_case["trimmed_reads_r1"]
            .as_str()
            .unwrap_or_else(|| panic!("trimmed_reads_r1 missing for pe case")),
    );
    let pe_trimmed_r2 = repo_root.join(
        pe_case["trimmed_reads_r2"]
            .as_str()
            .unwrap_or_else(|| panic!("trimmed_reads_r2 missing for pe case")),
    );
    assert!(pe_trimmed_r1.is_file(), "paired-end trimmed R1 FASTQ must exist");
    assert!(pe_trimmed_r2.is_file(), "paired-end trimmed R2 FASTQ must exist");
    assert_eq!(
        read_gz_fastq_sequences(&pe_trimmed_r1)?,
        vec!["ACGT".to_string(), "GGGG".to_string()]
    );
    assert_eq!(
        read_gz_fastq_sequences(&pe_trimmed_r2)?,
        vec!["TTTTCCCC".to_string(), "AAAACCCC".to_string()]
    );

    let pe_report_json = repo_root.join(
        pe_case["report_json"]
            .as_str()
            .unwrap_or_else(|| panic!("report_json missing for pe case")),
    );
    assert!(pe_report_json.is_file(), "paired-end trim report must exist");
    let pe_report: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&pe_report_json)?)?;
    assert_eq!(pe_report["reads_in"], serde_json::json!(4));
    assert_eq!(pe_report["reads_out"], serde_json::json!(4));
    assert_eq!(pe_report["pairs_in"], serde_json::json!(2));
    assert_eq!(pe_report["pairs_out"], serde_json::json!(2));
    assert_eq!(pe_report["min_length"], serde_json::json!(4));
    assert_eq!(pe_report["quality_cutoff"], serde_json::json!(20));

    Ok(())
}
