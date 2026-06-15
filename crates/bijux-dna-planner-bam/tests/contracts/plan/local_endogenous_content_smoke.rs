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

fn governed_endogenous_bam(repo_root: &Path) -> PathBuf {
    repo_root.join(
        "benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_endogenous_partial_mapping.sam",
    )
}

#[test]
fn local_endogenous_content_smoke_plans_use_governed_bam_and_host_scope() -> Result<()> {
    let repo_root = repo_root();
    let plans = bijux_dna_planner_bam::stage_api::local_endogenous_content_smoke_plans(&repo_root)?;
    assert_eq!(
        plans.len(),
        1,
        "governed local-smoke config must keep exactly one BAM endogenous-content case"
    );

    let case = plans
        .iter()
        .find(|case| case.sample_id == "human_like_endogenous_partial_mapping")
        .unwrap_or_else(|| panic!("governed BAM endogenous-content case missing"));
    assert_eq!(case.plan.stage_id.as_str(), "bam.endogenous_content");
    assert_eq!(case.plan.tool_id.as_str(), "samtools");
    assert_eq!(case.plan.resources.threads, 2);
    assert_eq!(
        case.bam,
        PathBuf::from(
            "benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_endogenous_partial_mapping.sam"
        )
    );
    assert_eq!(case.host_reference_scope, "human_host");
    assert_eq!(case.expected_total_reads, 5);
    assert_eq!(case.expected_mapped_reads, 3);
    assert!((case.expected_endogenous_fraction - 0.6).abs() <= 1e-9);
    assert_eq!(case.expected_method, "mapped_fraction_from_flagstat");
    assert_eq!(
        case.plan.out_dir,
        PathBuf::from(
            "runs/bench/local-smoke/bam.endogenous_content/human_like_endogenous_partial_mapping/samtools"
        )
    );
    assert_eq!(
        case.plan.params["bam"],
        serde_json::json!(
            "benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_endogenous_partial_mapping.sam"
        )
    );
    assert_eq!(case.plan.params["host_reference_scope"], serde_json::json!("human_host"));

    let output_names = case
        .plan
        .io
        .outputs
        .iter()
        .map(|artifact| artifact.name.as_str().to_string())
        .collect::<Vec<_>>();
    assert_eq!(output_names, vec!["endogenous_report", "summary", "stage_metrics"]);

    let report_output = case
        .plan
        .io
        .outputs
        .iter()
        .find(|artifact| artifact.name.as_str() == "endogenous_report")
        .unwrap_or_else(|| panic!("endogenous-content report output missing from BAM plan"));
    assert_eq!(
        report_output.path,
        PathBuf::from(
            "runs/bench/local-smoke/bam.endogenous_content/human_like_endogenous_partial_mapping/samtools/endogenous.content.json"
        )
    );

    Ok(())
}

#[test]
fn local_endogenous_content_smoke_stage_api_surface_stays_callable() {
    let _: fn(
        &Path,
    ) -> anyhow::Result<
        Vec<bijux_dna_planner_bam::stage_api::LocalEndogenousContentSmokeCasePlan>,
    > = bijux_dna_planner_bam::stage_api::local_endogenous_content_smoke_plans;
}

