#![allow(clippy::expect_used)]

use anyhow::Result;
use std::fs;
use std::path::{Path, PathBuf};

const GOVERNED_COVERAGE_SAMPLE_ID: &str = "human_like_target_window_coverage";
const GOVERNED_COVERAGE_BAM_PATH: &str =
    "benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_target_window_coverage.sam";
const GOVERNED_COVERAGE_REGIONS_PATH: &str =
    "benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/regions/human_like_target_window_coverage.bed";

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .unwrap_or_else(|| panic!("workspace root"))
        .to_path_buf()
}

#[test]
fn local_coverage_smoke_plans_use_governed_target_windows_fixture() -> Result<()> {
    let repo_root = repo_root();
    let plans = bijux_dna_planner_bam::stage_api::local_coverage_smoke_plans(&repo_root)?;
    assert_eq!(
        plans.len(),
        1,
        "governed local-smoke config must keep exactly one BAM coverage case"
    );

    let case = plans
        .iter()
        .find(|case| case.sample_id == GOVERNED_COVERAGE_SAMPLE_ID)
        .unwrap_or_else(|| panic!("governed BAM coverage case missing"));
    assert_eq!(case.plan.stage_id.as_str(), "bam.coverage");
    assert_eq!(case.plan.tool_id.as_str(), "samtools");
    assert_eq!(case.plan.resources.threads, 4);
    assert_eq!(case.bam, PathBuf::from(GOVERNED_COVERAGE_BAM_PATH));
    assert_eq!(case.regions, PathBuf::from(GOVERNED_COVERAGE_REGIONS_PATH));
    assert_eq!(case.depth_thresholds, vec![1, 5]);
    assert_eq!(case.expected_coverage_regime, "low_pass");
    assert_eq!(case.expected_rows.len(), 2);
    assert_eq!(case.expected_rows[0].region_id, "chr1_window");
    assert_eq!(case.expected_rows[0].contig, "chr1");
    assert_eq!(case.expected_rows[0].start, 1);
    assert_eq!(case.expected_rows[0].end, 6);
    assert_eq!(case.expected_rows[0].length, 6);
    assert!((case.expected_rows[0].mean_depth - (4.0 / 3.0)).abs() <= 1e-9);
    assert!((case.expected_rows[0].breadth_1x - 1.0).abs() <= 1e-9);
    assert_eq!(case.expected_rows[0].covered_bases, 6);
    assert_eq!(
        case.plan.out_dir,
        PathBuf::from(
            "runs/bench/local-smoke/bam.coverage/human_like_target_window_coverage/samtools"
        )
    );
    assert_eq!(case.plan.params["depth_thresholds"], serde_json::json!([1, 5]));
    assert_eq!(case.plan.params["regions"], serde_json::json!(GOVERNED_COVERAGE_REGIONS_PATH));

    let output_names = case
        .plan
        .io
        .outputs
        .iter()
        .map(|artifact| artifact.name.as_str().to_string())
        .collect::<Vec<_>>();
    assert_eq!(output_names, vec!["coverage_summary", "coverage_depth", "stage_metrics"]);

    let depth_output = case
        .plan
        .io
        .outputs
        .iter()
        .find(|artifact| artifact.name.as_str() == "coverage_depth")
        .unwrap_or_else(|| panic!("coverage depth output missing from BAM coverage plan"));
    assert_eq!(
        depth_output.path,
        PathBuf::from(
            "runs/bench/local-smoke/bam.coverage/human_like_target_window_coverage/samtools/coverage.depth.txt"
        )
    );

    Ok(())
}

#[test]
fn local_coverage_smoke_stage_api_surface_stays_callable() {
    let _: fn(
        &Path,
    )
        -> anyhow::Result<Vec<bijux_dna_planner_bam::stage_api::LocalCoverageSmokeCasePlan>> =
        bijux_dna_planner_bam::stage_api::local_coverage_smoke_plans;
}

