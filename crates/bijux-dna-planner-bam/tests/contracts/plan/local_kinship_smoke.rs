#![cfg(feature = "bam_downstream")]

use anyhow::Result;
use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .unwrap_or_else(|| panic!("workspace root"))
        .to_path_buf()
}

#[test]
fn local_kinship_smoke_plans_use_governed_pair_expectations() -> Result<()> {
    let repo_root = repo_root();
    let plans = bijux_dna_planner_bam::stage_api::local_kinship_smoke_plans(&repo_root)?;
    assert_eq!(plans.len(), 2, "governed local-smoke config must keep two BAM kinship cases");

    let insufficient = plans
        .iter()
        .find(|case| case.sample_id == "core-v1-kinship-insufficient-overlap")
        .unwrap_or_else(|| panic!("governed BAM kinship insufficient-overlap case missing"));
    assert_eq!(insufficient.plan.stage_id.as_str(), "bam.kinship");
    assert_eq!(insufficient.plan.tool_id.as_str(), "king");
    assert_eq!(insufficient.plan.resources.threads, 2);
    assert_eq!(
        insufficient.bam,
        PathBuf::from("assets/toy/core-v1/bam/kinship_low_overlap_pair.sam")
    );
    assert_eq!(insufficient.reference_panel, "toy_human_relatedness_panel");
    assert_eq!(insufficient.reference_build, "grch38");
    assert_eq!(insufficient.population_scope, "human_diploid_panel");
    assert_eq!(insufficient.min_overlap_snps, 5);
    assert!(insufficient.requires_cohort_context);
    assert_eq!(insufficient.expected_status, "insufficient");
    assert_eq!(insufficient.expected_observed_max_overlap_snps, 4);
    assert_eq!(
        insufficient.expected_insufficiency_reason.as_deref(),
        Some("insufficient_overlap_snps")
    );
    assert!(insufficient.expected_pairwise_results.is_empty());
    assert_eq!(
        insufficient.plan.out_dir,
        PathBuf::from(
            "target/local-smoke/bam.kinship/core-v1-kinship-insufficient-overlap/king"
        )
    );
    assert_eq!(
        insufficient.plan.params["reference_panel"],
        serde_json::json!("toy_human_relatedness_panel")
    );
    assert_eq!(insufficient.plan.params["min_overlap_snps"], serde_json::json!(5));
    assert_eq!(
        insufficient.plan.params["requires_cohort_context"],
        serde_json::json!(true)
    );

    let valid = plans
        .iter()
        .find(|case| case.sample_id == "core-v1-kinship-related-pair")
        .unwrap_or_else(|| panic!("governed BAM kinship valid pair case missing"));
    assert_eq!(valid.plan.stage_id.as_str(), "bam.kinship");
    assert_eq!(valid.plan.tool_id.as_str(), "king");
    assert_eq!(
        valid.bam,
        PathBuf::from("assets/toy/core-v1/bam/kinship_related_pair.sam")
    );
    assert_eq!(valid.reference_panel, "toy_human_relatedness_panel");
    assert_eq!(valid.reference_build, "grch38");
    assert_eq!(valid.population_scope, "human_diploid_panel");
    assert_eq!(valid.min_overlap_snps, 6);
    assert!(valid.requires_cohort_context);
    assert_eq!(valid.expected_status, "ok");
    assert_eq!(valid.expected_observed_max_overlap_snps, 6);
    assert_eq!(valid.expected_insufficiency_reason, None);
    assert_eq!(valid.expected_pairwise_results.len(), 1);
    let pair = &valid.expected_pairwise_results[0];
    assert_eq!(pair.sample_a, "sample_a");
    assert_eq!(pair.sample_b, "sample_b");
    assert_eq!(pair.overlap_snps, 6);
    assert_eq!(pair.matching_sites, 5);
    assert_eq!(pair.mismatch_sites, 1);
    assert!((pair.concordance - 0.833333).abs() <= 1e-9);
    assert!((pair.kinship_coefficient - 0.416667).abs() <= 1e-9);
    assert_eq!(pair.relationship_label, "first_degree");

    let output_names = valid
        .plan
        .io
        .outputs
        .iter()
        .map(|artifact| artifact.name.as_str().to_string())
        .collect::<Vec<_>>();
    assert_eq!(output_names, vec!["kinship_report", "summary", "stage_metrics"]);

    let summary_output = valid
        .plan
        .io
        .outputs
        .iter()
        .find(|artifact| artifact.name.as_str() == "summary")
        .unwrap_or_else(|| panic!("kinship summary output missing from BAM kinship plan"));
    assert_eq!(
        summary_output.path,
        PathBuf::from(
            "target/local-smoke/bam.kinship/core-v1-kinship-related-pair/king/kinship.summary.json"
        )
    );

    Ok(())
}

#[test]
fn local_kinship_smoke_stage_api_surface_stays_callable() {
    let _: fn(&Path) -> anyhow::Result<Vec<bijux_dna_planner_bam::stage_api::LocalKinshipSmokeCasePlan>> =
        bijux_dna_planner_bam::stage_api::local_kinship_smoke_plans;
}
