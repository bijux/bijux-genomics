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
fn local_filter_smoke_plans_use_governed_mixed_constraint_fixture() -> Result<()> {
    let repo_root = repo_root();
    let plans = bijux_dna_planner_bam::stage_api::local_filter_smoke_plans(&repo_root)?;
    assert_eq!(plans.len(), 1, "governed local-smoke config must keep exactly one BAM filter case");

    let case = plans
        .iter()
        .find(|case| case.sample_id == "core-v1-general-filter")
        .unwrap_or_else(|| panic!("governed BAM filter case missing"));
    assert_eq!(case.plan.stage_id.as_str(), "bam.filter");
    assert_eq!(case.plan.tool_id.as_str(), "samtools");
    assert_eq!(case.plan.resources.threads, 4);
    assert_eq!(case.bam, PathBuf::from("assets/toy/core-v1/bam/filter_mixed_constraints.sam"));
    assert_eq!(case.expected_input_reads, 5);
    assert_eq!(case.expected_kept_reads, 1);
    assert_eq!(case.expected_removed_reads, 4);
    assert_eq!(
        case.expected_active_filters,
        vec![
            "mapq_threshold".to_string(),
            "exclude_flags".to_string(),
            "min_length".to_string(),
            "remove_duplicates".to_string(),
        ]
    );
    assert_eq!(
        case.plan.out_dir,
        PathBuf::from("target/local-smoke/bam.filter/core-v1-general-filter/samtools")
    );
    assert_eq!(case.plan.params["mapq_threshold"], serde_json::json!(20));
    assert_eq!(case.plan.params["exclude_flags"], serde_json::json!([4]));
    assert_eq!(case.plan.params["min_length"], serde_json::json!(8));
    assert_eq!(case.plan.params["remove_duplicates"], serde_json::json!(true));

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
        .unwrap_or_else(|| panic!("summary output missing from BAM filter plan"));
    assert_eq!(
        summary_output.path,
        PathBuf::from(
            "target/local-smoke/bam.filter/core-v1-general-filter/samtools/filter.summary.json"
        )
    );

    Ok(())
}

#[test]
fn local_filter_smoke_stage_api_surface_stays_callable() {
    let _: fn(
        &Path,
    )
        -> anyhow::Result<Vec<bijux_dna_planner_bam::stage_api::LocalFilterSmokeCasePlan>> =
        bijux_dna_planner_bam::stage_api::local_filter_smoke_plans;
}

