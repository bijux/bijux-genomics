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
fn write_local_validate_smoke_report_materializes_governed_outputs() -> Result<()> {
    let repo_root = repo_root()?;
    let _guard = RepoRootOverrideGuard::install(&repo_root);
    let output_dir = repo_root.join("target/local-smoke/bam.validate");
    if output_dir.exists() {
        std::fs::remove_dir_all(&output_dir)?;
    }

    let report_path = bijux_dna_api::v1::api::bam::write_local_validate_smoke_report()?;
    assert_eq!(report_path, repo_root.join("target/local-smoke/bam.validate/validation.json"));
    assert!(report_path.is_file(), "local-smoke BAM validation summary must exist");

    let payload: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&report_path)?)?;
    assert_eq!(payload["stage_id"], serde_json::json!("bam.validate"));
    assert_eq!(payload["case_count"], serde_json::json!(2));
    assert_eq!(payload["all_cases_matched"], serde_json::json!(true));

    let cases = payload["cases"].as_array().unwrap_or_else(|| panic!("cases array missing"));
    assert_eq!(cases.len(), 2);
    assert!(cases.iter().any(|case| {
        case["sample_id"] == serde_json::json!("core-v1-coordinate-pass")
            && case["alignment_fixture_encoding"] == serde_json::json!("binary_bam")
            && case["validation_status"] == serde_json::json!("pass")
            && case["validation_errors"] == serde_json::json!([])
            && case["validation_warnings"] == serde_json::json!([])
            && case["expectation_matched"] == serde_json::json!(true)
            && case["validation_report_present"] == serde_json::json!(true)
            && case["input_bam_identity"]["input_bam"]
                == serde_json::json!("assets/toy/core-v1/bam/validation_pass.bam")
            && case["input_bam_identity"]["bam_index"]
                == serde_json::json!("assets/toy/core-v1/bam/validation_pass.bam.bai")
            && case["input_bam_identity"]["reference_fasta"]
                == serde_json::json!("assets/toy/core-v1/bam/validation_reference.fasta")
            && case["mapped_reads"] == serde_json::json!(2)
    }));
    assert!(cases.iter().any(|case| {
        case["sample_id"] == serde_json::json!("core-v1-malformed-refusal")
            && case["alignment_fixture_encoding"] == serde_json::json!("binary_bam")
            && case["validation_status"] == serde_json::json!("refusal")
            && case["validation_errors"] == serde_json::json!(["malformed_alignment_record"])
            && case["validation_warnings"] == serde_json::json!([])
            && case["expectation_matched"] == serde_json::json!(true)
            && case["validation_report_present"] == serde_json::json!(false)
            && case["input_bam_identity"]["input_bam"]
                == serde_json::json!("assets/toy/core-v1/bam/validation_malformed.bam")
            && case["input_bam_identity"]["bam_index"] == serde_json::Value::Null
            && case["input_bam_identity"]["reference_fasta"] == serde_json::Value::Null
            && case["refusal_codes"].as_array().is_some_and(|codes| {
                codes.contains(&serde_json::json!("malformed_alignment_record"))
            })
    }));

    for case in cases {
        let validation_report = repo_root.join(
            case["validation_report"]
                .as_str()
                .unwrap_or_else(|| panic!("validation_report path missing")),
        );
        let flagstat = repo_root
            .join(case["flagstat"].as_str().unwrap_or_else(|| panic!("flagstat path missing")));
        let stage_metrics = repo_root.join(
            case["stage_metrics"].as_str().unwrap_or_else(|| panic!("stage_metrics path missing")),
        );
        assert!(validation_report.is_file(), "case validation report must exist");
        assert!(flagstat.is_file(), "case flagstat must exist");
        assert!(stage_metrics.is_file(), "case stage metrics must exist");

        let stage_metrics_payload: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&stage_metrics)?)?;
        assert_eq!(stage_metrics_payload["stage_id"], serde_json::json!("bam.validate"));
        assert_eq!(
            stage_metrics_payload["alignment_fixture_encoding"], case["alignment_fixture_encoding"],
            "stage metrics must retain the governed alignment fixture encoding"
        );
        assert_eq!(
            stage_metrics_payload["validation_status"], case["validation_status"],
            "stage metrics must retain the validation status alias"
        );
        assert_eq!(
            stage_metrics_payload["validation_errors"], case["validation_errors"],
            "stage metrics must retain the validation errors alias"
        );
        assert_eq!(
            stage_metrics_payload["validation_warnings"], case["validation_warnings"],
            "stage metrics must retain the validation warnings alias"
        );
        assert_eq!(
            stage_metrics_payload["input_bam_identity"], case["input_bam_identity"],
            "stage metrics must retain the input BAM identity payload"
        );
    }

    Ok(())
}
