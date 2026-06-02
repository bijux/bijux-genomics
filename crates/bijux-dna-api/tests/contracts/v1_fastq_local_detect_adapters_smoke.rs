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
fn write_local_detect_adapters_smoke_report_materializes_governed_outputs() -> Result<()> {
    let repo_root = repo_root()?;
    let _guard = RepoRootOverrideGuard::install(&repo_root);
    let output_dir = repo_root.join("target/local-smoke/fastq.detect_adapters");
    if output_dir.exists() {
        std::fs::remove_dir_all(&output_dir)?;
    }

    let report_path = bijux_dna_api::v1::api::fastq::write_local_detect_adapters_smoke_report()?;
    assert_eq!(
        report_path,
        repo_root.join("target/local-smoke/fastq.detect_adapters/adapters.json")
    );
    assert!(report_path.is_file(), "local-smoke adapter report must exist");

    let payload: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&report_path)?)?;
    assert_eq!(
        payload["schema_version"],
        serde_json::json!("bijux.fastq.detect_adapters.local_smoke.report.v2")
    );
    assert_eq!(payload["stage_id"], serde_json::json!("fastq.detect_adapters"));
    assert_eq!(payload["case_count"], serde_json::json!(2));
    assert_eq!(payload["detected_case_count"], serde_json::json!(1));
    assert_eq!(payload["below_threshold_case_count"], serde_json::json!(1));

    let cases = payload["cases"].as_array().unwrap_or_else(|| panic!("cases array missing"));
    assert_eq!(cases.len(), 2);
    assert!(cases.iter().any(|case| {
        case["sample_id"] == serde_json::json!("adapter-hit-se")
            && case["adapter_status"] == serde_json::json!("adapter_detected")
            && case["adapter_report"].as_str().is_some()
            && case["candidate_adapter_count"].as_u64().unwrap_or(0) > 0
            && case["detected_adapter_ids"]
                .as_array()
                .is_some_and(|values| values.iter().any(|value| value == "truseq_universal"))
            && case["detection_confidence"].as_f64().is_some()
            && case["detection_threshold"].as_f64().is_some()
            && case["recommended_adapter_preset"] == serde_json::json!("illumina-default")
    }));
    assert!(cases.iter().any(|case| {
        case["sample_id"] == serde_json::json!("adapter-clear-se")
            && case["adapter_status"] == serde_json::json!("below_threshold")
            && case["adapter_report"].as_str().is_some()
            && case["candidate_adapter_count"] == serde_json::json!(0)
            && case["detected_adapter_ids"] == serde_json::json!([])
            && case["detection_confidence"].is_null()
            && case["detection_threshold"].as_f64().is_some()
            && case["recommended_adapter_preset"].is_null()
    }));

    for case in cases {
        let report_json = repo_root.join(
            case["adapter_report"]
                .as_str()
                .unwrap_or_else(|| panic!("adapter_report path missing")),
        );
        let adapter_evidence_dir = repo_root.join(
            case["adapter_evidence_dir"]
                .as_str()
                .unwrap_or_else(|| panic!("adapter_evidence_dir path missing")),
        );
        assert!(report_json.is_file(), "adapter report must exist");
        assert!(adapter_evidence_dir.is_dir(), "adapter evidence dir must exist");
        assert!(
            adapter_evidence_dir.join("normalized_adapter_evidence.json").is_file(),
            "normalized adapter evidence must exist"
        );
    }

    Ok(())
}
