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
fn local_validate_smoke_plans_use_governed_bam_fixtures() -> Result<()> {
    let repo_root = repo_root();
    let plans = bijux_dna_planner_bam::stage_api::local_validate_smoke_plans(&repo_root)?;
    assert_eq!(plans.len(), 2, "governed local-smoke config must keep pass and refusal coverage");

    let passing = plans
        .iter()
        .find(|case| case.sample_id == "core-v1-coordinate-pass")
        .unwrap_or_else(|| panic!("passing BAM validation case missing"));
    assert_eq!(passing.plan.stage_id.as_str(), "bam.validate");
    assert_eq!(passing.plan.tool_id.as_str(), "samtools");
    assert_eq!(passing.plan.resources.threads, 4);
    assert_eq!(passing.bam, PathBuf::from("assets/toy/core-v1/bam/validation_pass.bam"));
    assert_eq!(
        passing.alignment_fixture_encoding,
        bijux_dna_planner_bam::stage_api::LocalValidateAlignmentFixtureEncoding::BinaryBam
    );
    assert_eq!(
        passing.bam_index,
        Some(PathBuf::from("assets/toy/core-v1/bam/validation_pass.bam.bai"))
    );
    assert_eq!(
        passing.reference_fasta,
        Some(PathBuf::from("assets/toy/core-v1/bam/validation_reference.fasta"))
    );
    assert!(passing.expect_pass);
    assert!(passing.required_refusal_codes.is_empty());
    assert_eq!(
        passing.plan.out_dir,
        PathBuf::from("runs/bench/local-smoke/bam.validate/core-v1-coordinate-pass/samtools")
    );

    let refusal = plans
        .iter()
        .find(|case| case.sample_id == "core-v1-malformed-refusal")
        .unwrap_or_else(|| panic!("refusal BAM validation case missing"));
    assert_eq!(refusal.bam, PathBuf::from("assets/toy/core-v1/bam/validation_malformed.bam"));
    assert_eq!(
        refusal.alignment_fixture_encoding,
        bijux_dna_planner_bam::stage_api::LocalValidateAlignmentFixtureEncoding::BinaryBam
    );
    assert_eq!(refusal.bam_index, None);
    assert_eq!(refusal.reference_fasta, None);
    assert!(!refusal.expect_pass);
    assert_eq!(refusal.required_refusal_codes, vec!["malformed_alignment_record".to_string()]);
    assert_eq!(
        refusal.plan.out_dir,
        PathBuf::from("runs/bench/local-smoke/bam.validate/core-v1-malformed-refusal/samtools")
    );

    let validation_output = refusal
        .plan
        .io
        .outputs
        .iter()
        .find(|artifact| artifact.name.as_str() == "validation_report")
        .unwrap_or_else(|| panic!("validation_report output missing from refusal plan"));
    assert_eq!(
        validation_output.path,
        PathBuf::from(
            "runs/bench/local-smoke/bam.validate/core-v1-malformed-refusal/samtools/validation.json"
        )
    );

    Ok(())
}

#[test]
fn local_validate_smoke_stage_api_surface_stays_callable() {
    let _: fn(
        &Path,
    )
        -> anyhow::Result<Vec<bijux_dna_planner_bam::stage_api::LocalValidateSmokeCasePlan>> =
        bijux_dna_planner_bam::stage_api::local_validate_smoke_plans;
}

fn write_local_validate_config(root: &Path, body: &str) -> Result<()> {
    let config_dir = root.join("benchmarks/configs/local");
    fs::create_dir_all(&config_dir)?;
    fs::write(config_dir.join("bam-validate.toml"), body)?;
    Ok(())
}

fn stage_api_temp_repo() -> Result<tempfile::TempDir> {
    let temp = tempfile::tempdir()?;
    let repo_root = repo_root();
    let tool_dir = temp.path().join("domain/bam/tools");
    fs::create_dir_all(&tool_dir)?;
    fs::copy(repo_root.join("domain/bam/tools/samtools.yaml"), tool_dir.join("samtools.yaml"))?;
    Ok(temp)
}

