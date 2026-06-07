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

fn write_local_recalibration_config(root: &Path, body: &str) -> Result<()> {
    let config_dir = root.join("benchmarks/configs/local");
    fs::create_dir_all(&config_dir)?;
    fs::write(config_dir.join("bam-recalibration.toml"), body)?;
    Ok(())
}

fn stage_api_temp_repo() -> Result<tempfile::TempDir> {
    let temp = tempfile::tempdir()?;
    let repo_root = repo_root();
    let tool_dir = temp.path().join("domain/bam/tools");
    fs::create_dir_all(&tool_dir)?;
    fs::copy(repo_root.join("domain/bam/tools/gatk.yaml"), tool_dir.join("gatk.yaml"))?;
    Ok(temp)
}

#[test]
fn local_recalibration_smoke_plans_use_governed_skip_case() -> Result<()> {
    let repo_root = repo_root();
    let plans = bijux_dna_planner_bam::stage_api::local_recalibration_smoke_plans(&repo_root)?;
    assert_eq!(
        plans.len(),
        1,
        "governed local-smoke config must keep exactly one BAM recalibration case"
    );

    let case = plans
        .iter()
        .find(|case| case.sample_id == "human_like_recalibration_low_coverage")
        .unwrap_or_else(|| panic!("governed BAM recalibration case missing"));
    assert_eq!(case.plan.stage_id.as_str(), "bam.recalibration");
    assert_eq!(case.plan.tool_id.as_str(), "gatk");
    assert_eq!(case.plan.resources.threads, 2);
    assert_eq!(
        case.bam,
        PathBuf::from(
            "benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_recalibration_low_coverage.sam"
        )
    );
    assert_eq!(
        case.reference,
        PathBuf::from(
            "benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/reference/corpus_01_bam_reference.fasta"
        )
    );
    assert_eq!(
        case.known_sites,
        vec![PathBuf::from(
            "benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/variants/human_like_recalibration_known_sites.vcf"
        )]
    );
    assert_eq!(case.requested_mode, bijux_dna_domain_bam::params::BqsrMode::Standard);
    assert_eq!(case.effective_mode, bijux_dna_domain_bam::params::BqsrMode::Skip);
    assert_eq!(case.min_mean_coverage, 0.2);
    assert_eq!(case.min_breadth_1x, 0.2);
    assert!((case.observed_mean_coverage - 0.192).abs() <= 1e-9);
    assert!((case.observed_breadth_1x - 0.192).abs() <= 1e-9);
    assert_eq!(case.expected_status, "skipped");
    assert_eq!(case.expected_reason, "coverage_below_gate");
    assert_eq!(
        case.plan.out_dir,
        PathBuf::from(
            "runs/bench/local-smoke/bam.recalibration/human_like_recalibration_low_coverage/gatk"
        )
    );
    assert_eq!(
        case.plan.params["bam"],
        serde_json::json!("benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_recalibration_low_coverage.sam")
    );
    assert_eq!(
        case.plan.params["reference"],
        serde_json::json!(
            "benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/reference/corpus_01_bam_reference.fasta"
        )
    );
    assert_eq!(
        case.plan.params["known_sites"],
        serde_json::json!(["benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/variants/human_like_recalibration_known_sites.vcf"])
    );
    assert_eq!(case.plan.params["requested_mode"], serde_json::json!("standard"));
    assert_eq!(case.plan.params["mode"], serde_json::json!("skip"));
    assert_eq!(case.plan.params["status"], serde_json::json!("skipped"));
    assert_eq!(case.plan.params["decision_reason"], serde_json::json!("coverage_below_gate"));
    assert_eq!(case.plan.params["observed_mean_coverage"], serde_json::json!(0.192));
    assert_eq!(case.plan.params["observed_breadth_1x"], serde_json::json!(0.192));

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
    assert_eq!(
        output_names,
        vec!["recal_bam", "recal_bai", "recal_report", "summary", "stage_metrics"]
    );

    let summary_output = case
        .plan
        .io
        .outputs
        .iter()
        .find(|artifact| artifact.name.as_str() == "summary")
        .unwrap_or_else(|| panic!("recalibration summary output missing from BAM plan"));
    assert_eq!(
        summary_output.path,
        PathBuf::from(
            "runs/bench/local-smoke/bam.recalibration/human_like_recalibration_low_coverage/gatk/recal.summary.json"
        )
    );

    let command = case
        .plan
        .command
        .template
        .get(2)
        .unwrap_or_else(|| panic!("recalibration plan shell command missing"));
    assert!(command.contains("status=skipped"));
    assert!(command.contains("reason=coverage_below_gate"));

    Ok(())
}

#[test]
fn local_recalibration_smoke_stage_api_surface_stays_callable() {
    let _: fn(
        &Path,
    ) -> anyhow::Result<
        Vec<bijux_dna_planner_bam::stage_api::LocalRecalibrationSmokeCasePlan>,
    > = bijux_dna_planner_bam::stage_api::local_recalibration_smoke_plans;
}

