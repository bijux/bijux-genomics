#![cfg(feature = "bam_downstream")]

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

fn case_by_sample_id<'a>(payload: &'a serde_json::Value, sample_id: &str) -> &'a serde_json::Value {
    payload["cases"]
        .as_array()
        .and_then(|cases| cases.iter().find(|case| case["sample_id"] == serde_json::json!(sample_id)))
        .unwrap_or_else(|| panic!("case `{sample_id}` missing from bam.kinship smoke report"))
}

#[test]
fn write_local_kinship_smoke_report_materializes_governed_outputs() -> Result<()> {
    let repo_root = repo_root()?;
    let _guard = RepoRootOverrideGuard::install(&repo_root);
    let output_dir = repo_root.join("target/local-smoke/bam.kinship");
    if output_dir.exists() {
        std::fs::remove_dir_all(&output_dir)?;
    }

    let report_path = bijux_dna_api::v1::api::bam::write_local_kinship_smoke_report()?;
    assert_eq!(report_path, repo_root.join("target/local-smoke/bam.kinship/kinship.json"));
    assert!(report_path.is_file(), "local-smoke BAM kinship report must exist");

    let payload: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&report_path)?)?;
    assert_eq!(payload["stage_id"], serde_json::json!("bam.kinship"));
    assert_eq!(
        payload["schema_version"],
        serde_json::json!("bijux.bam.kinship.local_smoke.report.v1")
    );
    assert_eq!(payload["case_count"], serde_json::json!(2));
    assert_eq!(payload["all_cases_matched"], serde_json::json!(true));

    let insufficient = case_by_sample_id(&payload, "core-v1-kinship-insufficient-overlap");
    assert_eq!(insufficient["expectation_matched"], serde_json::json!(true));
    assert_eq!(insufficient["method"], serde_json::json!("king"));
    assert_eq!(insufficient["reference_panel"], serde_json::json!("toy_human_relatedness_panel"));
    assert_eq!(insufficient["reference_build"], serde_json::json!("grch38"));
    assert_eq!(insufficient["population_scope"], serde_json::json!("human_diploid_panel"));
    assert_eq!(insufficient["min_overlap_snps"], serde_json::json!(5));
    assert_eq!(insufficient["requires_cohort_context"], serde_json::json!(true));
    assert_eq!(insufficient["observed_max_overlap_snps"], serde_json::json!(4));
    assert_eq!(insufficient["pair_count"], serde_json::json!(0));
    assert_eq!(insufficient["status"], serde_json::json!("insufficient"));
    assert_eq!(
        insufficient["insufficiency_reason"],
        serde_json::json!("insufficient_overlap_snps")
    );
    assert_eq!(insufficient["pairwise_results"], serde_json::json!([]));

    let valid = case_by_sample_id(&payload, "core-v1-kinship-related-pair");
    assert_eq!(valid["expectation_matched"], serde_json::json!(true));
    assert_eq!(valid["method"], serde_json::json!("king"));
    assert_eq!(valid["reference_panel"], serde_json::json!("toy_human_relatedness_panel"));
    assert_eq!(valid["reference_build"], serde_json::json!("grch38"));
    assert_eq!(valid["population_scope"], serde_json::json!("human_diploid_panel"));
    assert_eq!(valid["min_overlap_snps"], serde_json::json!(6));
    assert_eq!(valid["requires_cohort_context"], serde_json::json!(true));
    assert_eq!(valid["observed_max_overlap_snps"], serde_json::json!(6));
    assert_eq!(valid["pair_count"], serde_json::json!(1));
    assert_eq!(valid["status"], serde_json::json!("ok"));
    assert_eq!(valid["insufficiency_reason"], serde_json::Value::Null);
    assert_eq!(valid["pairwise_results"][0]["sample_a"], serde_json::json!("sample_a"));
    assert_eq!(valid["pairwise_results"][0]["sample_b"], serde_json::json!("sample_b"));
    assert_eq!(valid["pairwise_results"][0]["overlap_snps"], serde_json::json!(6));
    assert_eq!(valid["pairwise_results"][0]["matching_sites"], serde_json::json!(5));
    assert_eq!(valid["pairwise_results"][0]["mismatch_sites"], serde_json::json!(1));
    assert_eq!(valid["pairwise_results"][0]["concordance"], serde_json::json!(0.833333));
    assert_eq!(
        valid["pairwise_results"][0]["kinship_coefficient"],
        serde_json::json!(0.416667)
    );
    assert_eq!(
        valid["pairwise_results"][0]["relationship_label"],
        serde_json::json!("first_degree")
    );

    for case in [insufficient, valid] {
        let kinship_report = repo_root.join(
            case["kinship_report"]
                .as_str()
                .unwrap_or_else(|| panic!("kinship_report path missing")),
        );
        let kinship_summary = repo_root.join(
            case["kinship_summary"]
                .as_str()
                .unwrap_or_else(|| panic!("kinship_summary path missing")),
        );
        let kinship_segments = repo_root.join(
            case["kinship_segments"]
                .as_str()
                .unwrap_or_else(|| panic!("kinship_segments path missing")),
        );
        let stage_metrics = repo_root.join(
            case["stage_metrics"]
                .as_str()
                .unwrap_or_else(|| panic!("stage_metrics path missing")),
        );
        for path in [&kinship_report, &kinship_summary, &kinship_segments, &stage_metrics] {
            assert!(path.is_file(), "governed BAM kinship artifact must exist: {}", path.display());
        }
    }

    let valid_summary_path = repo_root.join(
        valid["kinship_summary"]
            .as_str()
            .unwrap_or_else(|| panic!("valid kinship_summary path missing")),
    );
    let valid_summary: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&valid_summary_path)?)?;
    assert_eq!(
        valid_summary["schema_version"],
        serde_json::json!("bijux.bam.kinship_summary.v1")
    );
    assert_eq!(valid_summary["stage_id"], serde_json::json!("bam.kinship"));
    assert_eq!(valid_summary["method"], serde_json::json!("king"));
    assert_eq!(valid_summary["reference_panel"], serde_json::json!("toy_human_relatedness_panel"));
    assert_eq!(valid_summary["observed_max_overlap_snps"], serde_json::json!(6));
    assert_eq!(valid_summary["pair_count"], serde_json::json!(1));
    assert_eq!(valid_summary["status"], serde_json::json!("ok"));

    let valid_segments_path = repo_root.join(
        valid["kinship_segments"]
            .as_str()
            .unwrap_or_else(|| panic!("valid kinship_segments path missing")),
    );
    let valid_segments = std::fs::read_to_string(&valid_segments_path)?;
    assert!(
        valid_segments.contains(
            "sample_a\tsample_b\t6\t5\t1\t0.833333\t0.416667\tfirst_degree"
        ),
        "kinship segments report must contain the governed valid pair row"
    );

    let insufficient_metrics_path = repo_root.join(
        insufficient["stage_metrics"]
            .as_str()
            .unwrap_or_else(|| panic!("insufficient stage_metrics path missing")),
    );
    let insufficient_metrics: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&insufficient_metrics_path)?)?;
    assert_eq!(
        insufficient_metrics["schema_version"],
        serde_json::json!("bijux.bam.kinship.local_smoke.metrics.v1")
    );
    assert_eq!(insufficient_metrics["status"], serde_json::json!("insufficient"));
    assert_eq!(
        insufficient_metrics["insufficiency_reason"],
        serde_json::json!("insufficient_overlap_snps")
    );
    assert_eq!(insufficient_metrics["expectation_matched"], serde_json::json!(true));

    Ok(())
}
