use anyhow::Result;
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
fn write_local_validate_reads_smoke_report_materializes_governed_outputs() -> Result<()> {
    let repo_root = repo_root()?;
    let _guard = RepoRootOverrideGuard::install(&repo_root);
    let output_dir = repo_root.join("target/local-smoke/fastq.validate_reads");
    if output_dir.exists() {
        std::fs::remove_dir_all(&output_dir)?;
    }

    let report_path = bijux_dna_api::v1::api::fastq::write_local_validate_reads_smoke_report()?;
    assert_eq!(report_path, repo_root.join("target/local-smoke/fastq.validate_reads/report.json"));
    assert!(report_path.is_file(), "local-smoke validate report must exist");

    let payload: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&report_path)?)?;
    assert_eq!(payload["stage_id"], serde_json::json!("fastq.validate_reads"));
    assert_eq!(payload["case_count"], serde_json::json!(2));
    assert_eq!(payload["all_cases_passed"], serde_json::json!(true));
    assert_eq!(payload["missing_output_marker_present"], serde_json::json!(false));

    let cases = payload["cases"].as_array().unwrap_or_else(|| panic!("cases array missing"));
    assert_eq!(cases.len(), 2);
    assert!(cases.iter().any(|case| {
        case["sample_id"] == serde_json::json!("toy-se")
            && case["layout"] == serde_json::json!("single_end")
            && case["input_read_count_total"] == serde_json::json!(2)
            && case["validation_status"] == serde_json::json!("pass")
    }));
    assert!(cases.iter().any(|case| {
        case["sample_id"] == serde_json::json!("toy-pe")
            && case["layout"] == serde_json::json!("paired_end")
            && case["input_read_count_total"] == serde_json::json!(4)
            && case["input_pair_count"] == serde_json::json!(2)
            && case["validation_status"] == serde_json::json!("pass")
    }));

    for case in cases {
        let validation_report = repo_root.join(
            case["validation_report"]
                .as_str()
                .unwrap_or_else(|| panic!("validation_report path missing")),
        );
        let validated_reads_manifest = repo_root.join(
            case["validated_reads_manifest"]
                .as_str()
                .unwrap_or_else(|| panic!("validated_reads_manifest path missing")),
        );
        assert!(validation_report.is_file(), "validation report must exist");
        assert!(validated_reads_manifest.is_file(), "validated reads manifest must exist");
    }

    Ok(())
}
