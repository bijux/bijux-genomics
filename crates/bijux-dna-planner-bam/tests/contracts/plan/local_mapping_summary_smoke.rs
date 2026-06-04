use anyhow::Result;
use bijux_dna_core::prelude::{StageId, ToolId};
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
fn local_mapping_summary_smoke_plans_use_governed_partial_mapping_fixture() -> Result<()> {
    let repo_root = repo_root();
    let plans = bijux_dna_planner_bam::stage_api::local_mapping_summary_smoke_plans(&repo_root)?;
    assert_eq!(
        plans.len(),
        1,
        "governed local-smoke config must keep exactly one BAM mapping summary case"
    );

    let case = plans
        .iter()
        .find(|case| case.sample_id == "human_like_partial_mapping")
        .unwrap_or_else(|| panic!("governed BAM mapping summary case missing"));
    assert_eq!(case.plan.stage_id.as_str(), "bam.mapping_summary");
    assert_eq!(case.plan.tool_id.as_str(), "samtools");
    assert_eq!(case.plan.resources.threads, 4);
    assert_eq!(
        case.bam,
        PathBuf::from(
            "tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_partial_mapping.sam"
        )
    );
    assert_eq!(case.expected_total_reads, 3);
    assert_eq!(case.expected_mapped_reads, 2);
    assert_eq!(case.expected_mapping_fraction, 2.0 / 3.0);
    assert_eq!(case.expected_reference_name, "chr1");
    assert_eq!(
        case.plan.out_dir,
        PathBuf::from("target/local-smoke/bam.mapping_summary/human_like_partial_mapping/samtools")
    );

    let output_names = case
        .plan
        .io
        .outputs
        .iter()
        .map(|artifact| artifact.name.as_str().to_string())
        .collect::<Vec<_>>();
    assert_eq!(output_names, vec!["flagstat", "idxstats", "stats", "summary", "stage_metrics"]);

    let summary_output = case
        .plan
        .io
        .outputs
        .iter()
        .find(|artifact| artifact.name.as_str() == "summary")
        .unwrap_or_else(|| panic!("summary output missing from BAM mapping summary plan"));
    assert_eq!(
        summary_output.path,
        PathBuf::from(
            "target/local-smoke/bam.mapping_summary/human_like_partial_mapping/samtools/mapping.summary.json"
        )
    );

    Ok(())
}

#[test]
fn local_mapping_summary_smoke_stage_api_surface_stays_callable() {
    let _: fn(
        &Path,
    ) -> anyhow::Result<
        Vec<bijux_dna_planner_bam::stage_api::LocalMappingSummarySmokeCasePlan>,
    > = bijux_dna_planner_bam::stage_api::local_mapping_summary_smoke_plans;
}

