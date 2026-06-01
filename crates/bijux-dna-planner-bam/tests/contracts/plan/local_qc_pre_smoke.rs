use anyhow::Result;
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
        .find(|case| case.sample_id == "core-v1-duplicate-contigs")
        .unwrap_or_else(|| panic!("governed BAM qc_pre case missing"));
    assert_eq!(case.plan.stage_id.as_str(), "bam.qc_pre");
    assert_eq!(case.plan.tool_id.as_str(), "samtools");
    assert_eq!(case.plan.resources.threads, 4);
    assert_eq!(case.bam, PathBuf::from("assets/toy/core-v1/bam/qc_pre_core_metrics.sam"));
    assert_eq!(case.expected_total_reads, 3);
    assert_eq!(case.expected_mapped_reads, 3);
    assert_eq!(case.expected_unmapped_reads, 0);
    assert_eq!(case.expected_duplicate_flagged_reads, 1);
    assert_eq!(case.expected_contigs, vec!["chr1".to_string(), "chr2".to_string()]);
    assert_eq!(
        case.plan.out_dir,
        PathBuf::from("target/local-smoke/bam.qc_pre/core-v1-duplicate-contigs/samtools")
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
            "target/local-smoke/bam.qc_pre/core-v1-duplicate-contigs/samtools/idxstats.txt"
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
