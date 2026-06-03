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

fn write_local_sex_config(root: &Path, body: &str) -> Result<()> {
    let config_dir = root.join("configs/bench/local");
    fs::create_dir_all(&config_dir)?;
    fs::write(config_dir.join("bam-sex.toml"), body)?;
    Ok(())
}

#[test]
fn local_sex_smoke_plans_use_governed_bam_reference_and_expectations() -> Result<()> {
    let repo_root = repo_root();
    let plans = bijux_dna_planner_bam::stage_api::local_sex_smoke_plans(&repo_root)?;
    assert_eq!(plans.len(), 1, "governed local-smoke config must keep exactly one BAM sex case");

    let case = plans
        .iter()
        .find(|case| case.sample_id == "core-v1-sex-xy-autosome-male")
        .unwrap_or_else(|| panic!("governed BAM sex case missing"));
    assert_eq!(case.plan.stage_id.as_str(), "bam.sex");
    assert_eq!(case.plan.tool_id.as_str(), "rxy");
    assert_eq!(case.plan.resources.threads, 2);
    assert_eq!(case.bam, PathBuf::from("assets/toy/core-v1/bam/sex_xy_autosome_male.sam"));
    assert_eq!(
        case.reference,
        PathBuf::from("assets/toy/core-v1/bam/sex_reference_xy_autosome.fasta")
    );
    assert_eq!(case.chromosome_system, "xy");
    assert_eq!(case.minimum_y_sites, 5);
    assert_eq!(case.expected_method, "rxy");
    assert!((case.expected_x_coverage - 0.5).abs() <= 1e-9);
    assert!((case.expected_y_coverage - 0.5).abs() <= 1e-9);
    assert!((case.expected_autosomal_coverage - 1.0).abs() <= 1e-9);
    assert_eq!(case.expected_call, bijux_dna_domain_bam::metrics::SexConfidenceClass::Male);
    assert!((case.expected_confidence - 0.9).abs() <= 1e-9);
    assert_eq!(case.expected_status, "ok");
    assert_eq!(
        case.plan.out_dir,
        PathBuf::from("target/local-smoke/bam.sex/core-v1-sex-xy-autosome-male/rxy")
    );
    assert_eq!(
        case.plan.params["bam"],
        serde_json::json!("assets/toy/core-v1/bam/sex_xy_autosome_male.sam")
    );
    assert_eq!(
        case.plan.params["reference"],
        serde_json::json!("assets/toy/core-v1/bam/sex_reference_xy_autosome.fasta")
    );
    assert_eq!(case.plan.params["chromosome_system"], serde_json::json!("xy"));
    assert_eq!(case.plan.params["minimum_y_sites"], serde_json::json!(5));

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
    assert_eq!(output_names, vec!["sex_report", "summary", "stage_metrics"]);

    let summary_output = case
        .plan
        .io
        .outputs
        .iter()
        .find(|artifact| artifact.name.as_str() == "summary")
        .unwrap_or_else(|| panic!("sex summary output missing from BAM sex plan"));
    assert_eq!(
        summary_output.path,
        PathBuf::from(
            "target/local-smoke/bam.sex/core-v1-sex-xy-autosome-male/rxy/sex.summary.json"
        )
    );

    Ok(())
}

#[test]
fn local_sex_smoke_stage_api_surface_stays_callable() {
    let _: fn(
        &Path,
    ) -> anyhow::Result<Vec<bijux_dna_planner_bam::stage_api::LocalSexSmokeCasePlan>> =
        bijux_dna_planner_bam::stage_api::local_sex_smoke_plans;
}

#[test]
fn local_sex_smoke_plans_reject_empty_sample_ids() -> Result<()> {
    let temp = tempfile::tempdir()?;
    write_local_sex_config(
        temp.path(),
        r#"
schema_version = "bijux.bench.bam.local_sex.v1"
tool_id = "rxy"
threads = 2
output_dir = "target/local-smoke/bam.sex"

[[cases]]
sample_id = " "
bam = "assets/toy/core-v1/bam/sex_xy_autosome_male.sam"
reference = "assets/toy/core-v1/bam/sex_reference_xy_autosome.fasta"
chromosome_system = "xy"
minimum_y_sites = 5
expected_method = "rxy"
expected_x_coverage = 0.5
expected_y_coverage = 0.5
expected_autosomal_coverage = 1.0
expected_call = "male"
expected_confidence = 0.9
expected_status = "ok"
"#,
    )?;

    let error = bijux_dna_planner_bam::stage_api::local_sex_smoke_plans(temp.path())
        .expect_err("empty sample_id must be rejected before sex smoke plan construction");
    assert_eq!(error.to_string(), "local-smoke bam.sex sample_id must not be empty");
    Ok(())
}

#[test]
fn local_sex_smoke_plans_reject_duplicate_sample_ids() -> Result<()> {
    let temp = tempfile::tempdir()?;
    write_local_sex_config(
        temp.path(),
        r#"
schema_version = "bijux.bench.bam.local_sex.v1"
tool_id = "rxy"
threads = 2
output_dir = "target/local-smoke/bam.sex"

[[cases]]
sample_id = "duplicate-case"
bam = "assets/toy/core-v1/bam/sex_xy_autosome_male.sam"
reference = "assets/toy/core-v1/bam/sex_reference_xy_autosome.fasta"
chromosome_system = "xy"
minimum_y_sites = 5
expected_method = "rxy"
expected_x_coverage = 0.5
expected_y_coverage = 0.5
expected_autosomal_coverage = 1.0
expected_call = "male"
expected_confidence = 0.9
expected_status = "ok"

[[cases]]
sample_id = "duplicate-case"
bam = "assets/toy/core-v1/bam/sex_xy_autosome_male.sam"
reference = "assets/toy/core-v1/bam/sex_reference_xy_autosome.fasta"
chromosome_system = "xy"
minimum_y_sites = 5
expected_method = "rxy"
expected_x_coverage = 0.5
expected_y_coverage = 0.5
expected_autosomal_coverage = 1.0
expected_call = "male"
expected_confidence = 0.9
expected_status = "ok"
"#,
    )?;

    let error = bijux_dna_planner_bam::stage_api::local_sex_smoke_plans(temp.path())
        .expect_err("duplicate sample_id must be rejected before sex smoke plan construction");
    assert_eq!(error.to_string(), "duplicate local-smoke bam.sex sample_id `duplicate-case`");
    Ok(())
}
