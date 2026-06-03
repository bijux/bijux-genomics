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

fn stage_api_temp_repo() -> Result<tempfile::TempDir> {
    let temp = tempfile::tempdir()?;
    let repo_root = repo_root();
    let tool_dir = temp.path().join("domain/bam/tools");
    fs::create_dir_all(&tool_dir)?;
    fs::copy(repo_root.join("domain/bam/tools/picard.yaml"), tool_dir.join("picard.yaml"))?;
    Ok(temp)
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

#[test]
fn local_insert_size_smoke_plans_require_non_zero_read_pairs() -> Result<()> {
    let temp = stage_api_temp_repo()?;
    let repo_root = repo_root();
    write_local_insert_size_config(
        temp.path(),
        &format!(
            r#"
schema_version = "bijux.bench.bam.local_insert_size.v1"
tool_id = "picard"

[[cases]]
sample_id = "zero-pairs"
bam = "{bam}"
expected_read_pairs = 0
expected_median_insert_size = 20.0
expected_mean_insert_size = 21.666666666666668
expected_min_insert_size = 15
expected_max_insert_size = 30
"#,
            bam = repo_root.join("assets/toy/core-v1/bam/insert_size_paired_triplet.sam").display(),
        ),
    )?;

    let error = bijux_dna_planner_bam::stage_api::local_insert_size_smoke_plans(temp.path())
        .expect_err("insert-size cases must declare expected_read_pairs greater than zero");
    assert_eq!(
        error.to_string(),
        "local-smoke bam.insert_size case `zero-pairs` must declare expected_read_pairs greater than zero"
    );
    Ok(())
}

#[test]
fn local_insert_size_smoke_plans_require_non_zero_insert_size_bounds() -> Result<()> {
    let temp = stage_api_temp_repo()?;
    let repo_root = repo_root();
    write_local_insert_size_config(
        temp.path(),
        &format!(
            r#"
schema_version = "bijux.bench.bam.local_insert_size.v1"
tool_id = "picard"

[[cases]]
sample_id = "zero-bound"
bam = "{bam}"
expected_read_pairs = 3
expected_median_insert_size = 20.0
expected_mean_insert_size = 21.666666666666668
expected_min_insert_size = 0
expected_max_insert_size = 30
"#,
            bam = repo_root.join("assets/toy/core-v1/bam/insert_size_paired_triplet.sam").display(),
        ),
    )?;

    let error = bijux_dna_planner_bam::stage_api::local_insert_size_smoke_plans(temp.path())
        .expect_err("insert-size bounds must stay greater than zero");
    assert_eq!(
        error.to_string(),
        "local-smoke bam.insert_size case `zero-bound` must keep expected insert-size bounds greater than zero"
    );
    Ok(())
}

#[test]
fn local_insert_size_smoke_plans_require_ordered_insert_size_bounds() -> Result<()> {
    let temp = stage_api_temp_repo()?;
    let repo_root = repo_root();
    write_local_insert_size_config(
        temp.path(),
        &format!(
            r#"
schema_version = "bijux.bench.bam.local_insert_size.v1"
tool_id = "picard"

[[cases]]
sample_id = "inverted-bounds"
bam = "{bam}"
expected_read_pairs = 3
expected_median_insert_size = 20.0
expected_mean_insert_size = 21.666666666666668
expected_min_insert_size = 31
expected_max_insert_size = 30
"#,
            bam = repo_root.join("assets/toy/core-v1/bam/insert_size_paired_triplet.sam").display(),
        ),
    )?;

    let error = bijux_dna_planner_bam::stage_api::local_insert_size_smoke_plans(temp.path())
        .expect_err("insert-size bounds must stay ordered");
    assert_eq!(
        error.to_string(),
        "local-smoke bam.insert_size case `inverted-bounds` must keep expected min insert size less than or equal to expected max insert size"
    );
    Ok(())
}

#[test]
fn local_insert_size_smoke_plans_require_mean_within_bounds() -> Result<()> {
    let temp = stage_api_temp_repo()?;
    let repo_root = repo_root();
    write_local_insert_size_config(
        temp.path(),
        &format!(
            r#"
schema_version = "bijux.bench.bam.local_insert_size.v1"
tool_id = "picard"

[[cases]]
sample_id = "mean-out-of-bounds"
bam = "{bam}"
expected_read_pairs = 3
expected_median_insert_size = 20.0
expected_mean_insert_size = 40.0
expected_min_insert_size = 15
expected_max_insert_size = 30
"#,
            bam = repo_root.join("assets/toy/core-v1/bam/insert_size_paired_triplet.sam").display(),
        ),
    )?;

    let error = bijux_dna_planner_bam::stage_api::local_insert_size_smoke_plans(temp.path())
        .expect_err("insert-size mean must stay within declared bounds");
    assert_eq!(
        error.to_string(),
        "local-smoke bam.insert_size case `mean-out-of-bounds` must keep expected mean insert size within the declared bounds"
    );
    Ok(())
}

#[test]
fn local_insert_size_smoke_plans_require_median_within_bounds() -> Result<()> {
    let temp = stage_api_temp_repo()?;
    let repo_root = repo_root();
    write_local_insert_size_config(
        temp.path(),
        &format!(
            r#"
schema_version = "bijux.bench.bam.local_insert_size.v1"
tool_id = "picard"

[[cases]]
sample_id = "median-out-of-bounds"
bam = "{bam}"
expected_read_pairs = 3
expected_median_insert_size = 40.0
expected_mean_insert_size = 21.666666666666668
expected_min_insert_size = 15
expected_max_insert_size = 30
"#,
            bam = repo_root.join("assets/toy/core-v1/bam/insert_size_paired_triplet.sam").display(),
        ),
    )?;

    let error = bijux_dna_planner_bam::stage_api::local_insert_size_smoke_plans(temp.path())
        .expect_err("insert-size median must stay within declared bounds");
    assert_eq!(
        error.to_string(),
        "local-smoke bam.insert_size case `median-out-of-bounds` must keep expected median insert size within the declared bounds"
    );
    Ok(())
}