fn write_local_coverage_config(root: &Path, body: &str) -> Result<()> {
    let config_dir = root.join("benchmarks/configs/local");
    fs::create_dir_all(&config_dir)?;
    fs::write(config_dir.join("bam-coverage.toml"), body)?;
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
fn local_coverage_smoke_plans_reject_empty_sample_ids() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let bam = GOVERNED_COVERAGE_BAM_PATH;
    let regions = GOVERNED_COVERAGE_REGIONS_PATH;
    write_local_coverage_config(
        temp.path(),
        &format!(
            r#"
schema_version = "bijux.bench.bam.local_coverage.v1"
tool_id = "samtools"

[[cases]]
sample_id = " "
bam = "{bam}"
regions = "{regions}"
depth_thresholds = [1, 5]
expected_coverage_regime = "low_pass"

[[cases.expected_rows]]
region_id = "chr1_window"
contig = "chr1"
start = 1
end = 6
length = 6
mean_depth = 1.3333333333333333
breadth_1x = 1.0
covered_bases = 6
"#,
        ),
    )?;

    let error = bijux_dna_planner_bam::stage_api::local_coverage_smoke_plans(temp.path())
        .expect_err("empty sample_id must be rejected before plan construction");
    assert_eq!(error.to_string(), "local-smoke bam.coverage sample_id must not be empty");
    Ok(())
}

#[test]
fn local_coverage_smoke_plans_reject_duplicate_sample_ids() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let bam = GOVERNED_COVERAGE_BAM_PATH;
    let regions = GOVERNED_COVERAGE_REGIONS_PATH;
    write_local_coverage_config(
        temp.path(),
        &format!(
            r#"
schema_version = "bijux.bench.bam.local_coverage.v1"
tool_id = "samtools"

[[cases]]
sample_id = "duplicate-case"
bam = "{bam}"
regions = "{regions}"
depth_thresholds = [1, 5]
expected_coverage_regime = "low_pass"

[[cases.expected_rows]]
region_id = "chr1_window"
contig = "chr1"
start = 1
end = 6
length = 6
mean_depth = 1.3333333333333333
breadth_1x = 1.0
covered_bases = 6

[[cases]]
sample_id = "duplicate-case"
bam = "{bam}"
regions = "{regions}"
depth_thresholds = [1, 5]
expected_coverage_regime = "low_pass"

[[cases.expected_rows]]
region_id = "chr2_window"
contig = "chr2"
start = 2
end = 5
length = 4
mean_depth = 0.75
breadth_1x = 0.75
covered_bases = 3
"#,
        ),
    )?;

    let error = bijux_dna_planner_bam::stage_api::local_coverage_smoke_plans(temp.path())
        .expect_err("duplicate sample_id must be rejected before plan construction");
    assert_eq!(error.to_string(), "duplicate local-smoke bam.coverage sample_id `duplicate-case`");
    Ok(())
}

#[test]
fn local_coverage_smoke_plans_require_depth_thresholds() -> Result<()> {
    let temp = stage_api_temp_repo()?;
    let repo_root = repo_root();
    write_local_coverage_config(
        temp.path(),
        &format!(
            r#"
schema_version = "bijux.bench.bam.local_coverage.v1"
tool_id = "samtools"

[[cases]]
sample_id = "missing-thresholds"
bam = "{bam}"
regions = "{regions}"
depth_thresholds = []
expected_coverage_regime = "low_pass"

[[cases.expected_rows]]
region_id = "chr1_window"
contig = "chr1"
start = 1
end = 6
length = 6
mean_depth = 1.3333333333333333
breadth_1x = 1.0
covered_bases = 6
"#,
            bam = repo_root.join(GOVERNED_COVERAGE_BAM_PATH).display(),
            regions = repo_root.join(GOVERNED_COVERAGE_REGIONS_PATH).display(),
        ),
    )?;

    let error = bijux_dna_planner_bam::stage_api::local_coverage_smoke_plans(temp.path())
        .expect_err("coverage cases must declare at least one depth threshold");
    assert_eq!(
        error.to_string(),
        "local-smoke bam.coverage case `missing-thresholds` must declare at least one depth threshold"
    );
    Ok(())
}

#[test]
fn local_coverage_smoke_plans_reject_zero_depth_thresholds() -> Result<()> {
    let temp = stage_api_temp_repo()?;
    let repo_root = repo_root();
    write_local_coverage_config(
        temp.path(),
        &format!(
            r#"
schema_version = "bijux.bench.bam.local_coverage.v1"
tool_id = "samtools"

[[cases]]
sample_id = "zero-threshold"
bam = "{bam}"
regions = "{regions}"
depth_thresholds = [0, 5]
expected_coverage_regime = "low_pass"

[[cases.expected_rows]]
region_id = "chr1_window"
contig = "chr1"
start = 1
end = 6
length = 6
mean_depth = 1.3333333333333333
breadth_1x = 1.0
covered_bases = 6
"#,
            bam = repo_root.join(GOVERNED_COVERAGE_BAM_PATH).display(),
            regions = repo_root.join(GOVERNED_COVERAGE_REGIONS_PATH).display(),
        ),
    )?;

    let error = bijux_dna_planner_bam::stage_api::local_coverage_smoke_plans(temp.path())
        .expect_err("coverage thresholds must stay greater than zero");
    assert_eq!(
        error.to_string(),
        "local-smoke bam.coverage case `zero-threshold` must keep depth thresholds greater than zero"
    );
    Ok(())
}

