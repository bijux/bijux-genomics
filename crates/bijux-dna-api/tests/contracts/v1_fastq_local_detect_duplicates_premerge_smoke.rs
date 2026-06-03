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
fn write_local_detect_duplicates_premerge_smoke_report_materializes_governed_outputs() -> Result<()>
{
    let repo_root = repo_root()?;
    let _guard = RepoRootOverrideGuard::install(&repo_root);
    let output_dir = repo_root.join("target/local-smoke/fastq.detect_duplicates_premerge");
    if output_dir.exists() {
        std::fs::remove_dir_all(&output_dir)?;
    }

    let report_path =
        bijux_dna_api::v1::api::fastq::write_local_detect_duplicates_premerge_smoke_report()?;
    assert_eq!(
        report_path,
        repo_root.join("target/local-smoke/fastq.detect_duplicates_premerge/duplicates.json")
    );
    assert!(report_path.is_file(), "local-smoke duplicate summary must exist");

    let payload: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&report_path)?)?;
    assert_eq!(payload["stage_id"], serde_json::json!("fastq.detect_duplicates_premerge"));
    assert_eq!(payload["case_count"], serde_json::json!(2));
    assert_eq!(payload["duplicate_signal_case_count"], serde_json::json!(1));
    assert_eq!(payload["no_duplicate_signal_case_count"], serde_json::json!(1));

    let cases = payload["cases"].as_array().unwrap_or_else(|| panic!("cases array missing"));
    assert_eq!(cases.len(), 2);

    let duplicate_hit = cases
        .iter()
        .find(|case| case["sample_id"] == serde_json::json!("duplicate-hit-pe"))
        .unwrap_or_else(|| panic!("duplicate-hit-pe case missing from duplicate summary"));
    assert_eq!(duplicate_hit["layout"], serde_json::json!("paired_end"));
    assert_eq!(duplicate_hit["reads_in"], serde_json::json!(6));
    assert_eq!(duplicate_hit["duplicate_signal_reads"], serde_json::json!(2));
    assert_eq!(duplicate_hit["duplicate_signal_fraction"], serde_json::json!(2.0_f64 / 6.0_f64));
    assert_eq!(duplicate_hit["inspected_read_pair_count"], serde_json::json!(3));
    assert_eq!(duplicate_hit["duplicate_status"], serde_json::json!("duplicate_signal_detected"));

    let duplicate_clear = cases
        .iter()
        .find(|case| case["sample_id"] == serde_json::json!("duplicate-clear-pe"))
        .unwrap_or_else(|| panic!("duplicate-clear-pe case missing from duplicate summary"));
    assert_eq!(duplicate_clear["layout"], serde_json::json!("paired_end"));
    assert_eq!(duplicate_clear["reads_in"], serde_json::json!(4));
    assert_eq!(duplicate_clear["duplicate_signal_reads"], serde_json::json!(0));
    assert_eq!(duplicate_clear["duplicate_signal_fraction"], serde_json::json!(0.0));
    assert_eq!(duplicate_clear["inspected_read_pair_count"], serde_json::json!(2));
    assert_eq!(duplicate_clear["duplicate_status"], serde_json::json!("no_duplicate_signal"));

    for case in cases {
        let report_json = repo_root.join(
            case["report_json"].as_str().unwrap_or_else(|| panic!("report_json path missing")),
        );
        assert!(report_json.is_file(), "duplicate signal report must exist");
    }

    Ok(())
}