fn write_local_filter_config(root: &Path, body: &str) -> Result<()> {
    let config_dir = root.join("configs/bench/local");
    fs::create_dir_all(&config_dir)?;
    fs::write(config_dir.join("bam-filter.toml"), body)?;
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
fn local_filter_smoke_plans_reject_empty_sample_ids() -> Result<()> {
    let temp = tempfile::tempdir()?;
    write_local_filter_config(
        temp.path(),
        r#"
schema_version = "bijux.bench.bam.local_filter.v1"
tool_id = "samtools"

[[cases]]
sample_id = " "
bam = "assets/toy/core-v1/bam/filter_mixed_constraints.sam"
expected_input_reads = 5
expected_kept_reads = 1
expected_removed_reads = 4
expected_active_filters = ["mapq_threshold", "exclude_flags", "min_length", "remove_duplicates"]
mapq_threshold = 20
include_flags = []
exclude_flags = [4]
min_length = 8
remove_duplicates = true
base_quality_threshold = 20
"#,
    )?;

    let error = bijux_dna_planner_bam::stage_api::local_filter_smoke_plans(temp.path())
        .expect_err("empty sample_id must be rejected before plan construction");
    assert_eq!(error.to_string(), "local-smoke bam.filter sample_id must not be empty");
    Ok(())
}

#[test]
fn local_filter_smoke_plans_reject_duplicate_sample_ids() -> Result<()> {
    let temp = tempfile::tempdir()?;
    write_local_filter_config(
        temp.path(),
        r#"
schema_version = "bijux.bench.bam.local_filter.v1"
tool_id = "samtools"

[[cases]]
sample_id = "duplicate-case"
bam = "assets/toy/core-v1/bam/filter_mixed_constraints.sam"
expected_input_reads = 5
expected_kept_reads = 1
expected_removed_reads = 4
expected_active_filters = ["mapq_threshold", "exclude_flags", "min_length", "remove_duplicates"]
mapq_threshold = 20
include_flags = []
exclude_flags = [4]
min_length = 8
remove_duplicates = true
base_quality_threshold = 20

[[cases]]
sample_id = "duplicate-case"
bam = "assets/toy/core-v1/bam/filter_mixed_constraints.sam"
expected_input_reads = 5
expected_kept_reads = 1
expected_removed_reads = 4
expected_active_filters = ["mapq_threshold", "exclude_flags", "min_length", "remove_duplicates"]
mapq_threshold = 20
include_flags = []
exclude_flags = [4]
min_length = 8
remove_duplicates = true
base_quality_threshold = 20
"#,
    )?;

    let error = bijux_dna_planner_bam::stage_api::local_filter_smoke_plans(temp.path())
        .expect_err("duplicate sample_id must be rejected before plan construction");
    assert_eq!(error.to_string(), "duplicate local-smoke bam.filter sample_id `duplicate-case`");
    Ok(())
}

#[test]
fn local_filter_smoke_plans_require_aligned_removed_read_counts() -> Result<()> {
    let temp = stage_api_temp_repo()?;
    let repo_root = repo_root();
    write_local_filter_config(
        temp.path(),
        &format!(
            r#"
schema_version = "bijux.bench.bam.local_filter.v1"
tool_id = "samtools"

[[cases]]
sample_id = "removed-count-mismatch"
bam = "{bam}"
expected_input_reads = 5
expected_kept_reads = 1
expected_removed_reads = 3
expected_active_filters = ["mapq_threshold", "exclude_flags", "min_length", "remove_duplicates"]
mapq_threshold = 20
include_flags = []
exclude_flags = [4]
min_length = 8
remove_duplicates = true
base_quality_threshold = 20
"#,
            bam = repo_root.join("assets/toy/core-v1/bam/filter_mixed_constraints.sam").display(),
        ),
    )?;

    let error = bijux_dna_planner_bam::stage_api::local_filter_smoke_plans(temp.path())
        .expect_err("filter smoke removed reads must align with input and kept reads");
    assert_eq!(
        error.to_string(),
        "local-smoke bam.filter case `removed-count-mismatch` must keep expected removed reads aligned with input and kept reads"
    );
    Ok(())
}

#[test]
fn local_filter_smoke_plans_require_active_filters() -> Result<()> {
    let temp = stage_api_temp_repo()?;
    let repo_root = repo_root();
    write_local_filter_config(
        temp.path(),
        &format!(
            r#"
schema_version = "bijux.bench.bam.local_filter.v1"
tool_id = "samtools"

[[cases]]
sample_id = "missing-active-filters"
bam = "{bam}"
expected_input_reads = 5
expected_kept_reads = 1
expected_removed_reads = 4
expected_active_filters = []
mapq_threshold = 20
include_flags = []
exclude_flags = [4]
min_length = 8
remove_duplicates = true
base_quality_threshold = 20
"#,
            bam = repo_root.join("assets/toy/core-v1/bam/filter_mixed_constraints.sam").display(),
        ),
    )?;

    let error = bijux_dna_planner_bam::stage_api::local_filter_smoke_plans(temp.path())
        .expect_err("filter smoke cases must declare at least one active filter");
    assert_eq!(
        error.to_string(),
        "local-smoke bam.filter case `missing-active-filters` must declare at least one active filter"
    );
    Ok(())
}

#[test]
fn local_filter_smoke_plans_reject_empty_active_filter_names() -> Result<()> {
    let temp = stage_api_temp_repo()?;
    let repo_root = repo_root();
    write_local_filter_config(
        temp.path(),
        &format!(
            r#"
schema_version = "bijux.bench.bam.local_filter.v1"
tool_id = "samtools"

[[cases]]
sample_id = "empty-active-filter"
bam = "{bam}"
expected_input_reads = 5
expected_kept_reads = 1
expected_removed_reads = 4
expected_active_filters = ["mapq_threshold", " "]
mapq_threshold = 20
include_flags = []
exclude_flags = [4]
min_length = 8
remove_duplicates = true
base_quality_threshold = 20
"#,
            bam = repo_root.join("assets/toy/core-v1/bam/filter_mixed_constraints.sam").display(),
        ),
    )?;

    let error = bijux_dna_planner_bam::stage_api::local_filter_smoke_plans(temp.path())
        .expect_err("filter smoke cases must not declare empty active filter names");
    assert_eq!(
        error.to_string(),
        "local-smoke bam.filter case `empty-active-filter` must not declare empty active filter names"
    );
    Ok(())
}

#[test]
fn local_filter_smoke_plans_reject_duplicate_active_filters() -> Result<()> {
    let temp = stage_api_temp_repo()?;
    let repo_root = repo_root();
    write_local_filter_config(
        temp.path(),
        &format!(
            r#"
schema_version = "bijux.bench.bam.local_filter.v1"
tool_id = "samtools"

[[cases]]
sample_id = "duplicate-active-filter"
bam = "{bam}"
expected_input_reads = 5
expected_kept_reads = 1
expected_removed_reads = 4
expected_active_filters = ["mapq_threshold", "mapq_threshold"]
mapq_threshold = 20
include_flags = []
exclude_flags = [4]
min_length = 8
remove_duplicates = true
base_quality_threshold = 20
"#,
            bam = repo_root.join("assets/toy/core-v1/bam/filter_mixed_constraints.sam").display(),
        ),
    )?;

    let error = bijux_dna_planner_bam::stage_api::local_filter_smoke_plans(temp.path())
        .expect_err("filter smoke cases must not declare duplicate active filters");
    assert_eq!(
        error.to_string(),
        "local-smoke bam.filter case `duplicate-active-filter` declared duplicate active filter `mapq_threshold`"
    );
    Ok(())
}

#[test]
fn filter_plan_accepts_bamtools_and_bedtools_governed_planning_contracts() -> Result<()> {
    let repo_root = repo_root();
    let stage_id = StageId::new("bam.filter".to_string());
    let bam = PathBuf::from("assets/toy/core-v1/bam/filter_mixed_constraints.sam");

    for (tool, expected_command_fragment) in
        [("bamtools", "bamtools stats -in"), ("bedtools", "bedtools bamtobed -i")]
    {
        let tool_id = ToolId::new(tool);
        let tool_spec = bijux_dna_planner_bam::stage_api::load_bam_domain_tool_planning_spec(
            &repo_root, &stage_id, &tool_id,
        )?;
        let params = bijux_dna_domain_bam::params::FilterEffectiveParams {
            mapq_threshold: 20,
            include_flags: vec![],
            exclude_flags: vec![4],
            min_length: 8,
            remove_duplicates: true,
            base_quality_threshold: 20,
        };
        let out_dir =
            PathBuf::from(format!("target/local-smoke/bam.filter/core-v1-general-filter/{tool}"));
        let plan = bijux_dna_planner_bam::tool_adapters::stages_pre::filter::plan(
            &tool_spec, &bam, &out_dir, &params,
        )?;

        assert_eq!(plan.stage_id.as_str(), "bam.filter");
        assert_eq!(plan.tool_id.as_str(), tool);
        assert_eq!(plan.out_dir, out_dir);
        let summary_output = plan
            .io
            .outputs
            .iter()
            .find(|artifact| artifact.name.as_str() == "summary")
            .unwrap_or_else(|| panic!("summary output missing from {tool} bam.filter plan"));
        assert_eq!(
            summary_output.path,
            PathBuf::from(format!(
                "target/local-smoke/bam.filter/core-v1-general-filter/{tool}/filter.summary.json"
            ))
        );

        let command = plan.command.template.last().unwrap_or_else(|| {
            panic!("{tool} bam.filter command template must contain a shell body")
        });
        assert!(
            command.contains(expected_command_fragment)
                && command.contains("flagstat.before.txt")
                && command.contains("flagstat.after.txt")
                && command.contains("idxstats.before.txt")
                && command.contains("idxstats.after.txt")
                && command.contains("filter.summary.json"),
            "{tool} bam.filter command must preserve the governed audit-artifact contract"
        );
    }

    Ok(())
}
