#![cfg(feature = "bam_downstream")]

use anyhow::Result;
use std::fs;
use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .unwrap_or_else(|| panic!("workspace root"))
        .to_path_buf()
}

fn write_local_bias_mitigation_config(root: &Path, body: &str) -> Result<()> {
    let config_dir = root.join("configs/bench/local");
    fs::create_dir_all(&config_dir)?;
    fs::write(config_dir.join("bam-bias-mitigation.toml"), body)?;
    Ok(())
}

fn stage_api_temp_repo() -> Result<tempfile::TempDir> {
    let temp = tempfile::tempdir()?;
    let repo_root = repo_root();
    let tool_dir = temp.path().join("domain/bam/tools");
    fs::create_dir_all(&tool_dir)?;
    fs::copy(repo_root.join("domain/bam/tools/mapdamage2.yaml"), tool_dir.join("mapdamage2.yaml"))?;
    Ok(temp)
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
        .find(|case| case.sample_id == "human_like_gc_window_ladder")
        .unwrap_or_else(|| panic!("governed BAM bias-mitigation case missing"));
    assert_eq!(case.plan.stage_id.as_str(), "bam.bias_mitigation");
    assert_eq!(case.plan.tool_id.as_str(), "mapdamage2");
    assert_eq!(case.plan.resources.threads, 2);
    assert_eq!(
        case.bam,
        PathBuf::from(
            "tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_gc_window_ladder.sam"
        )
    );
    assert_eq!(
        case.reference,
        PathBuf::from(
            "tests/fixtures/corpora/corpus-01-bam-mini/reference/human_like_gc_window_ladder.fasta"
        )
    );
    assert_eq!(case.window_size, 10);
    assert_eq!(case.expected_metric_name, "gc_bias_score");
    assert!((case.expected_pre_mitigation_metric - 0.25).abs() <= 1e-9);
    assert!((case.expected_post_mitigation_metric - 0.125).abs() <= 1e-9);
    assert_eq!(
        case.plan.out_dir,
        PathBuf::from("target/local-smoke/bam.bias_mitigation/human_like_gc_window_ladder/mapdamage2")
    );
    assert_eq!(
        case.plan.params["bam"],
        serde_json::json!(
            "tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_gc_window_ladder.sam"
        )
    );
    assert_eq!(
        case.plan.params["reference"],
        serde_json::json!(
            "tests/fixtures/corpora/corpus-01-bam-mini/reference/human_like_gc_window_ladder.fasta"
        )
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
        PathBuf::from("target/local-smoke/bam.bias_mitigation/human_like_gc_window_ladder/mapdamage2/bias.summary.json")
    );

    Ok(())
}

#[test]
fn local_bias_mitigation_smoke_stage_api_surface_stays_callable() {
    let _: fn(
        &Path,
    ) -> anyhow::Result<
        Vec<bijux_dna_planner_bam::stage_api::LocalBiasMitigationSmokeCasePlan>,
    > = bijux_dna_planner_bam::stage_api::local_bias_mitigation_smoke_plans;
}

#[test]
fn local_bias_mitigation_smoke_plans_require_expected_metric_name_to_match_governed_summary(
) -> Result<()> {
    let temp = stage_api_temp_repo()?;
    let repo_root = repo_root();
    write_local_bias_mitigation_config(
        temp.path(),
        &format!(
            r#"
schema_version = "bijux.bench.bam.local_bias_mitigation.v1"
tool_id = "mapdamage2"
threads = 2
output_dir = "target/local-smoke/bam.bias_mitigation"

[[cases]]
sample_id = "wrong-metric-name"
bam = "{bam}"
reference = "{reference}"
window_size = 10
gc_bias_correction = true
map_bias_correction = false
expected_metric_name = "map_bias_score"
expected_pre_mitigation_metric = 0.25
expected_post_mitigation_metric = 0.125
"#,
            bam = repo_root
                .join(
                    "tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_gc_window_ladder.sam",
                )
                .display(),
            reference = repo_root
                .join(
                    "tests/fixtures/corpora/corpus-01-bam-mini/reference/human_like_gc_window_ladder.fasta",
                )
                .display(),
        ),
    )?;

    let error = bijux_dna_planner_bam::stage_api::local_bias_mitigation_smoke_plans(temp.path())
        .expect_err("expected_metric_name must stay aligned with the governed bias summary");
    assert_eq!(
        error.to_string(),
        "local-smoke bam.bias_mitigation case `wrong-metric-name` must keep expected_metric_name aligned with the governed bias summary"
    );
    Ok(())
}

