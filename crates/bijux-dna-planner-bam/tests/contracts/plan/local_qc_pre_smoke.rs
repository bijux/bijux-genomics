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
fn local_qc_pre_smoke_plans_use_governed_bam_metrics_fixture() -> Result<()> {
    let repo_root = repo_root();
    let plans = bijux_dna_planner_bam::stage_api::local_qc_pre_smoke_plans(&repo_root)?;
    assert_eq!(plans.len(), 1, "governed local-smoke config must keep exactly one BAM qc_pre case");

    let case = plans
        .iter()
        .find(|case| case.sample_id == "human_like_duplicate_flagged_multicontig")
        .unwrap_or_else(|| panic!("governed BAM qc_pre case missing"));
    assert_eq!(case.plan.stage_id.as_str(), "bam.qc_pre");
    assert_eq!(case.plan.tool_id.as_str(), "samtools");
    assert_eq!(case.plan.resources.threads, 4);
    assert_eq!(
        case.bam,
        PathBuf::from(
            "tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_duplicate_flagged_multicontig.sam"
        )
    );
    assert_eq!(case.expected_total_reads, 3);
    assert_eq!(case.expected_mapped_reads, 3);
    assert_eq!(case.expected_unmapped_reads, 0);
    assert_eq!(case.expected_duplicate_flagged_reads, 1);
    assert_eq!(case.expected_contigs, vec!["chr1".to_string(), "chr2".to_string()]);
    assert_eq!(
        case.plan.out_dir,
        PathBuf::from(
            "target/local-smoke/bam.qc_pre/human_like_duplicate_flagged_multicontig/samtools"
        )
    );

    let output_names = case
        .plan
        .io
        .outputs
        .iter()
        .map(|artifact| artifact.name.as_str().to_string())
        .collect::<Vec<_>>();
    assert_eq!(output_names, vec!["flagstat", "idxstats", "stats", "stage_metrics"]);

    let idxstats_output = case
        .plan
        .io
        .outputs
        .iter()
        .find(|artifact| artifact.name.as_str() == "idxstats")
        .unwrap_or_else(|| panic!("idxstats output missing from BAM qc_pre plan"));
    assert_eq!(
        idxstats_output.path,
        PathBuf::from(
            "target/local-smoke/bam.qc_pre/human_like_duplicate_flagged_multicontig/samtools/idxstats.txt"
        )
    );

    Ok(())
}

#[test]
fn local_qc_pre_smoke_stage_api_surface_stays_callable() {
    let _: fn(
        &Path,
    )
        -> anyhow::Result<Vec<bijux_dna_planner_bam::stage_api::LocalQcPreSmokeCasePlan>> =
        bijux_dna_planner_bam::stage_api::local_qc_pre_smoke_plans;
}

fn write_local_qc_pre_config(root: &Path, body: &str) -> Result<()> {
    let config_dir = root.join("configs/bench/local");
    fs::create_dir_all(&config_dir)?;
    fs::write(config_dir.join("bam-qc-pre.toml"), body)?;
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
fn local_qc_pre_smoke_plans_reject_empty_sample_ids() -> Result<()> {
    let temp = tempfile::tempdir()?;
    write_local_qc_pre_config(
        temp.path(),
        r#"
schema_version = "bijux.bench.bam.local_qc_pre.v1"
tool_id = "samtools"

[[cases]]
sample_id = " "
bam = "assets/toy/core-v1/bam/qc_pre_core_metrics.sam"
expected_total_reads = 3
expected_mapped_reads = 3
expected_unmapped_reads = 0
expected_duplicate_flagged_reads = 1
expected_contigs = ["chr1", "chr2"]
"#,
    )?;

    let error = bijux_dna_planner_bam::stage_api::local_qc_pre_smoke_plans(temp.path())
        .expect_err("empty sample_id must be rejected before plan construction");
    assert_eq!(error.to_string(), "local-smoke bam.qc_pre sample_id must not be empty");
    Ok(())
}

#[test]
fn local_qc_pre_smoke_plans_reject_duplicate_sample_ids() -> Result<()> {
    let temp = tempfile::tempdir()?;
    write_local_qc_pre_config(
        temp.path(),
        r#"
schema_version = "bijux.bench.bam.local_qc_pre.v1"
tool_id = "samtools"

[[cases]]
sample_id = "duplicate-case"
bam = "assets/toy/core-v1/bam/qc_pre_core_metrics.sam"
expected_total_reads = 3
expected_mapped_reads = 3
expected_unmapped_reads = 0
expected_duplicate_flagged_reads = 1
expected_contigs = ["chr1", "chr2"]

[[cases]]
sample_id = "duplicate-case"
bam = "assets/toy/core-v1/bam/qc_pre_core_metrics.sam"
expected_total_reads = 3
expected_mapped_reads = 3
expected_unmapped_reads = 0
expected_duplicate_flagged_reads = 1
expected_contigs = ["chr1", "chr2"]
"#,
    )?;

    let error = bijux_dna_planner_bam::stage_api::local_qc_pre_smoke_plans(temp.path())
        .expect_err("duplicate sample_id must be rejected before plan construction");
    assert_eq!(error.to_string(), "duplicate local-smoke bam.qc_pre sample_id `duplicate-case`");
    Ok(())
}

#[test]
fn local_qc_pre_smoke_plans_require_expected_contigs() -> Result<()> {
    let temp = stage_api_temp_repo()?;
    let repo_root = repo_root();
    write_local_qc_pre_config(
        temp.path(),
        &format!(
            r#"
schema_version = "bijux.bench.bam.local_qc_pre.v1"
tool_id = "samtools"

[[cases]]
sample_id = "missing-contigs"
bam = "{bam}"
expected_total_reads = 3
expected_mapped_reads = 3
expected_unmapped_reads = 0
expected_duplicate_flagged_reads = 1
expected_contigs = []
"#,
            bam = repo_root.join("assets/toy/core-v1/bam/qc_pre_core_metrics.sam").display(),
        ),
    )?;

    let error = bijux_dna_planner_bam::stage_api::local_qc_pre_smoke_plans(temp.path())
        .expect_err("qc_pre smoke cases must declare at least one expected contig");
    assert_eq!(
        error.to_string(),
        "local-smoke bam.qc_pre case `missing-contigs` must declare at least one expected contig"
    );
    Ok(())
}

#[test]
fn local_qc_pre_smoke_plans_require_balanced_read_totals() -> Result<()> {
    let temp = stage_api_temp_repo()?;
    let repo_root = repo_root();
    write_local_qc_pre_config(
        temp.path(),
        &format!(
            r#"
schema_version = "bijux.bench.bam.local_qc_pre.v1"
tool_id = "samtools"

[[cases]]
sample_id = "imbalanced-counts"
bam = "{bam}"
expected_total_reads = 3
expected_mapped_reads = 2
expected_unmapped_reads = 0
expected_duplicate_flagged_reads = 1
expected_contigs = ["chr1", "chr2"]
"#,
            bam = repo_root.join("assets/toy/core-v1/bam/qc_pre_core_metrics.sam").display(),
        ),
    )?;

    let error = bijux_dna_planner_bam::stage_api::local_qc_pre_smoke_plans(temp.path())
        .expect_err("qc_pre smoke totals must satisfy mapped + unmapped == total");
    assert_eq!(
        error.to_string(),
        "local-smoke bam.qc_pre case `imbalanced-counts` must satisfy mapped + unmapped == total"
    );
    Ok(())
}
