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
fn local_gc_bias_smoke_plans_use_governed_reference_and_bam() -> Result<()> {
    let repo_root = repo_root();
    let plans = bijux_dna_planner_bam::stage_api::local_gc_bias_smoke_plans(&repo_root)?;
    assert_eq!(
        plans.len(),
        1,
        "governed local-smoke config must keep exactly one BAM gc-bias case"
    );

    let case = plans
        .iter()
        .find(|case| case.sample_id == "human_like_gc_window_ladder")
        .unwrap_or_else(|| panic!("governed BAM gc-bias case missing"));
    assert_eq!(case.plan.stage_id.as_str(), "bam.gc_bias");
    assert_eq!(case.plan.tool_id.as_str(), "picard");
    assert_eq!(case.plan.resources.threads, 2);
    assert_eq!(
        case.bam,
        PathBuf::from(
            "benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_gc_window_ladder.sam"
        )
    );
    assert_eq!(
        case.reference,
        PathBuf::from(
            "benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/reference/human_like_gc_window_ladder.fasta"
        )
    );
    assert_eq!(case.window_size, 10);
    assert_eq!(
        case.expected_rows,
        vec![
            bijux_dna_planner_bam::stage_api::LocalGcBiasSmokeExpectedRow {
                gc_bin: 0,
                normalized_coverage: 0.75,
                windows: 1,
                read_starts: 1,
            },
            bijux_dna_planner_bam::stage_api::LocalGcBiasSmokeExpectedRow {
                gc_bin: 50,
                normalized_coverage: 1.5,
                windows: 1,
                read_starts: 2,
            },
            bijux_dna_planner_bam::stage_api::LocalGcBiasSmokeExpectedRow {
                gc_bin: 100,
                normalized_coverage: 0.75,
                windows: 1,
                read_starts: 1,
            },
        ]
    );
    assert_eq!(
        case.plan.out_dir,
        PathBuf::from("runs/bench/local-smoke/bam.gc_bias/human_like_gc_window_ladder/picard")
    );
    assert_eq!(
        case.plan.params["bam"],
        serde_json::json!(
            "benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_gc_window_ladder.sam"
        )
    );
    assert_eq!(
        case.plan.params["reference"],
        serde_json::json!(
            "benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/reference/human_like_gc_window_ladder.fasta"
        )
    );
    assert_eq!(case.plan.params["window_size"], serde_json::json!(10));

    let output_names = case
        .plan
        .io
        .outputs
        .iter()
        .map(|artifact| artifact.name.as_str().to_string())
        .collect::<Vec<_>>();
    assert_eq!(output_names, vec!["gc_bias_report", "gc_bias_plot", "summary", "stage_metrics"]);

    let summary_output = case
        .plan
        .io
        .outputs
        .iter()
        .find(|artifact| artifact.name.as_str() == "summary")
        .unwrap_or_else(|| panic!("gc-bias summary output missing from BAM gc-bias plan"));
    assert_eq!(
        summary_output.path,
        PathBuf::from(
            "runs/bench/local-smoke/bam.gc_bias/human_like_gc_window_ladder/picard/gc_bias.summary.json"
        )
    );

    Ok(())
}

#[test]
fn local_gc_bias_smoke_stage_api_surface_stays_callable() {
    let _: fn(
        &Path,
    )
        -> anyhow::Result<Vec<bijux_dna_planner_bam::stage_api::LocalGcBiasSmokeCasePlan>> =
        bijux_dna_planner_bam::stage_api::local_gc_bias_smoke_plans;
}

