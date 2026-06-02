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

fn assert_json_f64_close(value: &serde_json::Value, expected: f64) {
    let actual = value.as_f64().unwrap_or_else(|| panic!("expected JSON float, got {value}"));
    let delta = (actual - expected).abs();
    assert!(delta <= 1.0e-12, "expected {expected} but observed {actual} (delta {delta})");
}

#[test]
fn write_local_estimate_library_complexity_prealign_smoke_report_materializes_governed_outputs(
) -> Result<()> {
    let repo_root = repo_root()?;
    let _guard = RepoRootOverrideGuard::install(&repo_root);
    let output_dir =
        repo_root.join("target/local-smoke/fastq.estimate_library_complexity_prealign");
    if output_dir.exists() {
        std::fs::remove_dir_all(&output_dir)?;
    }

    let report_path = bijux_dna_api::v1::api::fastq::write_local_estimate_library_complexity_prealign_smoke_report()?;
    assert_eq!(
        report_path,
        repo_root
            .join("target/local-smoke/fastq.estimate_library_complexity_prealign/complexity.json")
    );
    assert!(report_path.is_file(), "local-smoke complexity summary must exist");

    let payload: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&report_path)?)?;
    assert_eq!(
        payload["stage_id"],
        serde_json::json!("fastq.estimate_library_complexity_prealign")
    );
    assert_eq!(payload["case_count"], serde_json::json!(2));
    assert_eq!(payload["estimated_case_count"], serde_json::json!(2));
    assert_eq!(payload["insufficient_reads_case_count"], serde_json::json!(0));

    let cases = payload["cases"].as_array().unwrap_or_else(|| panic!("cases array missing"));
    assert_eq!(cases.len(), 2);

    let complexity_hit = cases
        .iter()
        .find(|case| case["sample_id"] == serde_json::json!("complexity-hit-pe"))
        .unwrap_or_else(|| panic!("complexity-hit-pe case missing from complexity summary"));
    assert_eq!(complexity_hit["layout"], serde_json::json!("paired_end"));
    assert_eq!(complexity_hit["reads_in"], serde_json::json!(6));
    assert_eq!(complexity_hit["estimated_complexity"], serde_json::json!(2.0_f64 / 3.0_f64));
    assert_eq!(complexity_hit["estimated_unique_fraction"], serde_json::json!(2.0_f64 / 3.0_f64));
    assert_json_f64_close(&complexity_hit["estimated_duplicate_fraction"], 1.0_f64 / 3.0_f64);
    assert_eq!(complexity_hit["kmer_size"], serde_json::json!(4));
    assert_eq!(complexity_hit["insufficient_data_reason"], serde_json::Value::Null);
    assert_eq!(complexity_hit["complexity_status"], serde_json::json!("complexity_estimated"));

    let complexity_clear = cases
        .iter()
        .find(|case| case["sample_id"] == serde_json::json!("complexity-clear-pe"))
        .unwrap_or_else(|| panic!("complexity-clear-pe case missing from complexity summary"));
    assert_eq!(complexity_clear["layout"], serde_json::json!("paired_end"));
    assert_eq!(complexity_clear["reads_in"], serde_json::json!(4));
    assert_eq!(complexity_clear["estimated_complexity"], serde_json::json!(1.0));
    assert_eq!(complexity_clear["estimated_unique_fraction"], serde_json::json!(1.0));
    assert_eq!(complexity_clear["estimated_duplicate_fraction"], serde_json::json!(0.0));
    assert_eq!(complexity_clear["kmer_size"], serde_json::json!(4));
    assert_eq!(complexity_clear["insufficient_data_reason"], serde_json::Value::Null);
    assert_eq!(complexity_clear["complexity_status"], serde_json::json!("complexity_estimated"));

    for case in cases {
        let report_json = repo_root.join(
            case["report_json"].as_str().unwrap_or_else(|| panic!("report_json path missing")),
        );
        assert!(report_json.is_file(), "library complexity report must exist");
    }

    Ok(())
}
