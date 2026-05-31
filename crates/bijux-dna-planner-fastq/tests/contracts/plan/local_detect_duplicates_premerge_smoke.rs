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
fn local_detect_duplicates_premerge_smoke_plans_use_governed_toy_fixtures() -> Result<()> {
    let repo_root = repo_root();
    let plans =
        bijux_dna_planner_fastq::stage_api::local_detect_duplicates_premerge_smoke_plans(
            &repo_root,
        )?;
    assert_eq!(plans.len(), 2, "governed local-smoke config must keep duplicate-hit and clear coverage");

    let duplicate_hit = plans
        .iter()
        .find(|case| case.sample_id == "duplicate-hit-pe")
        .unwrap_or_else(|| panic!("duplicate-hit-pe case missing from local duplicate smoke plans"));
    assert_eq!(duplicate_hit.plan.stage_id.as_str(), "fastq.detect_duplicates_premerge");
    assert_eq!(duplicate_hit.plan.tool_id.as_str(), "bijux_dna");
    assert_eq!(
        duplicate_hit.r1,
        PathBuf::from("assets/toy/core-v1/fastq/duplicate_pairs_R1.fastq")
    );
    assert_eq!(
        duplicate_hit.r2,
        Some(PathBuf::from("assets/toy/core-v1/fastq/duplicate_pairs_R2.fastq"))
    );
    assert_eq!(
        duplicate_hit.plan.out_dir,
        PathBuf::from(
            "target/local-smoke/fastq.detect_duplicates_premerge/duplicate-hit-pe/bijux_dna"
        )
    );
    assert_eq!(duplicate_hit.plan.resources.threads, 1);
    assert_eq!(
        duplicate_hit.plan.params["duplicate_signal_report"],
        serde_json::json!(
            "target/local-smoke/fastq.detect_duplicates_premerge/duplicate-hit-pe/bijux_dna/duplicate_signal_report.json"
        )
    );

    let duplicate_clear = plans
        .iter()
        .find(|case| case.sample_id == "duplicate-clear-pe")
        .unwrap_or_else(|| panic!("duplicate-clear-pe case missing from local duplicate smoke plans"));
    assert_eq!(
        duplicate_clear.r1,
        PathBuf::from("assets/toy/core-v1/fastq/reads_1.fastq")
    );
    assert_eq!(
        duplicate_clear.r2,
        Some(PathBuf::from("assets/toy/core-v1/fastq/reads_2.fastq"))
    );
    assert_eq!(
        duplicate_clear.plan.out_dir,
        PathBuf::from(
            "target/local-smoke/fastq.detect_duplicates_premerge/duplicate-clear-pe/bijux_dna"
        )
    );
    assert_eq!(duplicate_clear.plan.effective_params["paired_mode"], serde_json::json!("paired_end"));
    assert_eq!(duplicate_clear.plan.effective_params["advisory_only"], serde_json::json!(true));

    Ok(())
}

#[test]
fn local_detect_duplicates_premerge_smoke_stage_api_surface_stays_callable() {
    let _: fn(
        &Path,
    ) -> anyhow::Result<Vec<bijux_dna_planner_fastq::LocalDetectDuplicatesPremergeSmokeCasePlan>> =
        bijux_dna_planner_fastq::stage_api::local_detect_duplicates_premerge_smoke_plans;
}