fn write_local_mapping_summary_config(root: &Path, body: &str) -> Result<()> {
    let config_dir = root.join("configs/bench/local");
    fs::create_dir_all(&config_dir)?;
    fs::write(config_dir.join("bam-mapping-summary.toml"), body)?;
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
fn local_mapping_summary_smoke_plans_reject_empty_sample_ids() -> Result<()> {
    let temp = tempfile::tempdir()?;
    write_local_mapping_summary_config(
        temp.path(),
        r#"
schema_version = "bijux.bench.bam.local_mapping_summary.v1"
tool_id = "samtools"

[[cases]]
sample_id = " "
bam = "assets/toy/core-v1/bam/mapping_summary_partial_mapping.sam"
expected_total_reads = 3
expected_mapped_reads = 2
expected_mapping_fraction = 0.6666666666666666
expected_reference_name = "chr1"
"#,
    )?;

    let error = bijux_dna_planner_bam::stage_api::local_mapping_summary_smoke_plans(temp.path())
        .expect_err("empty sample_id must be rejected before plan construction");
    assert_eq!(error.to_string(), "local-smoke bam.mapping_summary sample_id must not be empty");
    Ok(())
}

#[test]
fn local_mapping_summary_smoke_plans_reject_duplicate_sample_ids() -> Result<()> {
    let temp = tempfile::tempdir()?;
    write_local_mapping_summary_config(
        temp.path(),
        r#"
schema_version = "bijux.bench.bam.local_mapping_summary.v1"
tool_id = "samtools"

[[cases]]
sample_id = "duplicate-case"
bam = "assets/toy/core-v1/bam/mapping_summary_partial_mapping.sam"
expected_total_reads = 3
expected_mapped_reads = 2
expected_mapping_fraction = 0.6666666666666666
expected_reference_name = "chr1"

[[cases]]
sample_id = "duplicate-case"
bam = "assets/toy/core-v1/bam/mapping_summary_partial_mapping.sam"
expected_total_reads = 3
expected_mapped_reads = 2
expected_mapping_fraction = 0.6666666666666666
expected_reference_name = "chr1"
"#,
    )?;

    let error = bijux_dna_planner_bam::stage_api::local_mapping_summary_smoke_plans(temp.path())
        .expect_err("duplicate sample_id must be rejected before plan construction");
    assert_eq!(
        error.to_string(),
        "duplicate local-smoke bam.mapping_summary sample_id `duplicate-case`"
    );
    Ok(())
}

#[test]
fn local_mapping_summary_smoke_plans_require_expected_reference_name() -> Result<()> {
    let temp = stage_api_temp_repo()?;
    let repo_root = repo_root();
    write_local_mapping_summary_config(
        temp.path(),
        &format!(
            r#"
schema_version = "bijux.bench.bam.local_mapping_summary.v1"
tool_id = "samtools"

[[cases]]
sample_id = "missing-reference"
bam = "{bam}"
expected_total_reads = 3
expected_mapped_reads = 2
expected_mapping_fraction = 0.6666666666666666
expected_reference_name = " "
"#,
            bam = repo_root
                .join(
                    "tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_partial_mapping.sam"
                )
                .display(),
        ),
    )?;

    let error = bijux_dna_planner_bam::stage_api::local_mapping_summary_smoke_plans(temp.path())
        .expect_err("mapping_summary cases must declare a non-empty expected reference name");
    assert_eq!(
        error.to_string(),
        "local-smoke bam.mapping_summary case `missing-reference` must declare a non-empty expected reference name"
    );
    Ok(())
}

#[test]
fn local_mapping_summary_smoke_plans_reject_mapped_reads_greater_than_total() -> Result<()> {
    let temp = stage_api_temp_repo()?;
    let repo_root = repo_root();
    write_local_mapping_summary_config(
        temp.path(),
        &format!(
            r#"
schema_version = "bijux.bench.bam.local_mapping_summary.v1"
tool_id = "samtools"

[[cases]]
sample_id = "mapped-over-total"
bam = "{bam}"
expected_total_reads = 3
expected_mapped_reads = 4
expected_mapping_fraction = 1.0
expected_reference_name = "chr1"
"#,
            bam = repo_root
                .join(
                    "tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_partial_mapping.sam"
                )
                .display(),
        ),
    )?;

    let error = bijux_dna_planner_bam::stage_api::local_mapping_summary_smoke_plans(temp.path())
        .expect_err("mapping_summary cases cannot declare mapped reads greater than total reads");
    assert_eq!(
        error.to_string(),
        "local-smoke bam.mapping_summary case `mapped-over-total` cannot declare mapped reads greater than total reads"
    );
    Ok(())
}

#[test]
fn local_mapping_summary_smoke_plans_require_mapping_fraction_alignment() -> Result<()> {
    let temp = stage_api_temp_repo()?;
    let repo_root = repo_root();
    write_local_mapping_summary_config(
        temp.path(),
        &format!(
            r#"
schema_version = "bijux.bench.bam.local_mapping_summary.v1"
tool_id = "samtools"

[[cases]]
sample_id = "fraction-mismatch"
bam = "{bam}"
expected_total_reads = 3
expected_mapped_reads = 2
expected_mapping_fraction = 0.5
expected_reference_name = "chr1"
"#,
            bam = repo_root
                .join(
                    "tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_partial_mapping.sam"
                )
                .display(),
        ),
    )?;

    let error = bijux_dna_planner_bam::stage_api::local_mapping_summary_smoke_plans(temp.path())
        .expect_err("mapping_summary cases must align expected fraction with mapped and total");
    assert_eq!(
        error.to_string(),
        "local-smoke bam.mapping_summary case `fraction-mismatch` must keep expected mapping fraction aligned with mapped and total reads"
    );
    Ok(())
}

#[test]
fn mapping_summary_plan_accepts_picard_governed_planning_contract() -> Result<()> {
    let repo_root = repo_root();
    let stage_id = StageId::new("bam.mapping_summary".to_string());
    let tool_id = ToolId::new("picard");
    let tool_spec = bijux_dna_planner_bam::stage_api::load_bam_domain_tool_planning_spec(
        &repo_root, &stage_id, &tool_id,
    )?;
    let bam = PathBuf::from(
        "tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_partial_mapping.sam",
    );
    let out_dir =
        PathBuf::from("target/local-smoke/bam.mapping_summary/human_like_partial_mapping/picard");
    let plan = bijux_dna_planner_bam::tool_adapters::stages_pre::mapping_summary::plan(
        &tool_spec, &bam, &out_dir,
    )?;

    assert_eq!(plan.stage_id.as_str(), "bam.mapping_summary");
    assert_eq!(plan.tool_id.as_str(), "picard");
    assert_eq!(plan.out_dir, out_dir);

    let stats_output = plan
        .io
        .outputs
        .iter()
        .find(|artifact| artifact.name.as_str() == "stats")
        .unwrap_or_else(|| panic!("stats output missing from picard bam.mapping_summary plan"));
    assert_eq!(
        stats_output.path,
        PathBuf::from(
            "target/local-smoke/bam.mapping_summary/human_like_partial_mapping/picard/alignment_summary.metrics.txt"
        )
    );

    let command = plan.command.template.last().unwrap_or_else(|| {
        panic!("picard bam.mapping_summary command template must contain a shell body")
    });
    assert!(
        command.contains("CollectAlignmentSummaryMetrics")
            && command.contains("BamIndexStats")
            && command.contains("mapping.summary.json")
            && command.contains("alignment_summary.metrics.txt"),
        "picard bam.mapping_summary command must keep the governed alignment-summary and idxstats contract"
    );

    Ok(())
}