fn write_local_endogenous_content_config(root: &Path, body: &str) -> Result<()> {
    let config_dir = root.join("benchmarks/configs/local");
    fs::create_dir_all(&config_dir)?;
    fs::write(config_dir.join("bam-endogenous-content.toml"), body)?;
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
fn local_endogenous_content_smoke_plans_reject_empty_sample_ids() -> Result<()> {
    let temp = tempfile::tempdir()?;
    write_local_endogenous_content_config(
        temp.path(),
        r#"
schema_version = "bijux.bench.bam.local_endogenous_content.v1"
tool_id = "samtools"

[[cases]]
sample_id = " "
bam = "benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_endogenous_partial_mapping.sam"
host_reference_scope = "human_host"
expected_total_reads = 5
expected_mapped_reads = 3
expected_endogenous_fraction = 0.6
expected_method = "mapped_fraction_from_flagstat"
"#,
    )?;

    let error = bijux_dna_planner_bam::stage_api::local_endogenous_content_smoke_plans(temp.path())
        .expect_err("empty sample_id must be rejected before endogenous-content plan construction");
    assert_eq!(error.to_string(), "local-smoke bam.endogenous_content sample_id must not be empty");
    Ok(())
}

#[test]
fn local_endogenous_content_smoke_plans_reject_duplicate_sample_ids() -> Result<()> {
    let temp = tempfile::tempdir()?;
    write_local_endogenous_content_config(
        temp.path(),
        r#"
schema_version = "bijux.bench.bam.local_endogenous_content.v1"
tool_id = "samtools"

[[cases]]
sample_id = "duplicate-case"
bam = "benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_endogenous_partial_mapping.sam"
host_reference_scope = "human_host"
expected_total_reads = 5
expected_mapped_reads = 3
expected_endogenous_fraction = 0.6
expected_method = "mapped_fraction_from_flagstat"

[[cases]]
sample_id = "duplicate-case"
bam = "benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_endogenous_partial_mapping.sam"
host_reference_scope = "human_host"
expected_total_reads = 5
expected_mapped_reads = 3
expected_endogenous_fraction = 0.6
expected_method = "mapped_fraction_from_flagstat"
"#,
    )?;

    let error = bijux_dna_planner_bam::stage_api::local_endogenous_content_smoke_plans(temp.path())
        .expect_err(
            "duplicate sample_id must be rejected before endogenous-content plan construction",
        );
    assert_eq!(
        error.to_string(),
        "duplicate local-smoke bam.endogenous_content sample_id `duplicate-case`"
    );
    Ok(())
}

#[test]
fn local_endogenous_content_smoke_plans_require_non_empty_host_scope() -> Result<()> {
    let temp = stage_api_temp_repo()?;
    let repo_root = repo_root();
    write_local_endogenous_content_config(
        temp.path(),
        &format!(
            r#"
schema_version = "bijux.bench.bam.local_endogenous_content.v1"
tool_id = "samtools"

[[cases]]
sample_id = "blank-host-scope"
bam = "{bam}"
host_reference_scope = " "
expected_total_reads = 5
expected_mapped_reads = 3
expected_endogenous_fraction = 0.6
expected_method = "mapped_fraction_from_flagstat"
"#,
            bam = governed_endogenous_bam(&repo_root).display(),
        ),
    )?;

    let error = bijux_dna_planner_bam::stage_api::local_endogenous_content_smoke_plans(temp.path())
        .expect_err("host_reference_scope must not be blank");
    assert_eq!(
        error.to_string(),
        "local-smoke bam.endogenous_content case `blank-host-scope` must declare a non-empty host_reference_scope"
    );
    Ok(())
}

#[test]
fn local_endogenous_content_smoke_plans_require_positive_total_reads() -> Result<()> {
    let temp = stage_api_temp_repo()?;
    let repo_root = repo_root();
    write_local_endogenous_content_config(
        temp.path(),
        &format!(
            r#"
schema_version = "bijux.bench.bam.local_endogenous_content.v1"
tool_id = "samtools"

[[cases]]
sample_id = "zero-total-reads"
bam = "{bam}"
host_reference_scope = "human_host"
expected_total_reads = 0
expected_mapped_reads = 0
expected_endogenous_fraction = 0.0
expected_method = "mapped_fraction_from_flagstat"
"#,
            bam = governed_endogenous_bam(&repo_root).display(),
        ),
    )?;

    let error = bijux_dna_planner_bam::stage_api::local_endogenous_content_smoke_plans(temp.path())
        .expect_err("endogenous-content cases must declare total reads greater than zero");
    assert_eq!(
        error.to_string(),
        "local-smoke bam.endogenous_content case `zero-total-reads` must declare expected_total_reads greater than zero"
    );
    Ok(())
}

#[test]
fn local_endogenous_content_smoke_plans_reject_mapped_reads_above_total() -> Result<()> {
    let temp = stage_api_temp_repo()?;
    let repo_root = repo_root();
    write_local_endogenous_content_config(
        temp.path(),
        &format!(
            r#"
schema_version = "bijux.bench.bam.local_endogenous_content.v1"
tool_id = "samtools"

[[cases]]
sample_id = "mapped-above-total"
bam = "{bam}"
host_reference_scope = "human_host"
expected_total_reads = 5
expected_mapped_reads = 6
expected_endogenous_fraction = 1.0
expected_method = "mapped_fraction_from_flagstat"
"#,
            bam = governed_endogenous_bam(&repo_root).display(),
        ),
    )?;

    let error = bijux_dna_planner_bam::stage_api::local_endogenous_content_smoke_plans(temp.path())
        .expect_err("mapped reads must not exceed total reads");
    assert_eq!(
        error.to_string(),
        "local-smoke bam.endogenous_content case `mapped-above-total` cannot declare mapped reads greater than total reads"
    );
    Ok(())
}

#[test]
fn local_endogenous_content_smoke_plans_require_fraction_within_unit_interval() -> Result<()> {
    let temp = stage_api_temp_repo()?;
    let repo_root = repo_root();
    write_local_endogenous_content_config(
        temp.path(),
        &format!(
            r#"
schema_version = "bijux.bench.bam.local_endogenous_content.v1"
tool_id = "samtools"

[[cases]]
sample_id = "fraction-out-of-range"
bam = "{bam}"
host_reference_scope = "human_host"
expected_total_reads = 5
expected_mapped_reads = 3
expected_endogenous_fraction = 1.1
expected_method = "mapped_fraction_from_flagstat"
"#,
            bam = governed_endogenous_bam(&repo_root).display(),
        ),
    )?;

    let error = bijux_dna_planner_bam::stage_api::local_endogenous_content_smoke_plans(temp.path())
        .expect_err("endogenous fraction must stay within [0, 1]");
    assert_eq!(
        error.to_string(),
        "local-smoke bam.endogenous_content case `fraction-out-of-range` must keep expected_endogenous_fraction within [0, 1]"
    );
    Ok(())
}

#[test]
fn local_endogenous_content_smoke_plans_require_fraction_to_match_counts() -> Result<()> {
    let temp = stage_api_temp_repo()?;
    let repo_root = repo_root();
    write_local_endogenous_content_config(
        temp.path(),
        &format!(
            r#"
schema_version = "bijux.bench.bam.local_endogenous_content.v1"
tool_id = "samtools"

[[cases]]
sample_id = "fraction-mismatch"
bam = "{bam}"
host_reference_scope = "human_host"
expected_total_reads = 5
expected_mapped_reads = 3
expected_endogenous_fraction = 0.61
expected_method = "mapped_fraction_from_flagstat"
"#,
            bam = governed_endogenous_bam(&repo_root).display(),
        ),
    )?;

    let error = bijux_dna_planner_bam::stage_api::local_endogenous_content_smoke_plans(temp.path())
        .expect_err("governed endogenous fraction must agree with mapped and total reads");
    assert_eq!(
        error.to_string(),
        "local-smoke bam.endogenous_content case `fraction-mismatch` must keep expected_endogenous_fraction aligned with expected_mapped_reads and expected_total_reads"
    );
    Ok(())
}

#[test]
fn local_endogenous_content_smoke_plans_require_non_empty_expected_method() -> Result<()> {
    let temp = stage_api_temp_repo()?;
    let repo_root = repo_root();
    write_local_endogenous_content_config(
        temp.path(),
        &format!(
            r#"
schema_version = "bijux.bench.bam.local_endogenous_content.v1"
tool_id = "samtools"

[[cases]]
sample_id = "blank-method"
bam = "{bam}"
host_reference_scope = "human_host"
expected_total_reads = 5
expected_mapped_reads = 3
expected_endogenous_fraction = 0.6
expected_method = " "
"#,
            bam = governed_endogenous_bam(&repo_root).display(),
        ),
    )?;

    let error = bijux_dna_planner_bam::stage_api::local_endogenous_content_smoke_plans(temp.path())
        .expect_err("expected_method must not be blank");
    assert_eq!(
        error.to_string(),
        "local-smoke bam.endogenous_content case `blank-method` must declare a non-empty expected_method"
    );
    Ok(())
}