#[test]
fn local_coverage_smoke_plans_require_strictly_increasing_depth_thresholds() -> Result<()> {
    let temp = stage_api_temp_repo()?;
    let repo_root = repo_root();
    write_local_coverage_config(
        temp.path(),
        &format!(
            r#"
schema_version = "bijux.bench.bam.local_coverage.v1"
tool_id = "samtools"

[[cases]]
sample_id = "unordered-thresholds"
bam = "{bam}"
regions = "{regions}"
depth_thresholds = [5, 5]
expected_coverage_regime = "low_pass"

[[cases.expected_rows]]
region_id = "chr1_window"
contig = "chr1"
start = 1
end = 6
length = 6
mean_depth = 1.3333333333333333
breadth_1x = 1.0
covered_bases = 6
"#,
            bam = repo_root.join(GOVERNED_COVERAGE_BAM_PATH).display(),
            regions = repo_root.join(GOVERNED_COVERAGE_REGIONS_PATH).display(),
        ),
    )?;

    let error = bijux_dna_planner_bam::stage_api::local_coverage_smoke_plans(temp.path())
        .expect_err("coverage thresholds must be strictly increasing");
    assert_eq!(
        error.to_string(),
        "local-smoke bam.coverage case `unordered-thresholds` must keep depth thresholds strictly increasing"
    );
    Ok(())
}

#[test]
fn local_coverage_smoke_plans_require_non_empty_expected_coverage_regime() -> Result<()> {
    let temp = stage_api_temp_repo()?;
    let repo_root = repo_root();
    write_local_coverage_config(
        temp.path(),
        &format!(
            r#"
schema_version = "bijux.bench.bam.local_coverage.v1"
tool_id = "samtools"

[[cases]]
sample_id = "empty-regime"
bam = "{bam}"
regions = "{regions}"
depth_thresholds = [1, 5]
expected_coverage_regime = " "

[[cases.expected_rows]]
region_id = "chr1_window"
contig = "chr1"
start = 1
end = 6
length = 6
mean_depth = 1.3333333333333333
breadth_1x = 1.0
covered_bases = 6
"#,
            bam = repo_root.join(GOVERNED_COVERAGE_BAM_PATH).display(),
            regions = repo_root.join(GOVERNED_COVERAGE_REGIONS_PATH).display(),
        ),
    )?;

    let error = bijux_dna_planner_bam::stage_api::local_coverage_smoke_plans(temp.path())
        .expect_err("coverage cases must declare a non-empty regime");
    assert_eq!(
        error.to_string(),
        "local-smoke bam.coverage case `empty-regime` must declare a non-empty expected coverage regime"
    );
    Ok(())
}

#[test]
fn local_coverage_smoke_plans_reject_empty_region_identifiers() -> Result<()> {
    let temp = stage_api_temp_repo()?;
    let repo_root = repo_root();
    write_local_coverage_config(
        temp.path(),
        &format!(
            r#"
schema_version = "bijux.bench.bam.local_coverage.v1"
tool_id = "samtools"

[[cases]]
sample_id = "empty-region-id"
bam = "{bam}"
regions = "{regions}"
depth_thresholds = [1, 5]
expected_coverage_regime = "low_pass"

[[cases.expected_rows]]
region_id = " "
contig = "chr1"
start = 1
end = 6
length = 6
mean_depth = 1.3333333333333333
breadth_1x = 1.0
covered_bases = 6
"#,
            bam = repo_root.join(GOVERNED_COVERAGE_BAM_PATH).display(),
            regions = repo_root.join(GOVERNED_COVERAGE_REGIONS_PATH).display(),
        ),
    )?;

    let error = bijux_dna_planner_bam::stage_api::local_coverage_smoke_plans(temp.path())
        .expect_err("coverage cases must not declare empty region identifiers");
    assert_eq!(
        error.to_string(),
        "local-smoke bam.coverage case `empty-region-id` must not declare empty region identifiers"
    );
    Ok(())
}

