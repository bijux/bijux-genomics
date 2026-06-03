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
        .find(|case| case.sample_id == "core-v1-gc-window-ladder")
        .unwrap_or_else(|| panic!("governed BAM gc-bias case missing"));
    assert_eq!(case.plan.stage_id.as_str(), "bam.gc_bias");
    assert_eq!(case.plan.tool_id.as_str(), "picard");
    assert_eq!(case.plan.resources.threads, 2);
    assert_eq!(
        case.bam,
        PathBuf::from("assets/toy/core-v1/bam/gc_bias_window_reads.sam")
    );
    assert_eq!(
        case.reference,
        PathBuf::from("assets/toy/core-v1/bam/gc_bias_reference_windows.fasta")
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
        PathBuf::from("target/local-smoke/bam.gc_bias/core-v1-gc-window-ladder/picard")
    );
    assert_eq!(
        case.plan.params["bam"],
        serde_json::json!("assets/toy/core-v1/bam/gc_bias_window_reads.sam")
    );
    assert_eq!(
        case.plan.params["reference"],
        serde_json::json!("assets/toy/core-v1/bam/gc_bias_reference_windows.fasta")
    );

    let output_names = case
        .plan
        .io
        .outputs
        .iter()
        .map(|artifact| artifact.name.as_str().to_string())
        .collect::<Vec<_>>();
    assert_eq!(
        output_names,
        vec!["gc_bias_report", "gc_bias_plot", "summary", "stage_metrics"]
    );

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
            "target/local-smoke/bam.gc_bias/core-v1-gc-window-ladder/picard/gc_bias.summary.json"
        )
    );

    Ok(())
}

#[test]
fn local_gc_bias_smoke_stage_api_surface_stays_callable() {
    let _: fn(
        &Path,
    ) -> anyhow::Result<Vec<bijux_dna_planner_bam::stage_api::LocalGcBiasSmokeCasePlan>> =
        bijux_dna_planner_bam::stage_api::local_gc_bias_smoke_plans;
}

fn write_local_gc_bias_config(root: &Path, body: &str) -> Result<()> {
    let config_dir = root.join("configs/bench/local");
    fs::create_dir_all(&config_dir)?;
    fs::write(config_dir.join("bam-gc-bias.toml"), body)?;
    Ok(())
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
bam = "assets/toy/core-v1/bam/gc_bias_window_reads.sam"
reference = "assets/toy/core-v1/bam/gc_bias_reference_windows.fasta"
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
bam = "assets/toy/core-v1/bam/gc_bias_window_reads.sam"
reference = "assets/toy/core-v1/bam/gc_bias_reference_windows.fasta"
window_size = 10

[[cases.expected_rows]]
gc_bin = 0
normalized_coverage = 0.75
windows = 1
read_starts = 1

[[cases]]
sample_id = "duplicate-case"
bam = "assets/toy/core-v1/bam/gc_bias_window_reads.sam"
reference = "assets/toy/core-v1/bam/gc_bias_reference_windows.fasta"
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
    assert_eq!(
        error.to_string(),
        "duplicate local-smoke bam.gc_bias sample_id `duplicate-case`"
    );
    Ok(())
}
