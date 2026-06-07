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
        .find(|case| case.sample_id == "human_like_paired_overlap_control")
        .unwrap_or_else(|| panic!("governed BAM overlap-correction case missing"));
    assert_eq!(case.plan.stage_id.as_str(), "bam.overlap_correction");
    assert_eq!(case.plan.tool_id.as_str(), "bamutil");
    assert_eq!(case.plan.resources.threads, 2);
    assert_eq!(
        case.bam,
        PathBuf::from(
            "benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_paired_overlap_control.sam",
        )
    );
    assert_eq!(case.expected_pair_count, 2);
    assert_eq!(case.expected_corrected_pairs, 1);
    assert_eq!(case.expected_corrected_overlap_bases, 7);
    assert_eq!(
        case.plan.out_dir,
        PathBuf::from(
            "runs/bench/local-smoke/bam.overlap_correction/human_like_paired_overlap_control/bamutil",
        )
    );
    assert_eq!(
        case.plan.params["bam"],
        serde_json::json!(
            "benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_paired_overlap_control.sam"
        )
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
            "runs/bench/local-smoke/bam.overlap_correction/human_like_paired_overlap_control/bamutil/overlap.corrected.bam"
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
    let config_dir = root.join("benchmarks/configs/local");
    fs::create_dir_all(&config_dir)?;
    fs::write(config_dir.join("bam-overlap-correction.toml"), body)?;
    Ok(())
}

fn stage_api_temp_repo() -> Result<tempfile::TempDir> {
    let temp = tempfile::tempdir()?;
    let repo_root = repo_root();
    let tool_dir = temp.path().join("domain/bam/tools");
    fs::create_dir_all(&tool_dir)?;
    fs::copy(repo_root.join("domain/bam/tools/bamutil.yaml"), tool_dir.join("bamutil.yaml"))?;
    Ok(temp)
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
        .expect_err(
            "duplicate sample_id must be rejected before overlap-correction plan construction",
        );
    assert_eq!(
        error.to_string(),
        "duplicate local-smoke bam.overlap_correction sample_id `duplicate-case`"
    );
    Ok(())
}

#[test]
fn local_overlap_correction_smoke_plans_require_positive_pair_count() -> Result<()> {
    let temp = stage_api_temp_repo()?;
    let repo_root = repo_root();
    write_local_overlap_correction_config(
        temp.path(),
        &format!(
            r#"
schema_version = "bijux.bench.bam.local_overlap_correction.v1"
tool_id = "bamutil"

[[cases]]
sample_id = "zero-pairs"
bam = "{bam}"
expected_pair_count = 0
expected_corrected_pairs = 0
expected_corrected_overlap_bases = 0
"#,
            bam = repo_root
                .join(
                    "benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_paired_overlap_control.sam",
                )
                .display(),
        ),
    )?;

    let error = bijux_dna_planner_bam::stage_api::local_overlap_correction_smoke_plans(temp.path())
        .expect_err("overlap-correction cases must declare pair_count greater than zero");
    assert_eq!(
        error.to_string(),
        "local-smoke bam.overlap_correction case `zero-pairs` must declare expected_pair_count greater than zero"
    );
    Ok(())
}

#[test]
fn local_overlap_correction_smoke_plans_reject_corrected_pairs_above_pair_count() -> Result<()> {
    let temp = stage_api_temp_repo()?;
    let repo_root = repo_root();
    write_local_overlap_correction_config(
        temp.path(),
        &format!(
            r#"
schema_version = "bijux.bench.bam.local_overlap_correction.v1"
tool_id = "bamutil"

[[cases]]
sample_id = "too-many-corrected-pairs"
bam = "{bam}"
expected_pair_count = 2
expected_corrected_pairs = 3
expected_corrected_overlap_bases = 7
"#,
            bam = repo_root
                .join(
                    "benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_paired_overlap_control.sam",
                )
                .display(),
        ),
    )?;

    let error = bijux_dna_planner_bam::stage_api::local_overlap_correction_smoke_plans(temp.path())
        .expect_err("corrected_pairs must not exceed pair_count");
    assert_eq!(
        error.to_string(),
        "local-smoke bam.overlap_correction case `too-many-corrected-pairs` cannot declare corrected pairs greater than pair count"
    );
    Ok(())
}

#[test]
fn local_overlap_correction_smoke_plans_require_overlap_bases_when_pairs_corrected() -> Result<()> {
    let temp = stage_api_temp_repo()?;
    let repo_root = repo_root();
    write_local_overlap_correction_config(
        temp.path(),
        &format!(
            r#"
schema_version = "bijux.bench.bam.local_overlap_correction.v1"
tool_id = "bamutil"

[[cases]]
sample_id = "missing-overlap-bases"
bam = "{bam}"
expected_pair_count = 2
expected_corrected_pairs = 1
expected_corrected_overlap_bases = 0
"#,
            bam = repo_root
                .join(
                    "benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_paired_overlap_control.sam",
                )
                .display(),
        ),
    )?;

    let error = bijux_dna_planner_bam::stage_api::local_overlap_correction_smoke_plans(temp.path())
        .expect_err("corrected pairs must imply corrected overlap bases");
    assert_eq!(
        error.to_string(),
        "local-smoke bam.overlap_correction case `missing-overlap-bases` must declare positive expected_corrected_overlap_bases when expected_corrected_pairs is greater than zero"
    );
    Ok(())
}

#[test]
fn local_overlap_correction_smoke_plans_require_zero_overlap_bases_when_no_pairs_corrected(
) -> Result<()> {
    let temp = stage_api_temp_repo()?;
    let repo_root = repo_root();
    write_local_overlap_correction_config(
        temp.path(),
        &format!(
            r#"
schema_version = "bijux.bench.bam.local_overlap_correction.v1"
tool_id = "bamutil"

[[cases]]
sample_id = "orphan-overlap-bases"
bam = "{bam}"
expected_pair_count = 2
expected_corrected_pairs = 0
expected_corrected_overlap_bases = 5
"#,
            bam = repo_root
                .join(
                    "benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_paired_overlap_control.sam",
                )
                .display(),
        ),
    )?;

    let error = bijux_dna_planner_bam::stage_api::local_overlap_correction_smoke_plans(temp.path())
        .expect_err("overlap bases must stay zero when no pairs are corrected");
    assert_eq!(
        error.to_string(),
        "local-smoke bam.overlap_correction case `orphan-overlap-bases` must keep expected_corrected_overlap_bases at zero when expected_corrected_pairs is zero"
    );
    Ok(())
}
