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
fn local_mapq_filter_smoke_plans_use_governed_threshold_fixture() -> Result<()> {
    let repo_root = repo_root();
    let plans = bijux_dna_planner_bam::stage_api::local_mapq_filter_smoke_plans(&repo_root)?;
    assert_eq!(
        plans.len(),
        1,
        "governed local-smoke config must keep exactly one BAM MAPQ filter case"
    );

    let case = plans
        .iter()
        .find(|case| case.sample_id == "human_like_mapq_threshold_ladder")
        .unwrap_or_else(|| panic!("governed BAM MAPQ filter case missing"));
    assert_eq!(case.plan.stage_id.as_str(), "bam.mapq_filter");
    assert_eq!(case.plan.tool_id.as_str(), "samtools");
    assert_eq!(case.plan.resources.threads, 4);
    assert_eq!(
        case.bam,
        PathBuf::from(
            "benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_mapq_threshold_ladder.sam"
        )
    );
    assert_eq!(case.mapq_threshold, 30);
    assert_eq!(case.expected_input_reads, 4);
    assert_eq!(case.expected_kept_reads, 3);
    assert_eq!(case.expected_removed_reads, 1);
    assert_eq!(case.expected_mapped_reads_removed, 1);
    assert_eq!(case.expected_mapped_fraction_retained, 2.0 / 3.0);
    assert_eq!(
        case.plan.out_dir,
        PathBuf::from(
            "runs/bench/local-smoke/bam.mapq_filter/human_like_mapq_threshold_ladder/samtools"
        )
    );
    assert_eq!(case.plan.params["action"], serde_json::json!("mapq_filter"));
    assert_eq!(case.plan.params["mapq_threshold"], serde_json::json!(30));
    assert_eq!(case.plan.params["include_flags"], serde_json::json!([]));
    assert_eq!(case.plan.params["exclude_flags"], serde_json::json!([]));

    let output_names = case
        .plan
        .io
        .outputs
        .iter()
        .map(|artifact| artifact.name.as_str().to_string())
        .collect::<Vec<_>>();
    assert_eq!(
        output_names,
        vec![
            "filtered_bam",
            "filtered_bai",
            "flagstat_before",
            "flagstat_after",
            "idxstats_before",
            "idxstats_after",
            "summary",
            "stage_metrics",
        ]
    );

    let summary_output = case
        .plan
        .io
        .outputs
        .iter()
        .find(|artifact| artifact.name.as_str() == "summary")
        .unwrap_or_else(|| panic!("summary output missing from BAM MAPQ filter plan"));
    assert_eq!(
        summary_output.path,
        PathBuf::from(
            "runs/bench/local-smoke/bam.mapq_filter/human_like_mapq_threshold_ladder/samtools/mapq_filter.summary.json"
        )
    );

    Ok(())
}

#[test]
fn local_mapq_filter_smoke_stage_api_surface_stays_callable() {
    let _: fn(
        &Path,
    ) -> anyhow::Result<
        Vec<bijux_dna_planner_bam::stage_api::LocalMapqFilterSmokeCasePlan>,
    > = bijux_dna_planner_bam::stage_api::local_mapq_filter_smoke_plans;
}