fn write_local_gc_bias_config(root: &Path, body: &str) -> Result<()> {
    let config_dir = root.join("benchmarks/configs/local");
    fs::create_dir_all(&config_dir)?;
    fs::write(config_dir.join("bam-gc-bias.toml"), body)?;
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
fn local_gc_bias_smoke_plans_reject_empty_sample_ids() -> Result<()> {
    let temp = tempfile::tempdir()?;
    write_local_gc_bias_config(
        temp.path(),
        r#"
schema_version = "bijux.bench.bam.local_gc_bias.v1"
tool_id = "picard"

[[cases]]
sample_id = " "
bam = "benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_gc_window_ladder.sam"
reference = "benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/reference/human_like_gc_window_ladder.fasta"
window_size = 10

[[cases.expected_rows]]
gc_bin = 0
normalized_coverage = 0.75
windows = 1
read_starts = 1
"#,
    )?;

    let error = bijux_dna_planner_bam::stage_api::local_gc_bias_smoke_plans(temp.path())
        .expect_err("empty sample_id must be rejected before gc-bias plan construction");
    assert_eq!(error.to_string(), "local-smoke bam.gc_bias sample_id must not be empty");
    Ok(())
}

#[test]
fn local_gc_bias_smoke_plans_reject_duplicate_sample_ids() -> Result<()> {
    let temp = tempfile::tempdir()?;
    write_local_gc_bias_config(
        temp.path(),
        r#"
schema_version = "bijux.bench.bam.local_gc_bias.v1"
tool_id = "picard"

[[cases]]
sample_id = "duplicate-case"
bam = "benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_gc_window_ladder.sam"
reference = "benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/reference/human_like_gc_window_ladder.fasta"
window_size = 10

[[cases.expected_rows]]
gc_bin = 0
normalized_coverage = 0.75
windows = 1
read_starts = 1

[[cases]]
sample_id = "duplicate-case"
bam = "benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_gc_window_ladder.sam"
reference = "benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/reference/human_like_gc_window_ladder.fasta"
window_size = 10

[[cases.expected_rows]]
gc_bin = 50
normalized_coverage = 1.5
windows = 1
read_starts = 2
"#,
    )?;

    let error = bijux_dna_planner_bam::stage_api::local_gc_bias_smoke_plans(temp.path())
        .expect_err("duplicate sample_id must be rejected before gc-bias plan construction");
    assert_eq!(error.to_string(), "duplicate local-smoke bam.gc_bias sample_id `duplicate-case`");
    Ok(())
}

#[test]
fn local_gc_bias_smoke_plans_require_non_zero_window_size() -> Result<()> {
    let temp = stage_api_temp_repo()?;
    let repo_root = repo_root();
    write_local_gc_bias_config(
        temp.path(),
        &format!(
            r#"
schema_version = "bijux.bench.bam.local_gc_bias.v1"
tool_id = "picard"

[[cases]]
sample_id = "zero-window"
bam = "{bam}"
reference = "{reference}"
window_size = 0

[[cases.expected_rows]]
gc_bin = 0
normalized_coverage = 0.75
windows = 1
read_starts = 1
"#,
            bam = repo_root
                .join("benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_gc_window_ladder.sam")
                .display(),
            reference = repo_root
                .join("benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/reference/human_like_gc_window_ladder.fasta")
                .display(),
        ),
    )?;

    let error = bijux_dna_planner_bam::stage_api::local_gc_bias_smoke_plans(temp.path())
        .expect_err("gc-bias cases must declare window_size greater than zero");
    assert_eq!(
        error.to_string(),
        "local-smoke bam.gc_bias case `zero-window` must declare window_size greater than zero"
    );
    Ok(())
}

#[test]
fn local_gc_bias_smoke_plans_require_expected_gc_rows() -> Result<()> {
    let temp = stage_api_temp_repo()?;
    let repo_root = repo_root();
    write_local_gc_bias_config(
        temp.path(),
        &format!(
            r#"
schema_version = "bijux.bench.bam.local_gc_bias.v1"
tool_id = "picard"

[[cases]]
sample_id = "missing-rows"
bam = "{bam}"
reference = "{reference}"
window_size = 10
expected_rows = []
"#,
            bam = repo_root
                .join("benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_gc_window_ladder.sam")
                .display(),
            reference = repo_root
                .join("benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/reference/human_like_gc_window_ladder.fasta")
                .display(),
        ),
    )?;

    let error = bijux_dna_planner_bam::stage_api::local_gc_bias_smoke_plans(temp.path())
        .expect_err("gc-bias cases must declare at least one expected row");
    assert_eq!(
        error.to_string(),
        "local-smoke bam.gc_bias case `missing-rows` must declare at least one expected GC bin row"
    );
    Ok(())
}

#[test]
fn local_gc_bias_smoke_plans_reject_duplicate_expected_gc_bins() -> Result<()> {
    let temp = stage_api_temp_repo()?;
    let repo_root = repo_root();
    write_local_gc_bias_config(
        temp.path(),
        &format!(
            r#"
schema_version = "bijux.bench.bam.local_gc_bias.v1"
tool_id = "picard"

[[cases]]
sample_id = "duplicate-gc-bin"
bam = "{bam}"
reference = "{reference}"
window_size = 10

[[cases.expected_rows]]
gc_bin = 50
normalized_coverage = 1.0
windows = 1
read_starts = 1

[[cases.expected_rows]]
gc_bin = 50
normalized_coverage = 1.2
windows = 1
read_starts = 2
"#,
            bam = repo_root
                .join("benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_gc_window_ladder.sam")
                .display(),
            reference = repo_root
                .join("benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/reference/human_like_gc_window_ladder.fasta")
                .display(),
        ),
    )?;

    let error = bijux_dna_planner_bam::stage_api::local_gc_bias_smoke_plans(temp.path())
        .expect_err("gc-bias rows must not repeat the same GC bin");
    assert_eq!(
        error.to_string(),
        "local-smoke bam.gc_bias case `duplicate-gc-bin` must not repeat expected gc_bin `50`"
    );
    Ok(())
}

#[test]
fn local_gc_bias_smoke_plans_require_positive_expected_windows() -> Result<()> {
    let temp = stage_api_temp_repo()?;
    let repo_root = repo_root();
    write_local_gc_bias_config(
        temp.path(),
        &format!(
            r#"
schema_version = "bijux.bench.bam.local_gc_bias.v1"
tool_id = "picard"

[[cases]]
sample_id = "zero-windows"
bam = "{bam}"
reference = "{reference}"
window_size = 10

[[cases.expected_rows]]
gc_bin = 0
normalized_coverage = 0.75
windows = 0
read_starts = 1
"#,
            bam = repo_root
                .join("benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_gc_window_ladder.sam")
                .display(),
            reference = repo_root
                .join("benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/reference/human_like_gc_window_ladder.fasta")
                .display(),
        ),
    )?;

    let error = bijux_dna_planner_bam::stage_api::local_gc_bias_smoke_plans(temp.path())
        .expect_err("gc-bias rows must declare at least one reference window");
    assert_eq!(
        error.to_string(),
        "local-smoke bam.gc_bias case `zero-windows` must keep expected gc_bin `0` windows greater than zero"
    );
    Ok(())
}

#[test]
fn local_gc_bias_smoke_plans_require_non_negative_normalized_coverage() -> Result<()> {
    let temp = stage_api_temp_repo()?;
    let repo_root = repo_root();
    write_local_gc_bias_config(
        temp.path(),
        &format!(
            r#"
schema_version = "bijux.bench.bam.local_gc_bias.v1"
tool_id = "picard"

[[cases]]
sample_id = "negative-coverage"
bam = "{bam}"
reference = "{reference}"
window_size = 10

[[cases.expected_rows]]
gc_bin = 0
normalized_coverage = -0.1
windows = 1
read_starts = 1
"#,
            bam = repo_root
                .join("benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_gc_window_ladder.sam")
                .display(),
            reference = repo_root
                .join("benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/reference/human_like_gc_window_ladder.fasta")
                .display(),
        ),
    )?;

    let error = bijux_dna_planner_bam::stage_api::local_gc_bias_smoke_plans(temp.path())
        .expect_err("gc-bias rows must keep normalized coverage non-negative");
    assert_eq!(
        error.to_string(),
        "local-smoke bam.gc_bias case `negative-coverage` must keep expected gc_bin `0` normalized_coverage finite and non-negative"
    );
    Ok(())
}
