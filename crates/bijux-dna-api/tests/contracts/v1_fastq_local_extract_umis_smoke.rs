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

fn read_gz_fastq_headers(path: &Path) -> Result<Vec<String>> {
    let file = std::fs::File::open(path)?;
    let reader = BufReader::new(MultiGzDecoder::new(file));
    let mut lines = reader.lines();
    let mut headers = Vec::new();
    while let Some(header) = lines.next().transpose()? {
        let _sequence = lines
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
        headers.push(header);
    }
    Ok(headers)
}

#[test]
fn write_local_extract_umis_smoke_report_materializes_governed_outputs() -> Result<()> {
    let repo_root = repo_root()?;
    let _guard = RepoRootOverrideGuard::install(&repo_root);
    let output_dir = repo_root.join("target/local-smoke/fastq.extract_umis");
    if output_dir.exists() {
        std::fs::remove_dir_all(&output_dir)?;
    }

    let report_path = bijux_dna_api::v1::api::fastq::write_local_extract_umis_smoke_report()?;
    assert_eq!(report_path, repo_root.join("target/local-smoke/fastq.extract_umis/report.json"));
    assert!(report_path.is_file(), "local extract-umis summary must exist");

    let payload: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&report_path)?)?;
    assert_eq!(payload["stage_id"], serde_json::json!("fastq.extract_umis"));
    assert_eq!(payload["sample_id"], serde_json::json!("known-prefix-pe"));
    assert_eq!(payload["planned_tool_id"], serde_json::json!("umi_tools"));
    assert_eq!(payload["report_tool_id"], serde_json::json!("bijux"));
    assert_eq!(payload["umi_pattern"], serde_json::json!("NNNN"));
    assert_eq!(payload["extracted_umi_count"], serde_json::json!(2));
    assert_eq!(payload["invalid_umi_count"], serde_json::json!(2));
    assert_eq!(payload["tag_header_format"], serde_json::json!("append_to_header"));

    let top_level_r1 = repo_root.join(
        payload["umi_extracted_fastq_gz"]
            .as_str()
            .unwrap_or_else(|| panic!("umi_extracted_fastq_gz missing")),
    );
    let top_level_r2 = repo_root.join(
        payload["umi_extracted_r2_fastq_gz"]
            .as_str()
            .unwrap_or_else(|| panic!("umi_extracted_r2_fastq_gz missing")),
    );
    assert!(top_level_r1.is_file(), "top-level UMI-extracted R1 must exist");
    assert!(top_level_r2.is_file(), "top-level UMI-extracted R2 must exist");
    assert_eq!(
        read_gz_fastq_headers(&top_level_r1)?,
        vec!["@umi-valid/1 umi:ACGT".to_string(), "@umi-invalid/1".to_string(),]
    );
    assert_eq!(
        read_gz_fastq_headers(&top_level_r2)?,
        vec!["@umi-valid/2 umi:ACGT".to_string(), "@umi-invalid/2".to_string(),]
    );

    let case_report_path = repo_root.join(
        payload["case_report_json"].as_str().unwrap_or_else(|| panic!("case_report_json missing")),
    );
    assert!(case_report_path.is_file(), "per-case extract-umis report must exist");
    let case_report: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&case_report_path)?)?;
    assert_eq!(case_report["stage_id"], serde_json::json!("fastq.extract_umis"));
    assert_eq!(case_report["tool_id"], serde_json::json!("bijux"));
    assert_eq!(case_report["reads_in"], serde_json::json!(4));
    assert_eq!(case_report["reads_out"], serde_json::json!(4));
    assert_eq!(case_report["reads_with_umi"], serde_json::json!(2));
    assert_eq!(case_report["failed_extractions"], serde_json::json!(2));
    assert_eq!(case_report["read_name_transform"], serde_json::json!("append_to_header"));
    assert_eq!(
        case_report["raw_backend_report_format"],
        serde_json::json!("governed_local_smoke_log")
    );

    let raw_backend_report = repo_root.join(
        payload["raw_backend_report"]
            .as_str()
            .unwrap_or_else(|| panic!("raw_backend_report missing")),
    );
    assert!(raw_backend_report.is_file(), "raw backend report must exist");
    let raw_backend_log = std::fs::read_to_string(&raw_backend_report)?;
    assert!(raw_backend_log.contains("governed_local_smoke_runtime=bijux"));
    assert!(raw_backend_log.contains("planned_tool_id=umi_tools"));

    Ok(())
}
