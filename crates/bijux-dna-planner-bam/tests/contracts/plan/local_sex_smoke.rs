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

fn stage_api_temp_repo() -> Result<tempfile::TempDir> {
    let temp = tempfile::tempdir()?;
    let repo_root = repo_root();
    let tool_dir = temp.path().join("domain/bam/tools");
    fs::create_dir_all(&tool_dir)?;
    fs::copy(repo_root.join("domain/bam/tools/rxy.yaml"), tool_dir.join("rxy.yaml"))?;
    Ok(temp)
}

#[test]
fn local_sex_smoke_plans_use_governed_bam_reference_and_expectations() -> Result<()> {
    let repo_root = repo_root();
    let plans = bijux_dna_planner_bam::stage_api::local_sex_smoke_plans(&repo_root)?;
    assert_eq!(plans.len(), 1, "governed local-smoke config must keep exactly one BAM sex case");

    let case = plans
        .iter()
        .find(|case| case.sample_id == "adna_xy_autosome_coverage")
        .unwrap_or_else(|| panic!("governed BAM sex case missing"));
    assert_eq!(case.plan.stage_id.as_str(), "bam.sex");
    assert_eq!(case.plan.tool_id.as_str(), "rxy");
    assert_eq!(case.plan.resources.threads, 2);
    assert_eq!(
        case.bam,
        PathBuf::from(
            "tests/fixtures/corpora/corpus-01-adna-bam-mini/aligned/adna_xy_autosome_coverage.sam"
        )
    );
    assert_eq!(
        case.reference,
        PathBuf::from(
            "tests/fixtures/corpora/corpus-01-adna-bam-mini/reference/adna_bam_reference.fasta"
        )
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
        PathBuf::from("target/local-smoke/bam.sex/adna_xy_autosome_coverage/rxy")
    );
    assert_eq!(
        case.plan.params["bam"],
        serde_json::json!(
            "tests/fixtures/corpora/corpus-01-adna-bam-mini/aligned/adna_xy_autosome_coverage.sam"
        )
    );
    assert_eq!(
        case.plan.params["reference"],
        serde_json::json!(
            "tests/fixtures/corpora/corpus-01-adna-bam-mini/reference/adna_bam_reference.fasta"
        )
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
            "target/local-smoke/bam.sex/adna_xy_autosome_coverage/rxy/sex.summary.json"
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
fn local_sex_smoke_plans_require_expected_method_to_match_governed_tool() -> Result<()> {
    let temp = stage_api_temp_repo()?;
    let repo_root = repo_root();
    write_local_sex_config(
        temp.path(),
        &format!(
            r#"
schema_version = "bijux.bench.bam.local_sex.v1"
tool_id = "rxy"
threads = 2
output_dir = "target/local-smoke/bam.sex"

[[cases]]
sample_id = "wrong-method"
bam = "{bam}"
reference = "{reference}"
chromosome_system = "xy"
minimum_y_sites = 5
expected_method = "angsd"
expected_x_coverage = 0.5
expected_y_coverage = 0.5
expected_autosomal_coverage = 1.0
expected_call = "male"
expected_confidence = 0.9
expected_status = "ok"
"#,
            bam = repo_root.join("tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_xy_autosome_coverage.sam").display(),
            reference = repo_root
                .join("tests/fixtures/corpora/corpus-01-bam-mini/reference/corpus_01_bam_reference.fasta")
                .display(),
        ),
    )?;

    let error = bijux_dna_planner_bam::stage_api::local_sex_smoke_plans(temp.path())
        .expect_err("expected_method must stay aligned with the governed sex tool");
    assert_eq!(
        error.to_string(),
        "local-smoke bam.sex case `wrong-method` must keep expected_method aligned with the governed sex tool"
    );
    Ok(())
}

#[test]
fn local_sex_smoke_plans_require_expected_call_to_match_governed_summary() -> Result<()> {
    let temp = stage_api_temp_repo()?;
    let repo_root = repo_root();
    write_local_sex_config(
        temp.path(),
        &format!(
            r#"
schema_version = "bijux.bench.bam.local_sex.v1"
tool_id = "rxy"
threads = 2
output_dir = "target/local-smoke/bam.sex"

[[cases]]
sample_id = "wrong-call"
bam = "{bam}"
reference = "{reference}"
chromosome_system = "xy"
minimum_y_sites = 5
expected_method = "rxy"
expected_x_coverage = 0.5
expected_y_coverage = 0.5
expected_autosomal_coverage = 1.0
expected_call = "female"
expected_confidence = 0.9
expected_status = "ok"
"#,
            bam = repo_root.join("tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_xy_autosome_coverage.sam").display(),
            reference = repo_root
                .join("tests/fixtures/corpora/corpus-01-bam-mini/reference/corpus_01_bam_reference.fasta")
                .display(),
        ),
    )?;

    let error = bijux_dna_planner_bam::stage_api::local_sex_smoke_plans(temp.path())
        .expect_err("expected_call must stay aligned with the governed sex summary");
    assert_eq!(
        error.to_string(),
        "local-smoke bam.sex case `wrong-call` must keep expected_call aligned with the governed sex summary"
    );
    Ok(())
}

#[test]
fn local_sex_smoke_plans_require_expected_x_coverage_to_match_governed_summary() -> Result<()> {
    let temp = stage_api_temp_repo()?;
    let repo_root = repo_root();
    write_local_sex_config(
        temp.path(),
        &format!(
            r#"
schema_version = "bijux.bench.bam.local_sex.v1"
tool_id = "rxy"
threads = 2
output_dir = "target/local-smoke/bam.sex"

[[cases]]
sample_id = "wrong-x-coverage"
bam = "{bam}"
reference = "{reference}"
chromosome_system = "xy"
minimum_y_sites = 5
expected_method = "rxy"
expected_x_coverage = 0.6
expected_y_coverage = 0.5
expected_autosomal_coverage = 1.0
expected_call = "male"
expected_confidence = 0.9
expected_status = "ok"
"#,
            bam = repo_root.join("tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_xy_autosome_coverage.sam").display(),
            reference = repo_root
                .join("tests/fixtures/corpora/corpus-01-bam-mini/reference/corpus_01_bam_reference.fasta")
                .display(),
        ),
    )?;

    let error = bijux_dna_planner_bam::stage_api::local_sex_smoke_plans(temp.path())
        .expect_err("expected_x_coverage must stay aligned with the governed sex summary");
    assert_eq!(
        error.to_string(),
        "local-smoke bam.sex case `wrong-x-coverage` must keep expected_x_coverage aligned with the governed sex summary"
    );
    Ok(())
}

#[test]
fn local_sex_smoke_plans_require_expected_status_to_match_governed_summary() -> Result<()> {
    let temp = stage_api_temp_repo()?;
    let repo_root = repo_root();
    write_local_sex_config(
        temp.path(),
        &format!(
            r#"
schema_version = "bijux.bench.bam.local_sex.v1"
tool_id = "rxy"
threads = 2
output_dir = "target/local-smoke/bam.sex"

[[cases]]
sample_id = "wrong-status"
bam = "{bam}"
reference = "{reference}"
chromosome_system = "xy"
minimum_y_sites = 5
expected_method = "rxy"
expected_x_coverage = 0.5
expected_y_coverage = 0.5
expected_autosomal_coverage = 1.0
expected_call = "male"
expected_confidence = 0.9
expected_status = "insufficient_coverage"
"#,
            bam = repo_root.join("tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_xy_autosome_coverage.sam").display(),
            reference = repo_root
                .join("tests/fixtures/corpora/corpus-01-bam-mini/reference/corpus_01_bam_reference.fasta")
                .display(),
        ),
    )?;

    let error = bijux_dna_planner_bam::stage_api::local_sex_smoke_plans(temp.path())
        .expect_err("expected_status must stay aligned with the governed sex summary");
    assert_eq!(
        error.to_string(),
        "local-smoke bam.sex case `wrong-status` must keep expected_status aligned with the governed sex summary"
    );
    Ok(())
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
bam = "tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_xy_autosome_coverage.sam"
reference = "tests/fixtures/corpora/corpus-01-bam-mini/reference/corpus_01_bam_reference.fasta"
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
bam = "tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_xy_autosome_coverage.sam"
reference = "tests/fixtures/corpora/corpus-01-bam-mini/reference/corpus_01_bam_reference.fasta"
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
bam = "tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_xy_autosome_coverage.sam"
reference = "tests/fixtures/corpora/corpus-01-bam-mini/reference/corpus_01_bam_reference.fasta"
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