#[test]
fn local_coverage_smoke_plans_reject_duplicate_region_ids() -> Result<()> {
    let temp = stage_api_temp_repo()?;
    let repo_root = repo_root();
    write_local_coverage_config(
        temp.path(),
        &format!(
            r#"
schema_version = "bijux.bench.bam.local_coverage.v1"
tool_id = "samtools"

[[cases]]
sample_id = "duplicate-region"
bam = "{bam}"
regions = "{regions}"
depth_thresholds = [1, 5]
expected_coverage_regime = "low_pass"

[[cases.expected_rows]]
region_id = "chr1_window"
contig = "chr1"
start = 1
end = 6
length = 6
mean_depth = 1.3333333333333333
breadth_1x = 1.0
covered_bases = 6

[[cases.expected_rows]]
region_id = "chr1_window"
contig = "chr2"
start = 2
end = 5
length = 4
mean_depth = 0.75
breadth_1x = 0.75
covered_bases = 3
"#,
            bam = repo_root.join(GOVERNED_COVERAGE_BAM_PATH).display(),
            regions = repo_root.join(GOVERNED_COVERAGE_REGIONS_PATH).display(),
        ),
    )?;

    let error = bijux_dna_planner_bam::stage_api::local_coverage_smoke_plans(temp.path())
        .expect_err("coverage cases cannot repeat expected region identifiers");
    assert_eq!(
        error.to_string(),
        "local-smoke bam.coverage case `duplicate-region` declared duplicate region `chr1_window`"
    );
    Ok(())
}

#[test]
fn local_coverage_smoke_plans_reject_misaligned_row_lengths() -> Result<()> {
    let temp = stage_api_temp_repo()?;
    let repo_root = repo_root();
    write_local_coverage_config(
        temp.path(),
        &format!(
            r#"
schema_version = "bijux.bench.bam.local_coverage.v1"
tool_id = "samtools"

[[cases]]
sample_id = "length-mismatch"
bam = "{bam}"
regions = "{regions}"
depth_thresholds = [1, 5]
expected_coverage_regime = "low_pass"

[[cases.expected_rows]]
region_id = "chr1_window"
contig = "chr1"
start = 1
end = 6
length = 5
mean_depth = 1.3333333333333333
breadth_1x = 1.0
covered_bases = 6
"#,
            bam = repo_root.join(GOVERNED_COVERAGE_BAM_PATH).display(),
            regions = repo_root.join(GOVERNED_COVERAGE_REGIONS_PATH).display(),
        ),
    )?;

    let error = bijux_dna_planner_bam::stage_api::local_coverage_smoke_plans(temp.path())
        .expect_err("expected row length must align with coordinates");
    assert_eq!(
        error.to_string(),
        "local-smoke bam.coverage case `length-mismatch` must keep expected row length aligned with region coordinates"
    );
    Ok(())
}

#[test]
fn local_coverage_smoke_plans_reject_covered_bases_greater_than_length() -> Result<()> {
    let temp = stage_api_temp_repo()?;
    let repo_root = repo_root();
    write_local_coverage_config(
        temp.path(),
        &format!(
            r#"
schema_version = "bijux.bench.bam.local_coverage.v1"
tool_id = "samtools"

[[cases]]
sample_id = "covered-bases-over-length"
bam = "{bam}"
regions = "{regions}"
depth_thresholds = [1, 5]
expected_coverage_regime = "low_pass"

[[cases.expected_rows]]
region_id = "chr1_window"
contig = "chr1"
start = 1
end = 6
length = 6
mean_depth = 1.3333333333333333
breadth_1x = 1.0
covered_bases = 7
"#,
            bam = repo_root.join(GOVERNED_COVERAGE_BAM_PATH).display(),
            regions = repo_root.join(GOVERNED_COVERAGE_REGIONS_PATH).display(),
        ),
    )?;

    let error = bijux_dna_planner_bam::stage_api::local_coverage_smoke_plans(temp.path())
        .expect_err("covered bases cannot exceed region length");
    assert_eq!(
        error.to_string(),
        "local-smoke bam.coverage case `covered-bases-over-length` cannot declare covered bases greater than region length"
    );
    Ok(())
}
