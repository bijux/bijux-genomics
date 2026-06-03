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
fn local_overlap_correction_smoke_plans_use_governed_bam_fixture() -> Result<()> {
    let repo_root = repo_root();
    let plans = bijux_dna_planner_bam::stage_api::local_overlap_correction_smoke_plans(&repo_root)?;
    assert_eq!(
        plans.len(),
        1,
        "governed local-smoke config must keep exactly one BAM overlap-correction case"
    );

    let case = plans
        .iter()
        .find(|case| case.sample_id == "core-v1-paired-overlap")
        .unwrap_or_else(|| panic!("governed BAM overlap-correction case missing"));
    assert_eq!(case.plan.stage_id.as_str(), "bam.overlap_correction");
    assert_eq!(case.plan.tool_id.as_str(), "bamutil");
    assert_eq!(case.plan.resources.threads, 2);
    assert_eq!(
        case.bam,
        PathBuf::from("assets/toy/core-v1/bam/overlap_correction_paired_overlap.sam")
    );
    assert_eq!(case.expected_pair_count, 2);
    assert_eq!(case.expected_corrected_pairs, 1);
    assert_eq!(case.expected_corrected_overlap_bases, 7);
    assert_eq!(
        case.plan.out_dir,
        PathBuf::from("target/local-smoke/bam.overlap_correction/core-v1-paired-overlap/bamutil")
    );
    assert_eq!(
        case.plan.params["bam"],
        serde_json::json!("assets/toy/core-v1/bam/overlap_correction_paired_overlap.sam")
    );
    assert_eq!(case.plan.params["overlap_method"], serde_json::json!("paired_overlap_correction"));

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
            "overlap_corrected_bam",
            "overlap_corrected_bai",
            "summary",
            "stage_metrics",
            "flagstat_before",
            "flagstat_after",
            "idxstats_before",
            "idxstats_after",
        ]
    );

    let corrected_output = case
        .plan
        .io
        .outputs
        .iter()
        .find(|artifact| artifact.name.as_str() == "overlap_corrected_bam")
        .unwrap_or_else(|| panic!("overlap-corrected BAM output missing from BAM plan"));
    assert_eq!(
        corrected_output.path,
        PathBuf::from(
            "target/local-smoke/bam.overlap_correction/core-v1-paired-overlap/bamutil/overlap.corrected.bam"
        )
    );

    Ok(())
}

#[test]
fn local_overlap_correction_smoke_stage_api_surface_stays_callable() {
    let _: fn(
        &Path,
    ) -> anyhow::Result<
        Vec<bijux_dna_planner_bam::stage_api::LocalOverlapCorrectionSmokeCasePlan>,
    > = bijux_dna_planner_bam::stage_api::local_overlap_correction_smoke_plans;
}

fn write_local_overlap_correction_config(root: &Path, body: &str) -> Result<()> {
    let config_dir = root.join("configs/bench/local");
    fs::create_dir_all(&config_dir)?;
    fs::write(config_dir.join("bam-overlap-correction.toml"), body)?;
    Ok(())
}

#[test]
fn local_overlap_correction_smoke_plans_reject_empty_sample_ids() -> Result<()> {
    let temp = tempfile::tempdir()?;
    write_local_overlap_correction_config(
        temp.path(),
        r#"
schema_version = "bijux.bench.bam.local_overlap_correction.v1"
tool_id = "bamutil"

[[cases]]
sample_id = " "
bam = "assets/toy/core-v1/bam/overlap_correction_paired_overlap.sam"
expected_pair_count = 2
expected_corrected_pairs = 1
expected_corrected_overlap_bases = 7
"#,
    )?;

    let error = bijux_dna_planner_bam::stage_api::local_overlap_correction_smoke_plans(temp.path())
        .expect_err("empty sample_id must be rejected before overlap-correction plan construction");
    assert_eq!(error.to_string(), "local-smoke bam.overlap_correction sample_id must not be empty");
    Ok(())
}

#[test]
fn local_overlap_correction_smoke_plans_reject_duplicate_sample_ids() -> Result<()> {
    let temp = tempfile::tempdir()?;
    write_local_overlap_correction_config(
        temp.path(),
        r#"
schema_version = "bijux.bench.bam.local_overlap_correction.v1"
tool_id = "bamutil"

[[cases]]
sample_id = "duplicate-case"
bam = "assets/toy/core-v1/bam/overlap_correction_paired_overlap.sam"
expected_pair_count = 2
expected_corrected_pairs = 1
expected_corrected_overlap_bases = 7

[[cases]]
sample_id = "duplicate-case"
bam = "assets/toy/core-v1/bam/overlap_correction_paired_overlap.sam"
expected_pair_count = 2
expected_corrected_pairs = 1
expected_corrected_overlap_bases = 7
"#,
    )?;

    let error = bijux_dna_planner_bam::stage_api::local_overlap_correction_smoke_plans(temp.path())
        .expect_err("duplicate sample_id must be rejected before overlap-correction plan construction");
    assert_eq!(
        error.to_string(),
        "duplicate local-smoke bam.overlap_correction sample_id `duplicate-case`"
    );
    Ok(())
}
