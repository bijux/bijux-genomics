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
fn local_bias_mitigation_smoke_plans_use_governed_bam_reference_and_expectations() -> Result<()> {
    let repo_root = repo_root();
    let plans = bijux_dna_planner_bam::stage_api::local_bias_mitigation_smoke_plans(&repo_root)?;
    assert_eq!(
        plans.len(),
        1,
        "governed local-smoke config must keep exactly one BAM bias-mitigation case"
    );

    let case = plans
        .iter()
        .find(|case| case.sample_id == "core-v1-bias-mitigation-gc-window-ladder")
        .unwrap_or_else(|| panic!("governed BAM bias-mitigation case missing"));
    assert_eq!(case.plan.stage_id.as_str(), "bam.bias_mitigation");
    assert_eq!(case.plan.tool_id.as_str(), "mapdamage2");
    assert_eq!(case.plan.resources.threads, 2);
    assert_eq!(
        case.bam,
        PathBuf::from("assets/toy/core-v1/bam/bias_mitigation_gc_window_reads.sam")
    );
    assert_eq!(
        case.reference,
        PathBuf::from("assets/toy/core-v1/bam/bias_mitigation_reference_windows.fasta")
    );
    assert_eq!(case.window_size, 10);
    assert_eq!(case.expected_metric_name, "gc_bias_score");
    assert!((case.expected_pre_mitigation_metric - 0.25).abs() <= 1e-9);
    assert!((case.expected_post_mitigation_metric - 0.125).abs() <= 1e-9);
    assert_eq!(
        case.plan.out_dir,
        PathBuf::from(
            "target/local-smoke/bam.bias_mitigation/core-v1-bias-mitigation-gc-window-ladder/mapdamage2"
        )
    );
    assert_eq!(
        case.plan.params["bam"],
        serde_json::json!("assets/toy/core-v1/bam/bias_mitigation_gc_window_reads.sam")
    );
    assert_eq!(
        case.plan.params["reference"],
        serde_json::json!("assets/toy/core-v1/bam/bias_mitigation_reference_windows.fasta")
    );
    assert_eq!(case.plan.params["window_size"], serde_json::json!(10));
    assert_eq!(case.plan.params["gc_bias_correction"], serde_json::json!(true));
    assert_eq!(case.plan.params["map_bias_correction"], serde_json::json!(false));

    let input_names = case
        .plan
        .io
        .inputs
        .iter()
        .map(|artifact| artifact.name.as_str().to_string())
        .collect::<Vec<_>>();
    assert_eq!(input_names, vec!["bam", "reference"]);

    let output_names = case
        .plan
        .io
        .outputs
        .iter()
        .map(|artifact| artifact.name.as_str().to_string())
        .collect::<Vec<_>>();
    assert_eq!(output_names, vec!["bias_report", "summary", "stage_metrics"]);

    let summary_output = case
        .plan
        .io
        .outputs
        .iter()
        .find(|artifact| artifact.name.as_str() == "summary")
        .unwrap_or_else(|| panic!("bias-mitigation summary output missing from BAM plan"));
    assert_eq!(
        summary_output.path,
        PathBuf::from(
            "target/local-smoke/bam.bias_mitigation/core-v1-bias-mitigation-gc-window-ladder/mapdamage2/bias.summary.json"
        )
    );

    Ok(())
}

#[test]
fn local_bias_mitigation_smoke_stage_api_surface_stays_callable() {
    let _: fn(
        &Path,
    ) -> anyhow::Result<Vec<bijux_dna_planner_bam::stage_api::LocalBiasMitigationSmokeCasePlan>> =
        bijux_dna_planner_bam::stage_api::local_bias_mitigation_smoke_plans;
}
