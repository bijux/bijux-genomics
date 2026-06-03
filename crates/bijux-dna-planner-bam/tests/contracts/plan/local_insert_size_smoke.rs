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

#[test]
fn local_insert_size_smoke_plans_use_governed_paired_fixture() -> Result<()> {
    let repo_root = repo_root();
    let plans = bijux_dna_planner_bam::stage_api::local_insert_size_smoke_plans(&repo_root)?;
    assert_eq!(
        plans.len(),
        1,
        "governed local-smoke config must keep exactly one BAM insert-size case"
    );

    let case = plans
        .iter()
        .find(|case| case.sample_id == "core-v1-paired-triplet")
        .unwrap_or_else(|| panic!("governed BAM insert-size case missing"));
    assert_eq!(case.plan.stage_id.as_str(), "bam.insert_size");
    assert_eq!(case.plan.tool_id.as_str(), "picard");
    assert_eq!(case.plan.resources.threads, 2);
    assert_eq!(
        case.bam,
        PathBuf::from("assets/toy/core-v1/bam/insert_size_paired_triplet.sam")
    );
    assert_eq!(case.expected_read_pairs, 3);
    assert!((case.expected_median_insert_size - 20.0).abs() <= 1e-9);
    assert!((case.expected_mean_insert_size - 21.666666666666668).abs() <= 1e-9);
    assert_eq!(case.expected_min_insert_size, 15);
    assert_eq!(case.expected_max_insert_size, 30);
    assert_eq!(
        case.plan.out_dir,
        PathBuf::from("target/local-smoke/bam.insert_size/core-v1-paired-triplet/picard")
    );
    assert_eq!(
        case.plan.params["bam"],
        serde_json::json!("assets/toy/core-v1/bam/insert_size_paired_triplet.sam")
    );

    let output_names = case
        .plan
        .io
        .outputs
        .iter()
        .map(|artifact| artifact.name.as_str().to_string())
        .collect::<Vec<_>>();
    assert_eq!(
        output_names,
        vec!["insert_size_report", "insert_size_histogram", "summary", "stage_metrics"]
    );

    let summary_output = case
        .plan
        .io
        .outputs
        .iter()
        .find(|artifact| artifact.name.as_str() == "summary")
        .unwrap_or_else(|| panic!("insert-size summary output missing from BAM insert-size plan"));
    assert_eq!(
        summary_output.path,
        PathBuf::from(
            "target/local-smoke/bam.insert_size/core-v1-paired-triplet/picard/insert_size.summary.json"
        )
    );

    Ok(())
}

#[test]
fn local_insert_size_smoke_stage_api_surface_stays_callable() {
    let _: fn(
        &Path,
    )
        -> anyhow::Result<Vec<bijux_dna_planner_bam::stage_api::LocalInsertSizeSmokeCasePlan>> =
        bijux_dna_planner_bam::stage_api::local_insert_size_smoke_plans;
}

fn write_local_insert_size_config(root: &Path, body: &str) -> Result<()> {
    let config_dir = root.join("configs/bench/local");
    fs::create_dir_all(&config_dir)?;
    fs::write(config_dir.join("bam-insert-size.toml"), body)?;
    Ok(())
}

#[test]
fn local_insert_size_smoke_plans_reject_empty_sample_ids() -> Result<()> {
    let temp = tempfile::tempdir()?;
    write_local_insert_size_config(
        temp.path(),
        r#"
schema_version = "bijux.bench.bam.local_insert_size.v1"
tool_id = "picard"

[[cases]]
sample_id = " "
bam = "assets/toy/core-v1/bam/insert_size_paired_triplet.sam"
expected_read_pairs = 3
expected_median_insert_size = 20.0
expected_mean_insert_size = 21.666666666666668
expected_min_insert_size = 15
expected_max_insert_size = 30
"#,
    )?;

    let error = bijux_dna_planner_bam::stage_api::local_insert_size_smoke_plans(temp.path())
        .expect_err("empty sample_id must be rejected before plan construction");
    assert_eq!(error.to_string(), "local-smoke bam.insert_size sample_id must not be empty");
    Ok(())
}

#[test]
fn local_insert_size_smoke_plans_reject_duplicate_sample_ids() -> Result<()> {
    let temp = tempfile::tempdir()?;
    write_local_insert_size_config(
        temp.path(),
        r#"
schema_version = "bijux.bench.bam.local_insert_size.v1"
tool_id = "picard"

[[cases]]
sample_id = "duplicate-case"
bam = "assets/toy/core-v1/bam/insert_size_paired_triplet.sam"
expected_read_pairs = 3
expected_median_insert_size = 20.0
expected_mean_insert_size = 21.666666666666668
expected_min_insert_size = 15
expected_max_insert_size = 30

[[cases]]
sample_id = "duplicate-case"
bam = "assets/toy/core-v1/bam/insert_size_paired_triplet.sam"
expected_read_pairs = 3
expected_median_insert_size = 20.0
expected_mean_insert_size = 21.666666666666668
expected_min_insert_size = 15
expected_max_insert_size = 30
"#,
    )?;

    let error = bijux_dna_planner_bam::stage_api::local_insert_size_smoke_plans(temp.path())
        .expect_err("duplicate sample_id must be rejected before plan construction");
    assert_eq!(
        error.to_string(),
        "duplicate local-smoke bam.insert_size sample_id `duplicate-case`"
    );
    Ok(())
}