#[test]
fn local_bias_mitigation_smoke_plans_require_expected_pre_metric_to_match_governed_summary(
) -> Result<()> {
    let temp = stage_api_temp_repo()?;
    let repo_root = repo_root();
    write_local_bias_mitigation_config(
        temp.path(),
        &format!(
            r#"
schema_version = "bijux.bench.bam.local_bias_mitigation.v1"
tool_id = "mapdamage2"
threads = 2
output_dir = "target/local-smoke/bam.bias_mitigation"

[[cases]]
sample_id = "wrong-pre-metric"
bam = "{bam}"
reference = "{reference}"
window_size = 10
gc_bias_correction = true
map_bias_correction = false
expected_metric_name = "gc_bias_score"
expected_pre_mitigation_metric = 0.3
expected_post_mitigation_metric = 0.125
"#,
            bam = repo_root
                .join(
                    "tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_gc_window_ladder.sam",
                )
                .display(),
            reference = repo_root
                .join(
                    "tests/fixtures/corpora/corpus-01-bam-mini/reference/human_like_gc_window_ladder.fasta",
                )
                .display(),
        ),
    )?;

    let error = bijux_dna_planner_bam::stage_api::local_bias_mitigation_smoke_plans(temp.path())
        .expect_err("expected_pre_mitigation_metric must stay aligned with the governed summary");
    assert_eq!(
        error.to_string(),
        "local-smoke bam.bias_mitigation case `wrong-pre-metric` must keep expected_pre_mitigation_metric aligned with the governed bias summary"
    );
    Ok(())
}

#[test]
fn local_bias_mitigation_smoke_plans_require_expected_post_metric_to_match_governed_summary(
) -> Result<()> {
    let temp = stage_api_temp_repo()?;
    let repo_root = repo_root();
    write_local_bias_mitigation_config(
        temp.path(),
        &format!(
            r#"
schema_version = "bijux.bench.bam.local_bias_mitigation.v1"
tool_id = "mapdamage2"
threads = 2
output_dir = "target/local-smoke/bam.bias_mitigation"

[[cases]]
sample_id = "wrong-post-metric"
bam = "{bam}"
reference = "{reference}"
window_size = 10
gc_bias_correction = true
map_bias_correction = false
expected_metric_name = "gc_bias_score"
expected_pre_mitigation_metric = 0.25
expected_post_mitigation_metric = 0.2
"#,
            bam = repo_root
                .join(
                    "tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_gc_window_ladder.sam",
                )
                .display(),
            reference = repo_root
                .join(
                    "tests/fixtures/corpora/corpus-01-bam-mini/reference/human_like_gc_window_ladder.fasta",
                )
                .display(),
        ),
    )?;

    let error = bijux_dna_planner_bam::stage_api::local_bias_mitigation_smoke_plans(temp.path())
        .expect_err("expected_post_mitigation_metric must stay aligned with the governed summary");
    assert_eq!(
        error.to_string(),
        "local-smoke bam.bias_mitigation case `wrong-post-metric` must keep expected_post_mitigation_metric aligned with the governed bias summary"
    );
    Ok(())
}

#[test]
fn local_bias_mitigation_smoke_plans_reject_empty_sample_ids() -> Result<()> {
    let temp = tempfile::tempdir()?;
    write_local_bias_mitigation_config(
        temp.path(),
        r#"
schema_version = "bijux.bench.bam.local_bias_mitigation.v1"
tool_id = "mapdamage2"
threads = 2
output_dir = "target/local-smoke/bam.bias_mitigation"

[[cases]]
sample_id = " "
bam = "tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_gc_window_ladder.sam"
reference = "tests/fixtures/corpora/corpus-01-bam-mini/reference/human_like_gc_window_ladder.fasta"
window_size = 10
gc_bias_correction = true
map_bias_correction = false
expected_metric_name = "gc_bias_score"
expected_pre_mitigation_metric = 0.25
expected_post_mitigation_metric = 0.125
"#,
    )?;

    let error = bijux_dna_planner_bam::stage_api::local_bias_mitigation_smoke_plans(temp.path())
        .expect_err("empty sample ids must be rejected");
    assert_eq!(error.to_string(), "local-smoke bam.bias_mitigation sample_id must not be empty");
    Ok(())
}

#[test]
fn local_bias_mitigation_smoke_plans_reject_duplicate_sample_ids() -> Result<()> {
    let temp = tempfile::tempdir()?;
    write_local_bias_mitigation_config(
        temp.path(),
        r#"
schema_version = "bijux.bench.bam.local_bias_mitigation.v1"
tool_id = "mapdamage2"
threads = 2
output_dir = "target/local-smoke/bam.bias_mitigation"

[[cases]]
sample_id = "duplicate-case"
bam = "tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_gc_window_ladder.sam"
reference = "tests/fixtures/corpora/corpus-01-bam-mini/reference/human_like_gc_window_ladder.fasta"
window_size = 10
gc_bias_correction = true
map_bias_correction = false
expected_metric_name = "gc_bias_score"
expected_pre_mitigation_metric = 0.25
expected_post_mitigation_metric = 0.125

[[cases]]
sample_id = "duplicate-case"
bam = "tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_gc_window_ladder.sam"
reference = "tests/fixtures/corpora/corpus-01-bam-mini/reference/human_like_gc_window_ladder.fasta"
window_size = 10
gc_bias_correction = true
map_bias_correction = false
expected_metric_name = "gc_bias_score"
expected_pre_mitigation_metric = 0.25
expected_post_mitigation_metric = 0.125
"#,
    )?;

    let error = bijux_dna_planner_bam::stage_api::local_bias_mitigation_smoke_plans(temp.path())
        .expect_err("duplicate sample ids must be rejected");
    assert_eq!(
        error.to_string(),
        "duplicate local-smoke bam.bias_mitigation sample_id `duplicate-case`"
    );
    Ok(())
}