fn write_local_mapq_filter_config(root: &Path, body: &str) -> Result<()> {
    let config_dir = root.join("benchmarks/configs/local");
    fs::create_dir_all(&config_dir)?;
    fs::write(config_dir.join("bam-mapq-filter.toml"), body)?;
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
fn local_mapq_filter_smoke_plans_reject_empty_sample_ids() -> Result<()> {
    let temp = tempfile::tempdir()?;
    write_local_mapq_filter_config(
        temp.path(),
        r#"
schema_version = "bijux.bench.bam.local_mapq_filter.v1"
tool_id = "samtools"

[[cases]]
sample_id = " "
bam = "benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_mapq_threshold_ladder.sam"
mapq_threshold = 30
expected_input_reads = 4
expected_kept_reads = 3
expected_removed_reads = 1
expected_mapped_reads_removed = 1
expected_mapped_fraction_retained = 0.6666666666666666
"#,
    )?;

    let error = bijux_dna_planner_bam::stage_api::local_mapq_filter_smoke_plans(temp.path())
        .expect_err("empty sample_id must be rejected before plan construction");
    assert_eq!(error.to_string(), "local-smoke bam.mapq_filter sample_id must not be empty");
    Ok(())
}

#[test]
fn local_mapq_filter_smoke_plans_reject_duplicate_sample_ids() -> Result<()> {
    let temp = tempfile::tempdir()?;
    write_local_mapq_filter_config(
        temp.path(),
        r#"
schema_version = "bijux.bench.bam.local_mapq_filter.v1"
tool_id = "samtools"

[[cases]]
sample_id = "duplicate-case"
bam = "benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_mapq_threshold_ladder.sam"
mapq_threshold = 30
expected_input_reads = 4
expected_kept_reads = 3
expected_removed_reads = 1
expected_mapped_reads_removed = 1
expected_mapped_fraction_retained = 0.6666666666666666

[[cases]]
sample_id = "duplicate-case"
bam = "benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_mapq_threshold_ladder.sam"
mapq_threshold = 30
expected_input_reads = 4
expected_kept_reads = 3
expected_removed_reads = 1
expected_mapped_reads_removed = 1
expected_mapped_fraction_retained = 0.6666666666666666
"#,
    )?;

    let error = bijux_dna_planner_bam::stage_api::local_mapq_filter_smoke_plans(temp.path())
        .expect_err("duplicate sample_id must be rejected before plan construction");
    assert_eq!(
        error.to_string(),
        "duplicate local-smoke bam.mapq_filter sample_id `duplicate-case`"
    );
    Ok(())
}

#[test]
fn local_mapq_filter_smoke_plans_require_non_zero_threshold() -> Result<()> {
    let temp = stage_api_temp_repo()?;
    let repo_root = repo_root();
    write_local_mapq_filter_config(
        temp.path(),
        &format!(
            r#"
schema_version = "bijux.bench.bam.local_mapq_filter.v1"
tool_id = "samtools"

[[cases]]
sample_id = "zero-threshold"
bam = "{bam}"
mapq_threshold = 0
expected_input_reads = 4
expected_kept_reads = 3
expected_removed_reads = 1
expected_mapped_reads_removed = 1
expected_mapped_fraction_retained = 0.6666666666666666
"#,
            bam = repo_root
                .join(
                    "benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_mapq_threshold_ladder.sam"
                )
                .display(),
        ),
    )?;

    let error = bijux_dna_planner_bam::stage_api::local_mapq_filter_smoke_plans(temp.path())
        .expect_err("mapq_filter cases must declare a non-zero threshold");
    assert_eq!(
        error.to_string(),
        "local-smoke bam.mapq_filter case `zero-threshold` must declare a non-zero mapq_threshold"
    );
    Ok(())
}

#[test]
fn local_mapq_filter_smoke_plans_reject_kept_reads_greater_than_input() -> Result<()> {
    let temp = stage_api_temp_repo()?;
    let repo_root = repo_root();
    write_local_mapq_filter_config(
        temp.path(),
        &format!(
            r#"
schema_version = "bijux.bench.bam.local_mapq_filter.v1"
tool_id = "samtools"

[[cases]]
sample_id = "kept-over-input"
bam = "{bam}"
mapq_threshold = 30
expected_input_reads = 4
expected_kept_reads = 5
expected_removed_reads = 0
expected_mapped_reads_removed = 0
expected_mapped_fraction_retained = 1.0
"#,
            bam = repo_root
                .join(
                    "benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_mapq_threshold_ladder.sam"
                )
                .display(),
        ),
    )?;

    let error = bijux_dna_planner_bam::stage_api::local_mapq_filter_smoke_plans(temp.path())
        .expect_err("mapq_filter cases cannot keep more reads than they start with");
    assert_eq!(
        error.to_string(),
        "local-smoke bam.mapq_filter case `kept-over-input` cannot declare kept reads greater than input reads"
    );
    Ok(())
}

#[test]
fn local_mapq_filter_smoke_plans_require_aligned_removed_read_counts() -> Result<()> {
    let temp = stage_api_temp_repo()?;
    let repo_root = repo_root();
    write_local_mapq_filter_config(
        temp.path(),
        &format!(
            r#"
schema_version = "bijux.bench.bam.local_mapq_filter.v1"
tool_id = "samtools"

[[cases]]
sample_id = "removed-count-mismatch"
bam = "{bam}"
mapq_threshold = 30
expected_input_reads = 4
expected_kept_reads = 3
expected_removed_reads = 0
expected_mapped_reads_removed = 1
expected_mapped_fraction_retained = 0.6666666666666666
"#,
            bam = repo_root
                .join(
                    "benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_mapq_threshold_ladder.sam"
                )
                .display(),
        ),
    )?;

    let error = bijux_dna_planner_bam::stage_api::local_mapq_filter_smoke_plans(temp.path())
        .expect_err("mapq_filter removed reads must align with input and kept reads");
    assert_eq!(
        error.to_string(),
        "local-smoke bam.mapq_filter case `removed-count-mismatch` must keep expected removed reads aligned with input and kept reads"
    );
    Ok(())
}

#[test]
fn local_mapq_filter_smoke_plans_require_fraction_within_unit_interval() -> Result<()> {
    let temp = stage_api_temp_repo()?;
    let repo_root = repo_root();
    write_local_mapq_filter_config(
        temp.path(),
        &format!(
            r#"
schema_version = "bijux.bench.bam.local_mapq_filter.v1"
tool_id = "samtools"

[[cases]]
sample_id = "fraction-out-of-range"
bam = "{bam}"
mapq_threshold = 30
expected_input_reads = 4
expected_kept_reads = 3
expected_removed_reads = 1
expected_mapped_reads_removed = 1
expected_mapped_fraction_retained = 1.5
"#,
            bam = repo_root
                .join(
                    "benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_mapq_threshold_ladder.sam"
                )
                .display(),
        ),
    )?;

    let error = bijux_dna_planner_bam::stage_api::local_mapq_filter_smoke_plans(temp.path())
        .expect_err("mapq_filter fraction must stay within [0, 1]");
    assert_eq!(
        error.to_string(),
        "local-smoke bam.mapq_filter case `fraction-out-of-range` must declare mapped_fraction_retained within [0, 1]"
    );
    Ok(())
}

#[test]
fn mapq_filter_plan_accepts_bamtools_governed_planning_contract() -> Result<()> {
    let repo_root = repo_root();
    let stage_id = StageId::new("bam.mapq_filter".to_string());
    let tool_id = ToolId::new("bamtools");
    let tool_spec = bijux_dna_planner_bam::stage_api::load_bam_domain_tool_planning_spec(
        &repo_root, &stage_id, &tool_id,
    )?;
    let bam = PathBuf::from(
        "benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_mapq_threshold_ladder.sam",
    );
    let params = bijux_dna_domain_bam::params::FilterEffectiveParams {
        mapq_threshold: 30,
        include_flags: vec![],
        exclude_flags: vec![],
        min_length: 0,
        remove_duplicates: false,
        base_quality_threshold: 20,
    };
    let out_dir = PathBuf::from(
        "runs/bench/local-smoke/bam.mapq_filter/human_like_mapq_threshold_ladder/bamtools",
    );
    let plan = bijux_dna_planner_bam::tool_adapters::stages_pre::mapq_filter::plan(
        &tool_spec, &bam, &out_dir, &params,
    )?;

    assert_eq!(plan.stage_id.as_str(), "bam.mapq_filter");
    assert_eq!(plan.tool_id.as_str(), "bamtools");
    assert_eq!(plan.out_dir, out_dir);
    assert_eq!(plan.params["action"], serde_json::json!("mapq_filter"));
    assert_eq!(plan.params["mapq_threshold"], serde_json::json!(30));
    assert_eq!(plan.params["include_flags"], serde_json::json!([]));
    assert_eq!(plan.params["exclude_flags"], serde_json::json!([]));

    let output_names = plan
        .io
        .outputs
        .iter()
        .map(|artifact| artifact.name.as_str().to_string())
        .collect::<Vec<_>>();
    assert_eq!(
        output_names,
        vec![
            "filtered_bam",
            "filtered_bai",
            "flagstat_before",
            "flagstat_after",
            "idxstats_before",
            "idxstats_after",
            "summary",
            "stage_metrics",
        ]
    );

    let summary_output = plan
        .io
        .outputs
        .iter()
        .find(|artifact| artifact.name.as_str() == "summary")
        .unwrap_or_else(|| panic!("summary output missing from bamtools BAM MAPQ filter plan"));
    assert_eq!(
        summary_output.path,
        PathBuf::from(
            "runs/bench/local-smoke/bam.mapq_filter/human_like_mapq_threshold_ladder/bamtools/mapq_filter.summary.json"
        )
    );

    Ok(())
}