#[test]
fn local_validate_smoke_plans_reject_empty_sample_ids() -> Result<()> {
    let temp = tempfile::tempdir()?;
    write_local_validate_config(
        temp.path(),
        r#"
schema_version = "bijux.bench.bam.local_validate.v1"
tool_id = "samtools"

[[cases]]
sample_id = " "
bam = "assets/toy/core-v1/bam/validation_malformed.bam"
alignment_fixture_encoding = "binary_bam"
expect_pass = false
required_refusal_codes = ["malformed_alignment_record"]
"#,
    )?;

    let error = bijux_dna_planner_bam::stage_api::local_validate_smoke_plans(temp.path())
        .expect_err("empty sample_id must be rejected before plan construction");
    assert_eq!(error.to_string(), "local-smoke bam.validate sample_id must not be empty");
    Ok(())
}

#[test]
fn local_validate_smoke_plans_reject_duplicate_sample_ids() -> Result<()> {
    let temp = tempfile::tempdir()?;
    write_local_validate_config(
        temp.path(),
        r#"
schema_version = "bijux.bench.bam.local_validate.v1"
tool_id = "samtools"

[[cases]]
sample_id = "duplicate-case"
bam = "assets/toy/core-v1/bam/validation_pass.bam"
alignment_fixture_encoding = "binary_bam"
bam_index = "assets/toy/core-v1/bam/validation_pass.bam.bai"
reference_fasta = "assets/toy/core-v1/bam/validation_reference.fasta"
expect_pass = true

[[cases]]
sample_id = "duplicate-case"
bam = "assets/toy/core-v1/bam/validation_malformed.bam"
alignment_fixture_encoding = "binary_bam"
expect_pass = false
required_refusal_codes = ["malformed_alignment_record"]
"#,
    )?;

    let error = bijux_dna_planner_bam::stage_api::local_validate_smoke_plans(temp.path())
        .expect_err("duplicate sample_id must be rejected before plan construction");
    assert_eq!(error.to_string(), "duplicate local-smoke bam.validate sample_id `duplicate-case`");
    Ok(())
}

#[test]
fn local_validate_smoke_plans_reject_refusal_expectations_for_passing_cases() -> Result<()> {
    let temp = stage_api_temp_repo()?;
    let repo_root = repo_root();
    write_local_validate_config(
        temp.path(),
        &format!(
            r#"
schema_version = "bijux.bench.bam.local_validate.v1"
tool_id = "samtools"

[[cases]]
sample_id = "passing-case"
bam = "{bam}"
alignment_fixture_encoding = "binary_bam"
bam_index = "{bam_index}"
reference_fasta = "{reference_fasta}"
expect_pass = true
required_refusal_codes = ["malformed_alignment_record"]
"#,
            bam = repo_root.join("assets/toy/core-v1/bam/validation_pass.bam").display(),
            bam_index = repo_root.join("assets/toy/core-v1/bam/validation_pass.bam.bai").display(),
            reference_fasta =
                repo_root.join("assets/toy/core-v1/bam/validation_reference.fasta").display(),
        ),
    )?;

    let error = bijux_dna_planner_bam::stage_api::local_validate_smoke_plans(temp.path())
        .expect_err("passing cases must not declare refusal expectations");
    assert_eq!(
        error.to_string(),
        "local-smoke bam.validate passing case `passing-case` must not declare refusal expectations"
    );
    Ok(())
}

#[test]
fn local_validate_smoke_plans_require_refusal_expectations_for_failing_cases() -> Result<()> {
    let temp = stage_api_temp_repo()?;
    let repo_root = repo_root();
    write_local_validate_config(
        temp.path(),
        &format!(
            r#"
schema_version = "bijux.bench.bam.local_validate.v1"
tool_id = "samtools"

[[cases]]
sample_id = "failing-case"
bam = "{bam}"
alignment_fixture_encoding = "binary_bam"
expect_pass = false
"#,
            bam = repo_root.join("assets/toy/core-v1/bam/validation_malformed.bam").display(),
        ),
    )?;

    let error = bijux_dna_planner_bam::stage_api::local_validate_smoke_plans(temp.path())
        .expect_err("failing cases must declare at least one refusal expectation");
    assert_eq!(
        error.to_string(),
        "local-smoke bam.validate refusal case `failing-case` must declare at least one expected refusal code"
    );
    Ok(())
}