#[test]
fn local_recalibration_smoke_plans_reject_empty_sample_ids() -> Result<()> {
    let temp = tempfile::tempdir()?;
    write_local_recalibration_config(
        temp.path(),
        r#"
schema_version = "bijux.bench.bam.local_recalibration.v1"
tool_id = "gatk"
threads = 2
output_dir = "runs/bench/local-smoke/bam.recalibration"

[[cases]]
sample_id = " "
bam = "benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_recalibration_low_coverage.sam"
reference = "benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/reference/corpus_01_bam_reference.fasta"
known_sites = ["benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/variants/human_like_recalibration_known_sites.vcf"]
mode = "standard"
min_mean_coverage = 0.2
min_breadth_1x = 0.2
expected_status = "skipped"
expected_reason = "coverage_below_gate"
"#,
    )?;

    let error = bijux_dna_planner_bam::stage_api::local_recalibration_smoke_plans(temp.path())
        .expect_err("empty sample ids must be rejected");
    assert_eq!(error.to_string(), "local-smoke bam.recalibration sample_id must not be empty");
    Ok(())
}

#[test]
fn local_recalibration_smoke_plans_reject_duplicate_sample_ids() -> Result<()> {
    let temp = tempfile::tempdir()?;
    write_local_recalibration_config(
        temp.path(),
        r#"
schema_version = "bijux.bench.bam.local_recalibration.v1"
tool_id = "gatk"
threads = 2
output_dir = "runs/bench/local-smoke/bam.recalibration"

[[cases]]
sample_id = "duplicate-case"
bam = "benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_recalibration_low_coverage.sam"
reference = "benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/reference/corpus_01_bam_reference.fasta"
known_sites = ["benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/variants/human_like_recalibration_known_sites.vcf"]
mode = "standard"
min_mean_coverage = 0.2
min_breadth_1x = 0.2
expected_status = "skipped"
expected_reason = "coverage_below_gate"

[[cases]]
sample_id = "duplicate-case"
bam = "benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_recalibration_low_coverage.sam"
reference = "benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/reference/corpus_01_bam_reference.fasta"
known_sites = ["benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/variants/human_like_recalibration_known_sites.vcf"]
mode = "standard"
min_mean_coverage = 0.2
min_breadth_1x = 0.2
expected_status = "skipped"
expected_reason = "coverage_below_gate"
"#,
    )?;

    let error = bijux_dna_planner_bam::stage_api::local_recalibration_smoke_plans(temp.path())
        .expect_err("duplicate sample ids must be rejected");
    assert_eq!(
        error.to_string(),
        "duplicate local-smoke bam.recalibration sample_id `duplicate-case`"
    );
    Ok(())
}

#[test]
fn local_recalibration_smoke_plans_require_expected_status_to_match_governed_decision() -> Result<()>
{
    let temp = stage_api_temp_repo()?;
    let repo_root = repo_root();
    write_local_recalibration_config(
        temp.path(),
        &format!(
            r#"
schema_version = "bijux.bench.bam.local_recalibration.v1"
tool_id = "gatk"
threads = 2
output_dir = "runs/bench/local-smoke/bam.recalibration"

[[cases]]
sample_id = "wrong-status"
bam = "{bam}"
reference = "{reference}"
known_sites = ["{known_sites}"]
mode = "standard"
min_mean_coverage = 0.2
min_breadth_1x = 0.2
expected_status = "ready_to_run"
expected_reason = "coverage_below_gate"
"#,
            bam = repo_root
                .join("benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_recalibration_low_coverage.sam")
                .display(),
            reference = repo_root
                .join("benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/reference/corpus_01_bam_reference.fasta")
                .display(),
            known_sites =
                repo_root.join("benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/variants/human_like_recalibration_known_sites.vcf").display(),
        ),
    )?;

    let error = bijux_dna_planner_bam::stage_api::local_recalibration_smoke_plans(temp.path())
        .expect_err("expected_status must stay aligned with the governed recalibration decision");
    assert_eq!(
        error.to_string(),
        "local-smoke bam.recalibration case `wrong-status` must keep expected_status aligned with the governed recalibration decision"
    );
    Ok(())
}

#[test]
fn local_recalibration_smoke_plans_require_expected_reason_to_match_governed_decision() -> Result<()>
{
    let temp = stage_api_temp_repo()?;
    let repo_root = repo_root();
    write_local_recalibration_config(
        temp.path(),
        &format!(
            r#"
schema_version = "bijux.bench.bam.local_recalibration.v1"
tool_id = "gatk"
threads = 2
output_dir = "runs/bench/local-smoke/bam.recalibration"

[[cases]]
sample_id = "wrong-reason"
bam = "{bam}"
reference = "{reference}"
known_sites = ["{known_sites}"]
mode = "standard"
min_mean_coverage = 0.2
min_breadth_1x = 0.2
expected_status = "skipped"
expected_reason = "requested_skip_mode"
"#,
            bam = repo_root
                .join("benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_recalibration_low_coverage.sam")
                .display(),
            reference = repo_root
                .join("benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/reference/corpus_01_bam_reference.fasta")
                .display(),
            known_sites =
                repo_root.join("benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/variants/human_like_recalibration_known_sites.vcf").display(),
        ),
    )?;

    let error = bijux_dna_planner_bam::stage_api::local_recalibration_smoke_plans(temp.path())
        .expect_err("expected_reason must stay aligned with the governed recalibration decision");
    assert_eq!(
        error.to_string(),
        "local-smoke bam.recalibration case `wrong-reason` must keep expected_reason aligned with the governed recalibration decision"
    );
    Ok(())
}
